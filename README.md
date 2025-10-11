# LazyTask - Modern Terminal UI for Taskwarrior

A modern, responsive Terminal User Interface (TUI) for Taskwarrior, built with Rust and Ratatui. LazyTask provides an intuitive, keyboard-driven interface similar to popular TUIs like Lazygit and Yazi.

## Features

🚧 **Currently in Development** 🚧

### Planned Features

- **Complete Taskwarrior Integration**: Full support for all Taskwarrior commands and reports
- **Modern TUI Design**: Clean, responsive interface with multiple view modes
- **Advanced Filtering**: Interactive filter builder with real-time preview
- **Multiple Data Sources**: Direct SQLite access + CLI integration + JSON export/import
- **Customizable Interface**: Configurable themes, keybindings, and layouts
- **Contextual Workflows**: Smart context switching and task organization
- **Comprehensive Reports**: Calendar view, statistics, project summaries, and more

### Current Status

✅ **Completed:**

- Rust development environment setup
- Basic Ratatui application with event loop and terminal management
- TaskChampion SQLite database access and CLI wrapper foundation
- Configuration management with TOML support and theme system
- Core UI widgets: task list, status bar, and navigation framework

🔄 **In Progress:**

- Core UI widgets implementation

⏳ **Planned:**

- Full task management (CRUD operations)
- Interactive filter engine with real-time preview
- Split-panel interface (list view + detail view)
- Reports interface (calendar, statistics, project summaries)
- Configurable keyboard shortcuts and input handling
- Taskwarrior sync support with progress indicators
- Comprehensive test suite
- Performance optimization
- Documentation and distribution packages

## Requirements

- Rust 1.90.0 or later
- Taskwarrior 3.0+ (TaskChampion backend)
- A terminal that supports modern TUI applications

## Installation

### From Source

```bash
git clone https://github.com/osamamahmood/lazytask
cd lazytask
cargo build --release
```

The compiled binary will be available at `target/release/lazytask`.

## Usage

```bash
# Basic usage
lazytask

# Specify custom config
lazytask --config /path/to/config.toml

# Verbose output
lazytask --verbose
```

### Keybindings

Default keybindings (configurable):

- `q` - Quit application
- `F1` - Show help
- `F5` - Refresh
- `a` - Add new task
- `e` - Edit selected task
- `d` - Mark task as done
- `Delete` - Delete selected task
- `↑/↓/←/→` - Navigate
- `Enter` - Select/Open
- `Esc` - Go back
- `/` - Filter tasks
- `c` - Change context
- `r` - View reports
- `Ctrl+C` - Force quit

## Architecture

LazyTask uses a modular architecture with clean separation of concerns:

```
src/
├── app.rs              # Main application coordination
├── config.rs           # Configuration management
├── taskwarrior.rs      # Taskwarrior integration layer
├── ui/                 # User interface components
│   ├── components/     # Reusable UI widgets
│   ├── views/          # Screen layouts
│   └── themes.rs       # Color schemes and styling
├── handlers/           # Event and command processing
├── data/               # Data models and persistence
└── utils/              # Common utilities
```

### Data Integration Strategy

LazyTask uses a triple integration approach for maximum performance and compatibility:

1. **Direct SQLite Access**: Query TaskChampion database for performance-critical operations
2. **CLI Command Interface**: Execute `task` commands for complex operations
3. **JSON Export/Import**: Bulk data operations and compatibility

## Configuration

LazyTask stores its configuration in `~/.config/lazytask/config.toml`:

```toml
[theme]
name = "catppuccin-mocha"

[ui]
default_view = "task_list"
show_help_bar = true
task_list_columns = ["id", "project", "priority", "due", "description"]

[taskwarrior]
sync_enabled = false

[keybindings.global]
quit = "q"
help = "F1"
refresh = "F5"

[keybindings.task_list]
add_task = "a"
edit_task = "e"
done_task = "d"
delete_task = "Delete"
```

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/osamamahmood/lazytask
cd lazytask

# Build in development mode
cargo build

# Run with debug output
cargo run -- --verbose

# Run tests
cargo test

# Build optimized release
cargo build --release
```

### Project Structure

The codebase follows Rust best practices with a clean modular design:

- **Separation of Concerns**: UI, data, and business logic are clearly separated
- **Async/Await**: Full async support for responsive UI
- **Error Handling**: Comprehensive error handling with `anyhow`
- **Configuration**: TOML-based configuration with sane defaults
- **Testing**: Unit and integration tests for reliability

## Contributing

Contributions are welcome! Please see the implementation plan for current priorities.

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- [Taskwarrior](https://taskwarrior.org/) - The powerful CLI task manager
- [Ratatui](https://ratatui.rs/) - Rust library for building rich TUIs
- [Lazygit](https://github.com/jesseduffield/lazygit) - Inspiration for TUI design patterns
- [Yazi](https://yazi-rs.github.io/) - Modern TUI file manager inspiration
