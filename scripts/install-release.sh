#!/usr/bin/env sh
# Install an official stable Tersh release, or let an existing updater upgrade it.
set -eu
[ "$#" -eq 0 ] || { echo 'Usage: install-release.sh (optional: TERSH_INSTALL_DIR)' >&2; exit 2; }

if [ -n "${TERSH_INSTALL_DIR:-}" ]; then
    installed=$TERSH_INSTALL_DIR/tersh
else
    installed=$(command -v tersh || true)
fi
if [ -f "$installed" ] && [ -x "$installed" ]; then
    case "$("$installed" --help)" in
        *--check-update*) exec "$installed" --update --yes ;;
    esac
fi

case "$(uname -s)/$(uname -m)" in
    Darwin/arm64) platform=macos-arm64 ;;
    Linux/x86_64) platform=linux-x86_64 ;;
    Linux/aarch64|Linux/arm64) platform=linux-arm64 ;;
    *) echo 'Supported releases: macOS arm64, Linux/WSL x86_64 and arm64.' >&2; exit 1 ;;
esac

work=$(mktemp -d)
trap 'rm -rf "$work"' 0
trap 'exit 1' HUP INT TERM
repo=https://github.com/QiushanHuang/Tersh/releases
github() {
    curl -q --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
        --connect-timeout 5 --max-time 90 --max-redirs 5 "$@"
}
effective=$(github --head --output /dev/null --write-out '%{url_effective}' "$repo/latest")
case "$effective" in
    "$repo/tag/"*) tag=${effective#"$repo/tag/"} ;;
    *) echo "Could not resolve the official latest release." >&2; exit 1 ;;
esac
if ! printf '%s\n' "$tag" | LC_ALL=C grep -Eq '^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'; then
    echo "Latest release does not have a stable Tersh version tag." >&2
    exit 1
fi

if [ -f "$installed" ] && [ -x "$installed" ]; then
    current=$("$installed" --version)
    # This branch migrates pre-updater installations. All subsequent upgrades
    # use the executable's semantic-version check and locked replacement.
    action=$(awk -v current="$current" -v next_version="${tag#v}" 'BEGIN {
        if (current !~ /^tersh [0-9]+[.][0-9]+[.][0-9]+([-+][0-9A-Za-z.+-]+)?$/) { print "invalid"; exit }
        sub(/^tersh /, "", current)
        prerelease=(current ~ /^[0-9.]+-/)
        sub(/[-+].*$/, "", current)
        split(current,a,"[.]"); split(next_version,b,"[.]")
        for (i=1;i<=3;i++) if (a[i]+0 != b[i]+0) {
            print (a[i]+0 < b[i]+0 ? "upgrade" : "retain"); exit
        }
        print (prerelease ? "upgrade" : "retain")
    }')
    case "$action" in
        retain) echo "Retaining $installed ($current); latest stable is $tag."; exit 0 ;;
        upgrade) ;;
        *) echo "Cannot verify version of existing $installed." >&2; exit 1 ;;
    esac
fi

asset=tersh-$tag-$platform.tar.gz
github --max-filesize 4096 --output "$work/checksum" "$repo/download/$tag/$asset.sha256"
digest=$(awk -v name="$asset" 'NR == 1 && NF == 2 && length($1) == 64 && $1 !~ /[^0-9a-fA-F]/ && ($2 == name || $2 == "*" name) { h=tolower($1) } END { if (NR != 1 || !h) exit 1; print h }' "$work/checksum")
github --max-filesize 33554432 --output "$work/archive" "$repo/download/$tag/$asset"
if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum <"$work/archive")
else
    actual=$(shasum -a 256 <"$work/archive")
fi
[ "${actual%% *}" = "$digest" ] || { echo 'Release archive checksum mismatch.' >&2; exit 1; }

tar -xzOf "$work/archive" tersh >"$work/tersh"
chmod 755 "$work/tersh"
[ "$("$work/tersh" --version)" = "tersh ${tag#v}" ] || { echo 'Release executable version mismatch.' >&2; exit 1; }
tar -xzOf "$work/archive" install.sh >"$work/install.sh"
sh "$work/install.sh" "$work/tersh"
