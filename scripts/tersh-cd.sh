tersh-cd() {
    if ! command -v tersh >/dev/null 2>&1; then
        printf 'tersh: command not found\n' >&2
        return 127
    fi

    target_dir=$(tersh --print-cwd "$@") || return $?
    if [ -n "$target_dir" ]; then
        cd -- "$target_dir" || return $?
    fi
}

alias tcd=tersh-cd
