#!/usr/bin/env bash

fail() {
	echo "Not a git repository (or any of the parent directories): .git.
Do NOT download the repository as a zip file from github.com!
Please download deadlocked by cloning the Git repository: 'git clone https://github.com/avitran0/deadlocked'"
	exit 1
}

[[ -d '.git' ]] || fail

# Auto-detect Wayland and X11 display environment variables for clipboard GUI fallback compatibility
if [[ -z "$WAYLAND_DISPLAY" ]]; then
    for socket in /run/user/$(id -u)/wayland-*; do
        if [[ -S "$socket" ]]; then
            export WAYLAND_DISPLAY="${socket##*/}"
            echo "Auto-detected WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
            break
        fi
    done
fi

if [[ -z "$DISPLAY" ]]; then
    for socket in /tmp/.X11-unix/X*; do
        if [[ -S "$socket" ]]; then
            export DISPLAY=":${socket##*/X}"
            echo "Auto-detected DISPLAY=$DISPLAY"
            break
        fi
    done
fi

if [[ -z "$XAUTHORITY" && -f "$HOME/.Xauthority" ]]; then
    export XAUTHORITY="$HOME/.Xauthority"
fi

cheat() {
    cargo run --release --bin deadlocked -- "$@"
}

cheat "$@"
