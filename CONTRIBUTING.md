# 🤝 Contributing to TokenGrid

Thank you for your interest in contributing to TokenGrid! Whether it is bug fixes, performance optimizations for the TUI binary, or workflow enhancements, contributions are welcome.

## Development Workflow

1. **Fork & Clone**: Fork the repository and clone your fork locally.
2. **Branching**: Create a descriptive feature or bugfix branch (```git checkout -b feature/amazing-feature```).
3. **Testing Locally**:
   - Sync changes to your Omarchy plugins directory: ```cp -a . ~/.config/omarchy/plugins/io.github.cybercoretech.tokengrid```
   - Rescan plugins and reload compositor: ```omarchy-shell shell rescanPlugins && hyprctl reload```
4. **Submitting**: Push your branch and open a Pull Request using our provided templates.
