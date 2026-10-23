use std::net::IpAddr;

use anyhow::{Context, Result, bail, ensure};

pub(super) struct ClientPatch {
    pub(super) content: String,
    pub(super) applicable: usize,
    pub(super) changed: usize,
}

pub(super) fn has_client_includes(input: &str) -> bool {
    input
        .lines()
        .any(|line| directive(line).is_some_and(|d| d.key.eq_ignore_ascii_case("Include")))
}

pub(super) fn patch_client_config(input: &str) -> Result<ClientPatch> {
    let sections = client_sections(input);
    let first_match = input
        .split_inclusive('\n')
        .enumerate()
        .find(|(_, line)| {
            directive(split_line(line).0).is_some_and(|d| d.key.eq_ignore_ascii_case("Match"))
        })
        .map(|(line, _)| line + 1);
    let mut patch = ClientPatch {
        content: String::with_capacity(input.len()),
        applicable: 0,
        changed: 0,
    };
    for (index, section) in sections.iter().enumerate() {
        let routes = patch_client_routes(section.text, section.start_line)?;
        if routes.applicable == 0 {
            patch.content.push_str(section.text);
            continue;
        }
        ensure!(
            first_match.is_none(),
            "SSH config line {}: Match makes WSL Host lifecycle settings conditional; manual review required",
            first_match.unwrap_or(0)
        );
        let selectors = section.selectors.as_ref().context(
            "WSL ProxyCommand outside a Host stanza is ambiguous; scope it to the intended Host",
        )?;
        ensure!(
            !selectors.is_empty()
                && selectors
                    .iter()
                    .all(|selector| !selector.contains(['\'', '"', '\\'])),
            "WSL Host selector syntax requires manual review"
        );
        for earlier in &sections[..index] {
            if selectors_may_overlap(earlier.selectors.as_deref(), selectors) {
                for line in earlier.text.split_inclusive('\n') {
                    if let Some(d) = directive(split_line(line).0) {
                        let conflict = CLIENT_SETTINGS.iter().any(|(key, target)| {
                            d.key.eq_ignore_ascii_case(key)
                                && before_comment(&split_line(line).0[d.value_start..]).trim()
                                    != *target
                        });
                        ensure!(
                            !conflict && !d.key.eq_ignore_ascii_case("ProxyCommand"),
                            "earlier global or overlapping Host {} setting can override WSL lifecycle protection; manual review required",
                            d.key
                        );
                    }
                }
            }
        }
        let content = patch_settings(
            &routes.content,
            &CLIENT_SETTINGS,
            "  ",
            "SSH WSL Host config",
        )?;
        patch.changed += if content != routes.content {
            routes.applicable
        } else {
            routes.changed
        };
        patch.applicable += routes.applicable;
        patch.content.push_str(&content);
    }
    Ok(patch)
}

const CLIENT_SETTINGS: [(&str, &str); 2] =
    [("ServerAliveInterval", "10"), ("ServerAliveCountMax", "2")];

struct ClientSection<'a> {
    text: &'a str,
    start_line: usize,
    selectors: Option<Vec<&'a str>>,
}

fn client_sections(input: &str) -> Vec<ClientSection<'_>> {
    let mut sections = Vec::new();
    let mut start = 0;
    let mut start_line = 0;
    let mut offset = 0;
    let mut selectors = None;
    for (line_number, line) in input.split_inclusive('\n').enumerate() {
        let body = split_line(line).0;
        if let Some(d) = directive(body).filter(|d| d.key.eq_ignore_ascii_case("Host")) {
            sections.push(ClientSection {
                text: &input[start..offset],
                start_line,
                selectors,
            });
            start = offset;
            start_line = line_number;
            selectors = Some(
                before_comment(&body[d.value_start..])
                    .split_ascii_whitespace()
                    .collect(),
            );
        }
        offset += line.len();
    }
    sections.push(ClientSection {
        text: &input[start..],
        start_line,
        selectors,
    });
    sections
}

// Exact, disjoint aliases can be proven independent. Pattern intersections are
// deliberately conservative rather than guessing effective first-wins values.
fn selectors_may_overlap(earlier: Option<&[&str]>, current: &[&str]) -> bool {
    let Some(earlier) = earlier else {
        return true;
    };
    earlier
        .iter()
        .chain(current)
        .any(|name| name.contains(['*', '?', '[', '!', '"', '\'']))
        || earlier
            .iter()
            .any(|name| current.iter().any(|other| name.eq_ignore_ascii_case(other)))
}

fn patch_client_routes(input: &str, line_offset: usize) -> Result<ClientPatch> {
    let mut patch = ClientPatch {
        content: String::with_capacity(input.len()),
        applicable: 0,
        changed: 0,
    };
    for (index, line) in input.split_inclusive('\n').enumerate() {
        let (body, ending) = split_line(line);
        let Some(directive) = directive(body) else {
            patch.content.push_str(line);
            continue;
        };
        if !directive.key.eq_ignore_ascii_case("ProxyCommand") {
            patch.content.push_str(line);
            continue;
        }
        let command = &body[directive.value_start..];
        let result = patch_proxy_command(command).with_context(|| {
            format!(
                "SSH config line {}: unsupported WSL nc route",
                line_offset + index + 1
            )
        })?;
        match result {
            None => patch.content.push_str(line),
            Some(replacement) => {
                patch.applicable += 1;
                if let Some(command) = replacement {
                    patch.content.push_str(&body[..directive.value_start]);
                    patch.content.push_str(&command);
                    patch.content.push_str(ending);
                    patch.changed += 1;
                } else {
                    patch.content.push_str(line);
                }
            }
        }
    }
    Ok(patch)
}

pub(super) fn patch_endpoint_config(input: &str) -> Result<String> {
    let settings = [
        ("ClientAliveInterval", "10"),
        ("ClientAliveCountMax", "2"),
        ("LoginGraceTime", "30"),
    ];
    for (line_number, line) in input.split_inclusive('\n').enumerate() {
        let Some(directive) = directive(split_line(line).0) else {
            continue;
        };
        ensure!(
            !directive.key.eq_ignore_ascii_case("Include")
                && !directive.key.eq_ignore_ascii_case("Match"),
            "Tersh endpoint config line {} uses {}; resolve included or conditional settings before applying lifecycle limits",
            line_number + 1,
            directive.key
        );
    }
    patch_settings(input, &settings, "", "Tersh endpoint config")
}

fn patch_settings(
    input: &str,
    settings: &[(&str, &str)],
    indent: &str,
    description: &str,
) -> Result<String> {
    let mut found = vec![false; settings.len()];
    let mut output = String::with_capacity(input.len());
    for (line_number, line) in input.split_inclusive('\n').enumerate() {
        let (body, ending) = split_line(line);
        let Some(directive) = directive(body) else {
            output.push_str(line);
            continue;
        };
        let Some(index) = settings
            .iter()
            .position(|(key, _)| directive.key.eq_ignore_ascii_case(key))
        else {
            output.push_str(line);
            continue;
        };
        let value = &body[directive.value_start..];
        let value = before_comment(value).trim_end_matches([' ', '\t']);
        ensure!(
            value.split_ascii_whitespace().count() == 1,
            "{} line {} has an ambiguous {} value",
            description,
            line_number + 1,
            directive.key
        );
        found[index] = true;
        if value == settings[index].1 {
            output.push_str(line);
        } else {
            output.push_str(&body[..directive.value_start]);
            output.push_str(settings[index].1);
            output.push_str(&body[directive.value_start + value.len()..]);
            output.push_str(ending);
        }
    }

    let newline = input.find('\n').map_or("\n", |index| {
        if index > 0 && input.as_bytes()[index - 1] == b'\r' {
            "\r\n"
        } else {
            "\n"
        }
    });
    let final_newline = input.is_empty() || input.ends_with('\n');
    let missing: Vec<_> = settings
        .iter()
        .enumerate()
        .filter(|(index, _)| !found[*index])
        .map(|(_, setting)| setting)
        .collect();
    for (index, (key, value)) in missing.iter().enumerate() {
        if !output.is_empty() && !output.ends_with('\n') {
            output.push_str(newline);
        }
        output.push_str(indent);
        output.push_str(key);
        output.push(' ');
        output.push_str(value);
        if final_newline || index + 1 < missing.len() {
            output.push_str(newline);
        }
    }
    Ok(output)
}

fn split_line(line: &str) -> (&str, &str) {
    if let Some(body) = line.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = line.strip_suffix('\n') {
        (body, "\n")
    } else {
        (line, "")
    }
}

struct Directive<'a> {
    key: &'a str,
    value_start: usize,
}

fn directive(line: &str) -> Option<Directive<'_>> {
    let start = line.len() - line.trim_start_matches([' ', '\t']).len();
    let remainder = &line[start..];
    if remainder.is_empty() || remainder.starts_with('#') {
        return None;
    }
    let key_end = remainder.find([' ', '\t', '=']).unwrap_or(remainder.len());
    let key = &remainder[..key_end];
    let value = remainder[key_end..].trim_start_matches([' ', '\t']);
    let value = value
        .strip_prefix('=')
        .unwrap_or(value)
        .trim_start_matches([' ', '\t']);
    Some(Directive {
        key,
        value_start: line.len() - value.len(),
    })
}

fn before_comment(value: &str) -> &str {
    value
        .char_indices()
        .find(|(index, ch)| {
            *ch == '#' && (*index == 0 || matches!(value.as_bytes()[index - 1], b' ' | b'\t'))
        })
        .map_or(value, |(index, _)| &value[..index])
}

// This deliberately accepts a narrow, flat command form. Shell wrappers need an
// explicit migration rather than a text replacement that might alter quoting.
// None is unrelated; Some(None) is applicable and already safe.
fn patch_proxy_command(command: &str) -> Result<Option<Option<String>>> {
    let active = before_comment(command);
    if !active.to_ascii_lowercase().contains("wsl.exe") || !active.contains("/usr/bin/nc") {
        return Ok(None);
    }
    ensure!(
        !active.contains(['\'', '"', '\\', '`', '$', ';', '|', '&', '(', ')', '<', '>'])
            && !active.chars().any(|ch| ch.is_control() && ch != '\t'),
        "quoted commands, substitutions and shell operators require manual review"
    );
    let words: Vec<_> = active.split_ascii_whitespace().collect();
    let wsl = words
        .iter()
        .position(|word| word.eq_ignore_ascii_case("wsl.exe"))
        .context("expected a plain wsl.exe invocation")?;
    validate_ssh_prefix(&words[..wsl])?;
    let mut nc = wsl + 1;
    while nc < words.len() && words[nc] != "/usr/bin/nc" {
        match words[nc] {
            "-d" | "--distribution" | "-u" | "--user" | "--cd" => {
                ensure!(nc + 1 < words.len(), "missing WSL option argument");
                ensure!(
                    !words[nc + 1].starts_with('-'),
                    "invalid WSL option argument"
                );
                nc += 2;
            }
            "--" | "-e" | "--exec" => {
                nc += 1;
                break;
            }
            _ => bail!("unsupported WSL invocation; expected direct /usr/bin/nc execution"),
        }
    }
    ensure!(
        words.get(nc) == Some(&"/usr/bin/nc"),
        "expected direct /usr/bin/nc execution"
    );
    let mut index = nc + 1;
    let mut q = Vec::new();
    let mut w = Vec::new();
    while index < words.len() && words[index].starts_with('-') {
        let option = words[index];
        let (key, argument) = if matches!(option, "-q" | "-w") {
            index += 1;
            (
                option,
                *words.get(index).context("missing nc timeout argument")?,
            )
        } else if let Some(argument) = option.strip_prefix("-q") {
            ("-q", argument)
        } else if let Some(argument) = option.strip_prefix("-w") {
            ("-w", argument)
        } else {
            bail!("unsupported nc option {option}; only -q and -w are normalized");
        };
        let value: i64 = argument.parse().context("invalid nc timeout argument")?;
        if key == "-q" {
            q.push(value);
        } else {
            ensure!(value >= 0, "nc -w must not be negative");
            w.push(value);
        }
        index += 1;
    }
    ensure!(
        words.len() == index + 2,
        "expected a literal nc loopback address and port without a shell suffix"
    );
    let address = words[index].parse::<IpAddr>();
    if !address.is_ok_and(|address| address.is_loopback()) {
        return Ok(None);
    }
    let port = words[index + 1];
    ensure!(
        port.bytes().all(|byte| byte.is_ascii_digit())
            && port.parse::<u16>().is_ok_and(|port| port != 0),
        "nc loopback port must be a literal integer in 1..65535"
    );
    if q == [0] && w == [30] {
        return Ok(Some(None));
    }
    // Slices refer to the original command, retaining the route, WSL options,
    // address, port, comments, and trailing whitespace verbatim.
    let nc_end = words[nc].as_ptr() as usize - command.as_ptr() as usize + words[nc].len();
    let address_start = words[index].as_ptr() as usize - command.as_ptr() as usize;
    Ok(Some(Some(format!(
        "{} -q 0 -w 30 {}",
        &command[..nc_end],
        &command[address_start..]
    ))))
}

fn validate_ssh_prefix(words: &[&str]) -> Result<()> {
    let mut index = 0;
    while index < words.len() {
        ensure!(
            words[index].rsplit('/').next() == Some("ssh"),
            "expected an SSH relay chain before wsl.exe"
        );
        index += 1;
        while index < words.len() && words[index].starts_with('-') {
            let option = words[index];
            if option == "--" {
                index += 1;
                break;
            }
            let argument_options = [
                "-b", "-c", "-E", "-e", "-F", "-i", "-J", "-l", "-m", "-o", "-p", "-S", "-B",
            ];
            if argument_options.contains(&option) {
                ensure!(index + 1 < words.len(), "missing SSH option argument");
                index += 2;
            } else if argument_options
                .iter()
                .any(|prefix| option.starts_with(prefix) && option.len() > prefix.len())
                || matches!(
                    option,
                    "-T" | "-4"
                        | "-6"
                        | "-a"
                        | "-A"
                        | "-C"
                        | "-K"
                        | "-k"
                        | "-q"
                        | "-t"
                        | "-tt"
                        | "-v"
                        | "-vv"
                        | "-vvv"
                        | "-x"
                        | "-X"
                        | "-Y"
                )
            {
                index += 1;
            } else {
                bail!("unsupported SSH relay option {option}");
            }
        }
        ensure!(
            index < words.len() && !words[index].starts_with('-'),
            "missing SSH relay host"
        );
        index += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROUTE: &str =
        "ssh -T -o BatchMode=yes relay ssh -T windows wsl.exe -d Debian -u alice -- /usr/bin/nc";
    const ALIVE: &str = "  ServerAliveInterval 10\n  ServerAliveCountMax 2\n";

    #[test]
    fn patches_only_wsl_loopback_routes_and_preserves_the_ssh_chain() {
        let input = format!(
            "# retained\nHost lab\n{ALIVE}  ProxyCommand {ROUTE} 127.0.0.1 2207\n  User alice\nHost plain\n  ProxyCommand ssh -W %h:%p relay\n"
        );
        let patch = patch_client_config(&input).unwrap();
        assert_eq!(patch.applicable, 1);
        assert_eq!(patch.changed, 1);
        assert_eq!(
            patch.content,
            input.replace("/usr/bin/nc 127.0.0.1", "/usr/bin/nc -q 0 -w 30 127.0.0.1")
        );
    }

    #[test]
    fn normalizes_missing_or_unsafe_bounds() {
        for options in [
            "",
            " -q 0",
            " -w 30",
            " -q -1 -w 0",
            " -w5 -q2",
            " -q 0 -q 5 -w 30",
        ] {
            let input = format!("Host lab\n{ALIVE}ProxyCommand {ROUTE}{options} 127.0.0.1 2222\n");
            let patch = patch_client_config(&input).unwrap();
            assert_eq!(
                patch.content,
                format!("Host lab\n{ALIVE}ProxyCommand {ROUTE} -q 0 -w 30 127.0.0.1 2222\n")
            );
            assert_eq!(patch.changed, 1);
            assert_eq!(patch.applicable, 1);
        }
    }

    #[test]
    fn safe_config_is_byte_identical_and_second_install_is_a_noop() {
        for options in [" -q 0 -w 30", "\t-w30  -q0"] {
            let input = format!(
                "Host lab\r\n  ServerAliveInterval 10\r\n  ServerAliveCountMax 2\r\nProxyCommand {ROUTE}{options} ::1 65535 # keep this\r\n"
            );
            let patch = patch_client_config(&input).unwrap();
            assert_eq!(patch.content, input);
            assert_eq!(patch.applicable, 1);
            assert_eq!(patch.changed, 0);
        }
        let first =
            patch_client_config(&format!("Host lab\nProxyCommand {ROUTE} 127.0.0.2 1")).unwrap();
        let second = patch_client_config(&first.content).unwrap();
        assert_eq!(second.content, first.content);
        assert_eq!(second.changed, 0);
        assert_eq!(second.applicable, 1);
    }

    #[test]
    fn preserves_mixed_case_crlf_equals_and_missing_final_newline() {
        let input = format!(
            "Host example\r\n  ServerAliveInterval 10\r\n  ServerAliveCountMax 2\r\n\tpRoXyCoMmAnD = {ROUTE} 127.0.0.1 2223"
        );
        let patch = patch_client_config(&input).unwrap();
        assert_eq!(
            patch.content,
            input.replace("/usr/bin/nc 127.0.0.1", "/usr/bin/nc -q 0 -w 30 127.0.0.1")
        );
        assert!(!patch.content.ends_with('\n'));
    }

    #[test]
    fn supports_direct_wsl_and_all_loopback_addresses_without_host_constants() {
        for endpoint in ["127.12.34.56 1", "::1 65535"] {
            let input = format!(
                "Host lab\n{ALIVE}ProxyCommand wsl.exe --distribution Other --user bob --exec /usr/bin/nc {endpoint}\n"
            );
            let patch = patch_client_config(&input).unwrap();
            assert_eq!(
                patch.content,
                input.replace("/usr/bin/nc ", "/usr/bin/nc -q 0 -w 30 ")
            );
            assert_eq!(patch.applicable, 1);
        }
    }

    #[test]
    fn unrelated_nc_and_nonloopback_routes_are_unchanged() {
        let input = format!(
            "# ProxyCommand {ROUTE} 127.0.0.1 2222\nHost plain\nProxyCommand nc 127.0.0.1 2222\nProxyCommand {ROUTE} 192.0.2.5 2222\nProxyCommand wsl.exe -- /usr/bin/nc localhost 2222\nInclude other.conf\n"
        );
        let patch = patch_client_config(&input).unwrap();
        assert_eq!(patch.content, input);
        assert_eq!(patch.applicable, 0);
        assert_eq!(patch.changed, 0);
    }

    #[test]
    fn rejects_known_wsl_routes_with_unsupported_options_or_shell_composition() {
        for command in [
            format!("{ROUTE} -z 127.0.0.1 2222"),
            format!("{ROUTE} -q 0 -w bad 127.0.0.1 2222"),
            format!("{ROUTE} 127.0.0.1 0"),
            format!("{ROUTE} 127.0.0.1 65536"),
            format!("{ROUTE} 127.0.0.1 2222; echo done"),
            format!("{ROUTE} 127.0.0.1 2222 | cat"),
            format!("{ROUTE} 127.0.0.1 $(get-port)"),
            format!("{ROUTE} 127.0.0.1 `get-port`"),
            "ssh relay \"wsl.exe -- /usr/bin/nc 127.0.0.1 2222\"".to_string(),
            "ssh relay 'wsl.exe -- /usr/bin/nc 127.0.0.1 2222'".to_string(),
            "ssh relay printf wsl.exe -- /usr/bin/nc 127.0.0.1 2222".to_string(),
            "wsl.exe -d Debian -- sh -c '/usr/bin/nc 127.0.0.1 2222'".to_string(),
        ] {
            let error = patch_client_config(&format!("Host custom\n  ProxyCommand {command}\n"))
                .err()
                .expect(&command);
            let text = error.to_string();
            assert!(text.contains("line 2"), "{text}");
            assert!(text.contains("WSL"), "{text}");
        }
    }

    #[test]
    fn client_keepalive_protects_idle_sessions_before_the_server_is_upgraded() {
        let input = format!(
            "Host ordinary\n  ServerAliveInterval 30\n  ServerAliveCountMax 3\nHost lab\n  ProxyCommand {ROUTE} -q 0 -w 30 127.0.0.1 2222\n  ServerAliveInterval 30 # old default\n  ServerAliveCountMax 3\n"
        );
        let expected = format!(
            "Host ordinary\n  ServerAliveInterval 30\n  ServerAliveCountMax 3\nHost lab\n  ProxyCommand {ROUTE} -q 0 -w 30 127.0.0.1 2222\n  ServerAliveInterval 10 # old default\n  ServerAliveCountMax 2\n"
        );
        let patch = patch_client_config(&input).unwrap();
        assert_eq!(patch.content, expected);
        assert_eq!(patch.applicable, 1);
        assert_eq!(patch.changed, 1);
    }

    #[test]
    fn appends_missing_client_keepalive_limits_to_each_wsl_host() {
        let input = format!(
            "Host first\r\n  ProxyCommand {ROUTE} -q 0 -w 30 127.0.0.1 2222\r\nHost second\r\n  ProxyCommand {ROUTE} -q 0 -w 30 ::1 2223"
        );
        let expected = format!(
            "Host first\r\n  ProxyCommand {ROUTE} -q 0 -w 30 127.0.0.1 2222\r\n  ServerAliveInterval 10\r\n  ServerAliveCountMax 2\r\nHost second\r\n  ProxyCommand {ROUTE} -q 0 -w 30 ::1 2223\r\n  ServerAliveInterval 10\r\n  ServerAliveCountMax 2"
        );
        let patch = patch_client_config(&input).unwrap();
        assert_eq!(patch.content, expected);
        assert_eq!(patch.applicable, 2);
        assert_eq!(patch.changed, 2);
        assert_eq!(patch_client_config(&patch.content).unwrap().changed, 0);
    }

    #[test]
    fn normalizes_all_duplicate_alive_directives_inside_wsl_host() {
        let input = format!(
            "Host lab\n  ProxyCommand {ROUTE} -q 0 -w 30 127.0.0.1 2222\n  ServerAliveInterval 30\n  ServerAliveInterval 40\n  ServerAliveCountMax 3\n  ServerAliveCountMax 4\n"
        );
        let patch = patch_client_config(&input).unwrap();
        assert_eq!(patch.content.matches("ServerAliveInterval 10").count(), 2);
        assert_eq!(patch.content.matches("ServerAliveCountMax 2").count(), 2);
        assert_eq!(patch.changed, 1);
    }

    #[test]
    fn refuses_unscoped_conditional_or_prior_conflicting_shared_settings() {
        let proxy = format!("  ProxyCommand {ROUTE} 127.0.0.1 2222\n");
        for input in [
            proxy.clone(),
            format!("Host lab\n{proxy}Match user alice\n  ServerAliveInterval 30\n"),
            format!("ServerAliveInterval 30\nHost lab\n{proxy}"),
            format!("Host *\n  ServerAliveInterval 30\nHost lab\n{proxy}"),
            format!("Host lab alias\n  ServerAliveCountMax 3\nHost alias\n{proxy}"),
        ] {
            assert!(patch_client_config(&input).is_err(), "{input}");
        }
        assert!(
            patch_client_config(
                "Host ordinary\n  ServerAliveInterval 30\nMatch all\n  User alice\n"
            )
            .is_ok()
        );
    }

    #[test]
    fn sets_endpoint_lifecycle_limits_and_preserves_unrelated_fields_and_comments() {
        let input = "# Tersh listener\nPort 2300\n  clientaliveinterval 0 # check across proxy\nClientAliveCountMax=3\nLoginGraceTime 120\nPasswordAuthentication no\n";
        let expected = "# Tersh listener\nPort 2300\n  clientaliveinterval 10 # check across proxy\nClientAliveCountMax=2\nLoginGraceTime 30\nPasswordAuthentication no\n";
        assert_eq!(patch_endpoint_config(input).unwrap(), expected);
    }

    #[test]
    fn appends_missing_endpoint_limits_preserving_newline_style_and_termination() {
        assert_eq!(
            patch_endpoint_config("Port 2300\r\nClientAliveInterval 10\r\n").unwrap(),
            "Port 2300\r\nClientAliveInterval 10\r\nClientAliveCountMax 2\r\nLoginGraceTime 30\r\n"
        );
        assert_eq!(
            patch_endpoint_config("Port 2300").unwrap(),
            "Port 2300\nClientAliveInterval 10\nClientAliveCountMax 2\nLoginGraceTime 30"
        );
    }

    #[test]
    fn endpoint_patch_is_byte_identical_when_already_safe_and_idempotent() {
        let input = "# fixed\r\nPort 2300\r\nclientaliveinterval = 10 # keep\r\nClientAliveCountMax 2\r\nLoginGraceTime\t30";
        assert_eq!(patch_endpoint_config(input).unwrap(), input);
        let first = patch_endpoint_config("Port 2300\n").unwrap();
        assert_eq!(patch_endpoint_config(&first).unwrap(), first);
    }

    #[test]
    fn refuses_endpoint_includes_matches_and_ambiguous_target_values() {
        for directive in [
            "Include /etc/ssh/conf.d/*",
            "include=other",
            "Match User alice",
            "ClientAliveInterval 10 extra",
            "LoginGraceTime",
        ] {
            assert!(
                patch_endpoint_config(&format!("Port 2300\n{directive}\n")).is_err(),
                "{directive}"
            );
        }
        assert!(
            patch_endpoint_config("# Include optional\n# Match User root\nPort 2300\n").is_ok()
        );
    }
}
