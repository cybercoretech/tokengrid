# 🪙⚡ TokenGrid

A high-performance TUI-based telemetry and monitoring plugin built for **Omarchy**, running natively inside **Ghostty** and managed via **Hyprland v0.55+** Lua window rules.

---

## ✨ Features

- **🚀 Native TUI Execution**: Launches instantly via an optimized Ghostty terminal instance.
- **🖼️ Precision Floating Layout**: Automatically floats, centers, and sizes to precisely `1100x650` using modern Hyprland Lua window rules (`hl.window_rule`).
- **🎛️ Omarchy Bar Integration**: Seamlessly hooks into the Omarchy panel via a custom QML bar widget (`BarWidget.qml`).
- **🔒 Process Isolation**: Uses a dedicated Ghostty configuration profile (`gtk-single-instance = false`) to enforce unique window title matching (`TokenGrid`).

---

## 🛠️ System Requirements

- **Compositor**: Hyprland v0.55+ (Lua configuration architecture)
- **Terminal**: Ghostty
- **Shell Environment**: Zsh / Omarchy Shell
- **Plugin Framework**: Omarchy Plugins Manager

---

## 📂 Project Structure

```text
tokengrid/
├── BarWidget.qml          # Omarchy panel trigger widget
├── manifest.json          # Omarchy plugin manifest
├── bin/                   # Compiled TUI binary (tokengrid-tui)
├── config/
│   └── tokengrid.ghostty  # Dedicated Ghostty profile (title matching)
└── helpers/
    └── launch-tokengrid.sh # Execution wrapper script
```

---

## 📥 Installation & Setup

### 1. Clone or Copy to Omarchy Plugins
Place the repository inside your Omarchy plugins directory:
```zsh
git clone https://github.com/cybercoretech/tokengrid.git ~/.config/omarchy/plugins/io.github.cybercoretech.tokengrid
```

### 2. Configure Hyprland Window Rules
Add the native Lua window rule to your Hyprland configuration (`~/.config/hypr/looknfeel.lua`):
```lua
-- TokenGrid TUI Monitor Floating Rule
hl.window_rule({ 
    match = { title = "TokenGrid" }, 
    float = true, 
    size = "1100 650", 
    center = true 
})
```

### 3. Rescan & Restart Omarchy
Refresh your plugin registry and restart the shell to register the bar widget:
```zsh
omarchy-shell shell rescanPlugins
omarchy restart shell
hyprctl reload
```

---

## 🚀 Usage

Click the **TokenGrid** icon in your Omarchy status bar, or trigger the helper script directly from your terminal:

```zsh
~/.config/omarchy/plugins/io.github.cybercoretech.tokengrid/helpers/launch-tokengrid.sh
```

---

## 📜 License

Distributed under the MIT License. See `LICENSE` for more information.
