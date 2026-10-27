# 🏛️ TokenGrid Architecture

TokenGrid is engineered as a decoupled, high-performance telemetry plugin for **Omarchy**, bridging modern Wayland compositing, terminal process isolation, and a native TUI binary.

---

## 🔄 Execution & Lifecycle Flow

1. **Bar Trigger (`BarWidget.qml`)**: 
   - The user interacts with the Omarchy panel icon.
   - Triggers the execution wrapper script located at ```helpers/launch-tokengrid.sh```.

2. **Process Isolation (`config/tokengrid.ghostty`)**:
   - Ghostty is invoked with ```--config-file``` pointing to a dedicated configuration profile.
   - Enforces ```gtk-single-instance = false``` to prevent background daemon multiplexing, which would otherwise strip CLI flags and drop custom class/title parameters.
   - Sets a hardcoded window title: ```title = TokenGrid```.

3. **Compositor Rule Routing (`looknfeel.lua`)**:
   - Hyprland v0.55+ evaluates the active window creation event via its native Lua IPC rule engine.
   - A dedicated rule (```hl.window_rule```) matches the window title ```TokenGrid``` and instantly applies positioning parameters: ```float = true```, ```size = "1100 650"```, and ```center = true```.

4. **TUI Telemetry Execution**:
   - The isolated Ghostty terminal instance spawns the compiled binary (```bin/tokengrid-tui```), rendering system metrics inside the precision floating container.
