#!/usr/bin/env bash

UDEV_RULE_FILE="/etc/udev/rules.d/99-uinput.rules"
UINPUT_GROUP="uinput"
CURRENT_USER=$(whoami)

git config core.hooksPath .hooks

echo 'KERNEL=="uinput", MODE="0660", GROUP="uinput"' | sudo tee "$UDEV_RULE_FILE" > /dev/null
echo "created udev file: $UDEV_RULE_FILE"

if ! getent group "$UINPUT_GROUP" > /dev/null; then
    sudo groupadd "$UINPUT_GROUP"
    echo "created group $UINPUT_GROUP"
fi

sudo usermod -aG "$UINPUT_GROUP" "$CURRENT_USER"
echo "added user $CURRENT_USER to group $UINPUT_GROUP"

sudo udevadm control --reload-rules
sudo udevadm trigger
echo "reloaded udev rules"

if [ "$XDG_CURRENT_DESKTOP" = "Hyprland" ]; then
    echo "detected Hyprland as window manager"

    CONF_FILE="$HOME/.config/hypr/hyprland.conf"
    LUA_FILE="$HOME/.config/hypr/hyprland.lua"

    REPLY="Y"
    if [ -t 0 ]; then
        read -p "Do you want to add Hyprland overlay window rules to your config? [Y/n] " INPUT_REPLY
        REPLY=${INPUT_REPLY:-Y}
    fi

    if [[ "$REPLY" =~ ^[Yy]$ ]]; then
        if [ -f "$LUA_FILE" ]; then
            if grep -q "deadlocked_overlay" "$LUA_FILE"; then
                echo "deadlocked_overlay windowrules already present in hyprland.lua, skipping"
            else
                cat << 'EOF' >> "$LUA_FILE"

-- Deadlocked overlay rules
hl.window_rule({ match = { title = "^(deadlocked_overlay)$" }, float = 1 })
hl.window_rule({ match = { title = "^(deadlocked_overlay)$" }, no_focus = 1 })
hl.window_rule({ match = { title = "^(deadlocked_overlay)$" }, pin = 1 })
hl.window_rule({ match = { title = "^(deadlocked_overlay)$" }, no_blur = 1 })
hl.window_rule({ match = { title = "^(deadlocked_overlay)$" }, no_anim = 1 })
hl.window_rule({ match = { title = "^(deadlocked_overlay)$" }, no_shadow = 1 })
hl.window_rule({ match = { class = "^(deadlocked)$" }, no_blur = 1 })
EOF
                echo "added windowrules to hyprland.lua"
            fi
        else
            mkdir -p "$(dirname "$CONF_FILE")"
            if grep -q "deadlocked_overlay" "$CONF_FILE" 2>/dev/null; then
                echo "deadlocked_overlay windowrules already present in hyprland.conf, skipping"
            else
                cat << 'EOF' >> "$CONF_FILE"

# Deadlocked overlay rules
windowrule = float 1, match:title ^(deadlocked_overlay)$
windowrule = no_focus 1, match:title ^(deadlocked_overlay)$
windowrule = pin 1, match:title ^(deadlocked_overlay)$
windowrule = no_blur 1, match:title ^(deadlocked_overlay)$
windowrule = no_anim 1, match:title ^(deadlocked_overlay)$
windowrule = no_shadow 1, match:title ^(deadlocked_overlay)$
windowrule = no_blur 1, match:class ^(deadlocked)$
EOF
                echo "added windowrules to hyprland.conf"
            fi
        fi
    else
        echo "Skipping Hyprland configuration changes."
    fi
fi
