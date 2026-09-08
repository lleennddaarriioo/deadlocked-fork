<div align="center">

# deadlocked

[![Matrix Invite](https://img.shields.io/matrix/open-source-cs2-hacking%3Amatrix.org?style=for-the-badge\&logo=matrix\&label=Matrix)](https://matrix.to/#/%23open-source-cs2-hacking:matrix.org)
[![Discord Invite](https://img.shields.io/discord/1333541580249890949?style=for-the-badge\&logo=discord\&logoColor=white\&label=Discord)](https://discord.gg/eXjG4Ar9Sx)

[![Casual Maintenance Intended](https://casuallymaintained.tech/badge.svg)](https://casuallymaintained.tech/)

<br>

simple cs2 aimbot and esp, for linux only.

<br>

Releases are tagged `v<version>` matching the version in `Cargo.toml` (e.g. `v1.0.0`).
The built-in update checker compares against the latest release tag and will prompt when a newer version is available.

</div>

<br>

## Quick Start

> [!NOTE]
> Running NixOS, Fedora Atomic, Hyprland (Legacy .conf config)?
>
> See the [compatibility.md](compatibility.md).

Download the [latest release](https://github.com/avitran0/deadlocked/releases). Each release contains the `deadlocked` binary and `setup.sh`.

**Setup (one-time only):**

```bash
./setup.sh
```

> **Restart your machine (required)**

This creates a `uinput` group, adds your user to it, and installs a udev rule.
You only need to do this once, even when updating to newer versions.

**Run:**

```bash
./deadlocked
```

The binary will refuse to start if setup hasn't been completed.
Also make sure the `uinput` kernel module is loaded.

<br>

## Build from Source

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/lleennddaarriioo/deadlocked-fork
cd deadlocked
cargo run --release
```

<br>

## Running

```bash
./run.sh
```

<br>

## Features

### Aimbot

- Hotkey
- FOV (Static & Distance-Adjusted)
- Smooth & Inertia Control
- Start bullet
- Targeting mode
- Visibility check (BVH VPK parsing, Bone LoS, Bone Fast)
- Bone selection & Bone mode (Nearest / Priority)
- Flash check
- FOV circle
- Grenade lineup auto-aim

### ESP

- Hotkey
- Box
- Skeleton
- Health bar
- Armor bar
- Player name
- Weapon icon
- Player tags (helmet, defuser, bomb)
- Dropped weapons
- Bomb timer
- 3D Sound & Footstep ESP
- Offscreen enemy indicators (OOF arrows)
- Animated floating damage text

### Hitsounds & Audio

- Custom audio hitsounds & kill sound engine (Rust Headshot, COD Hitmarker, Metallic Bell, CSGO Ding, Bubble, Neverlose NL, Skeet GS, Aimware MS, Primordial, Custom WAV)
- Volume & pitch controls with live GUI preview on change
- Only local player shot filter & 1-tap fatal kill filter

### Triggerbot

- Activation mode
- Min/max delay
- Additional Duration
- Visibility check
- Flash check
- Scope check
- Velocity threshold
- Head only mode
- Aimbot-locked triggerbot threshold

### Bunnyhop & Movement

- Auto Bunnyhop
- Auto Strafe (Air acceleration optimization)
- Edge Jump

### Web Radar

- WebSockets-based real-time 2D web radar dashboard ([FAQ](radar.md))

### Standalone RCS

- Smoothing

### Per-Weapon Overrides

- Aimbot
- Triggerbot
- RCS

### Unsafe

> [!WARNING]
> These features write to game memory and might get you banned.

- No flash (with max flash alpha)
- FOV changer
- No smoke
- Smoke color change

<br>

## FAQ

<details>
<summary>Where are my configs saved?</summary>

Configs are saved in `$XDG_CONFIG_HOME` with fallback to `$HOME/.config`. Otherwise they're saved alongside the executable.

</details>

<br>

<details>
<summary>Which desktop environments and window managers are supported?</summary>

**Best support:**

- GNOME (Mutter)
- KDE (KWin)

**Good support:**

- SwayWM
- Weston

**Fair support:**

- i3
- OpenBox
- XFCE
- Hyprland (tweaks may be needed, no guarantees; see [compatibility.md](compatibility.md/#hyprland))

</details>

<br>

<details>
<summary>Hyprland Window Rules</summary>

For `hyprland.conf`:
```ini
windowrule = float 1, match:title ^(deadlocked_overlay)$
windowrule = no_focus 1, match:title ^(deadlocked_overlay)$
windowrule = pin 1, match:title ^(deadlocked_overlay)$
windowrule = no_blur 1, match:title ^(deadlocked_overlay)$
windowrule = no_anim 1, match:title ^(deadlocked_overlay)$
windowrule = no_shadow 1, match:title ^(deadlocked_overlay)$
windowrule = no_blur 1, match:class ^(deadlocked)$
```

</details>
