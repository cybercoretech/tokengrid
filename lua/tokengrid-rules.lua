-- TokenGrid Native Lua Window Rule (Hyprland v0.55+ API)
hl.window_rule({
    name = "tokengrid-floating-rule",
    match = {
        initial_title = "TokenGridTelemetry"
    },
    float = true,
    size = "1100 650",
    center = true
})
