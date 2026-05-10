<div align="center">

# HyprClip

**A clipboard history manager for Hyprland**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-1.0.0-green.svg)](https://github.com/SterTheStar/hyprclip/releases)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GTK4-libadwaita-blueviolet.svg)](https://gtk.org/)
[![Hyprland](https://img.shields.io/badge/Hyprland-Wayland-blue.svg)](https://hyprland.org/)

A fast, minimal clipboard history popup for **Hyprland** using GTK4, libadwaita, and the wlr-layer-shell protocol.

---

## Installation

### From releases

Download the latest package from [Releases](https://github.com/SterTheStar/hyprclip/releases):

```bash
# Debian/Ubuntu
sudo dpkg -i hyprclip_*_amd64.deb

# Fedora
sudo rpm -i hyprclip-*-1.x86_64.rpm

# Binary
tar -xf hyprclip-*-x86_64.tar.xz
sudo cp hyprclip /usr/bin/
```

### Build from source

```bash
# Dependencies (Debian/Ubuntu)
sudo apt install libgtk-4-dev libadwaita-1-dev libgtk4-layer-shell-dev

# Dependencies (Arch)
sudo pacman -S gtk4 libadwaita gtk4-layer-shell

# Build
git clone https://github.com/SterTheStar/hyprclip.git
cd hyprclip
cargo build --release
sudo cp target/release/hyprclip /usr/bin/
```

### Build packages

```bash
./build.sh
# Outputs: dist/*.deb, dist/*.rpm, dist/*.tar.xz
```

## Usage

```bash
# Start background clipboard monitor
hyprclip

# Open the popup GUI
hyprclip --gui

# Show version
hyprclip --version
```

## Hyprland Configuration

Add the following to your `~/.config/hypr/hyprland.conf`:

```ini
# ── Clipboard Manager ──
exec-once = hyprclip
bind = SUPER, V, exec, hyprclip --gui
```

## How it works

1. `hyprclip` starts as a **background service** monitoring the clipboard
2. Press **Super+V** (or your custom keybind) to open the popup
3. **Search** through your clipboard history
4. **Double-click** an entry to copy it and close the popup
5. **Escape** or click outside to close
6. The popup closes automatically after 2 seconds of focus loss

## Tech Stack

- **Rust** 2021 edition
- **GTK4** + **libadwaita** for UI
- **gtk4-layer-shell** for Wayland layer-shell protocol
- **wlr-layer-shell** for always-on-top popup

## License

This project is licensed under the **GNU General Public License v3.0** - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**[Report Bug](https://github.com/SterTheStar/hyprclip/issues)** · **[Request Feature](https://github.com/SterTheStar/hyprclip/issues)**

Made with ❤️ by Esther

</div>
