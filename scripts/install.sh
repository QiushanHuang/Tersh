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
cargo build --release --bin tersh

mkdir -p "$install_dir"
cp "$project_dir/target/release/tersh" "$install_dir/tersh"
chmod 755 "$install_dir/tersh"

printf 'Installed Tersh CLI as %s/tersh\n' "$install_dir"

case ":$PATH:" in
    *":$install_dir:"*) ;;
    *)
        printf 'Add %s to PATH to run tersh from any directory.\n' "$install_dir"
        ;;
esac
