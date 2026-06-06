#!/usr/bin/env sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

choose_install_dir() {
    if [ -n "${TERSH_INSTALL_DIR:-}" ]; then
        printf '%s\n' "$TERSH_INSTALL_DIR"
        return
    fi

    if command -v tersh >/dev/null 2>&1; then
        dirname -- "$(command -v tersh)"
        return
    fi

    for candidate in "$HOME/.local/bin" "$HOME/bin" /opt/homebrew/bin /usr/local/bin; do
        case ":$PATH:" in
            *":$candidate:"*)
                if [ -d "$candidate" ] && [ -w "$candidate" ]; then
                    printf '%s\n' "$candidate"
                    return
                fi
                ;;
        esac
    done

    printf '%s\n' "$HOME/.local/bin"
}

install_dir=$(choose_install_dir)

cd "$project_dir"
cargo build --locked --release --bin tersh

mkdir -p "$install_dir"
if [ ! -w "$install_dir" ]; then
    printf 'Install directory is not writable: %s\n' "$install_dir" >&2
    printf 'Set TERSH_INSTALL_DIR to a writable directory or rerun with appropriate permissions.\n' >&2
    exit 1
fi
tmp_install=$(mktemp "$install_dir/.tersh.XXXXXX")
trap 'rm -f "$tmp_install"' EXIT HUP INT TERM
cp "$project_dir/target/release/tersh" "$tmp_install"
chmod 755 "$tmp_install"
mv -f "$tmp_install" "$install_dir/tersh"
trap - EXIT HUP INT TERM

printf 'Installed Tersh CLI as %s/tersh\n' "$install_dir"
printf 'Optional visual cd helper: source %s/scripts/tersh-cd.sh from your shell profile.\n' "$project_dir"

case ":$PATH:" in
    *":$install_dir:"*) ;;
    *)
        printf 'Add %s to PATH to run tersh from any directory.\n' "$install_dir"
        ;;
esac
