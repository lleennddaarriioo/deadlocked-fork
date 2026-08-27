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

radar() {
    cd radar/client
    npm run build
    cd ../../
    mkdir -p radar/server/assets
    cp -r radar/client/dist/* radar/server/assets/
    cd radar/server
    cargo run --bin server --release &
    cloudflared tunnel --url http://127.0.0.1:6346 > /tmp/cloudflared.log 2>&1 &
    cd ../../
}

cheat() {
    cargo run --bin deadlocked -- "$@"
}

bind_keys() {
    # 1. Edit the global CS2 cfg as a fallback (autoexec.cfg)
    local cfg_dir="$HOME/.local/share/Steam/steamapps/common/Counter-Strike Global Offensive/game/csgo/cfg"
    if [[ -d "$cfg_dir" ]]; then
        echo 'bind "space" "+lookatweapon"' >> "$cfg_dir/autoexec.cfg"
        echo 'bind "end" "+jump"' >> "$cfg_dir/autoexec.cfg"
        echo "Appended binds to autoexec.cfg"
    fi

    # 2. Modify user-specific VCFG configuration files for all accounts
    local count=0
    for vcfg in $(find "$HOME/.steam/steam/userdata" -name "cs2_user_keys_*.vcfg" 2>/dev/null); do
        if [[ -f "$vcfg" ]]; then
            python3 -c "
import sys
content = open('$vcfg', 'r').read()
if '\"bindings\"' in content:
    idx = content.find('\"bindings\"')
    brace_open = content.find('{', idx)
    brace_count = 1
    i = brace_open + 1
    while i < len(content) and brace_count > 0:
        if content[i] == '{': brace_count += 1
        elif content[i] == '}': brace_count -= 1
        i += 1
    brace_close = i - 1
    
    # Replace SPACE binding to +lookatweapon so it is tracked but doesn't jump directly
    if '\"SPACE\"' in content:
        lines = content.split('\n')
        for j, line in enumerate(lines):
            if '\"SPACE\"' in line:
                lines[j] = '\t\t\"SPACE\"\t\t\"+lookatweapon\"'
        content = '\n'.join(lines)
    else:
        content = content[:brace_close] + '\t\t\"SPACE\"\t\t\"+lookatweapon\"\n' + content[brace_close:]
        # re-calculate brace_close because we inserted text
        idx = content.find('\"bindings\"')
        brace_open = content.find('{', idx)
        brace_count = 1
        i = brace_open + 1
        while i < len(content) and brace_count > 0:
            if content[i] == '{': brace_count += 1
            elif content[i] == '}': brace_count -= 1
            i += 1
        brace_close = i - 1

    # Ensure END is bound to +jump
    if '\"END\"' not in content:
        content = content[:brace_close] + '\t\t\"END\"\t\t\"+jump\"\n' + content[brace_close:]
        
    open('$vcfg', 'w').write(content)
"
            ((count++))
        fi
    done
    echo "Successfully updated $count user profile config(s) (SPACE -> +lookatweapon, END -> +jump)."
}

# git pull

if [[ $1 == "bind" ]]; then
    bind_keys
elif [[ $1 == "radar" ]]; then
    radar
    cheat "$@"
else
    cheat "$@"
fi
