//! Keep network I/O off the UI thread. Confirmation happens after raw mode
//! is restored, so typing into a filename, shell or filter can never update Tersh.
use crate::update::{self, Release};
use anyhow::{Context, Result};
use ratatui::{Frame, layout::Rect, text::Line};
use std::{
    fs::OpenOptions,
    io::{self, BufRead, IsTerminal, Write},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

pub fn update_command(install: bool, yes: bool) -> Result<()> {
    let release = update::check()?;
    let current = env!("CARGO_PKG_VERSION");
    if !release.is_newer() {
        println!(
            "Tersh {current} is up to date (latest stable: {}).",
            release.tag()
        );
        return Ok(());
    }
    if !install {
        println!(
            "Tersh {current} -> {}. Run tersh --update to install.",
            release.tag()
        );
        return Ok(());
    }
    install_if_confirmed(&release, yes)
}

fn install_if_confirmed(release: &Release, yes: bool) -> Result<()> {
    if !yes {
        let mut terminal = OpenOptions::new().read(true).write(true).open("/dev/tty")
            .context("a terminal is required for confirmation; for unattended installation use tersh --update --yes")?;
        write!(
            terminal,
            "Update Tersh {} to {} now? [y/N] ",
            env!("CARGO_PKG_VERSION"),
            release.tag()
        )?;
        terminal.flush()?;
        let mut line = String::new();
        io::BufReader::new(terminal).read_line(&mut line)?;
        if !matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            return Ok(());
        }
    }
    let path = update::install(release)?;
    eprintln!("Installed Tersh {} at {}", release.tag(), path.display());
    Ok(())
}

pub(crate) struct StartupCheck {
    pending: Option<Receiver<Option<Release>>>,
    available: Option<Release>,
}

impl StartupCheck {
    pub(crate) fn start() -> Self {
        let enabled = io::stdin().is_terminal()
            && !std::env::var("TERSH_UPDATE_CHECK")
                .is_ok_and(|s| matches!(s.to_ascii_lowercase().as_str(), "off" | "0" | "false"));
        let pending = enabled.then(|| {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                let _ = tx.send(update::check().ok().filter(Release::is_newer));
            });
            rx
        });
        Self {
            pending,
            available: None,
        }
    }

    pub(crate) fn poll(&mut self) -> bool {
        let Some(rx) = &self.pending else {
            return false;
        };
        match rx.try_recv() {
            Ok(release) => self.available = release,
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => {}
        }
        self.pending = None;
        self.available.is_some()
    }

    pub(crate) fn draw_notice(&self, frame: &mut Frame) -> Rect {
        let mut body = frame.area();
        if let Some(release) = &self.available
            && body.height >= 6
        {
            body.height -= 1;
            let text = if body.width < 48 {
                format!("Update {} available | choose on exit", release.tag())
            } else {
                format!(
                    "Update available: {} -> {}  |  Choose whether to install on exit",
                    env!("CARGO_PKG_VERSION"),
                    release.tag()
                )
            };
            frame.render_widget(
                Line::styled(
                    text,
                    crate::theme::Theme::current().bold(crate::theme::Tone::Accent),
                ),
                Rect::new(body.x, body.bottom(), body.width, 1),
            );
        }
        body
    }

    pub(crate) fn offer_on_exit(mut self) {
        // A quick visit can end before the first poll. Finish the bounded
        // metadata request with the terminal restored, then offer the update.
        if let Some(pending) = self.pending.take() {
            self.available = pending
                .recv_timeout(std::time::Duration::from_secs(4))
                .ok()
                .flatten();
        }
        if let Some(release) = self.available
            && let Err(error) = install_if_confirmed(&release, false)
        {
            eprintln!("Tersh update: {error:#}");
        }
    }
}
