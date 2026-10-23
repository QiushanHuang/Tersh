#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
if [ "$#" -gt 1 ]; then
    printf 'Usage: %s [built-tersh-executable]\n' "$0" >&2
    exit 2
fi
binary_path=${1:-"$script_dir/tersh"}
if [ ! -f "$binary_path" ] || [ ! -r "$binary_path" ]; then
    printf 'Built Tersh executable is missing or unreadable: %s\n' "$binary_path" >&2
    exit 1
fi

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
mkdir -p "$install_dir"
if [ ! -w "$install_dir" ]; then
    printf 'Install directory is not writable: %s\n' "$install_dir" >&2
    printf 'Set TERSH_INSTALL_DIR to a writable directory or rerun with appropriate permissions.\n' >&2
    exit 1
fi
install_dir=$(CDPATH= cd -- "$install_dir" && pwd)
installed_path=$install_dir/tersh
if [ -d "$installed_path" ]; then
    printf 'Executable destination is a directory: %s\n' "$installed_path" >&2
    exit 1
fi
tmp_install=$(mktemp "$install_dir/.tersh.XXXXXX")
trap 'rm -f "$tmp_install"' 0
trap 'exit 1' HUP INT TERM
cp "$binary_path" "$tmp_install"
chmod 755 "$tmp_install"
mv -f "$tmp_install" "$installed_path"
trap - 0 HUP INT TERM

printf 'Installed Tersh CLI as %s\n' "$installed_path"
if "$installed_path" --install-patches; then
    printf 'Tersh installation and applicable lifecycle patches completed.\n'
else
    patch_status=$?
    printf 'Tersh binary is installed, but the WSL lifecycle patch is incomplete.\n' >&2
    printf 'Resolve the error above, then rerun: "%s" --install-patches\n' "$installed_path" >&2
    exit "$patch_status"
fi

case ":$PATH:" in
    *":$install_dir:"*) ;;
    *)
        printf 'Add %s to PATH to run tersh from any directory.\n' "$install_dir"
        ;;
esac
