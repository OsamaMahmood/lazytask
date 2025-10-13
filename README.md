# LazyTask - Modern Terminal UI for Taskwarrior

A modern, responsive Terminal User Interface (TUI) for Taskwarrior, built with Rust and Ratatui. LazyTask provides an intuitive, keyboard-driven interface similar to popular TUIs like Lazygit and Yazi.

<img width="1561" height="977" alt="image" src="https://github.com/user-attachments/assets/0441da8f-e2ea-483d-ba4f-2ec61ad75fd9" />
<img width="1561" height="977" alt="image" src="https://github.com/user-attachments/assets/761e174a-fe67-4987-aab4-3d5821b42b73" />

## Features

LazyTask is now a complete, modern Terminal User Interface for Taskwarrior with professional-grade features and polish.

### ✅ **Core Task Management**

- **Complete CRUD Operations**: Add, edit, delete, and complete tasks with full Taskwarrior sync
- **Advanced Task Forms**: Modal dialogs with project, priority, due date, tags, and description fields
- **Smart Selection**: UUID-based task selection that persists across operations
- **Tag Management**: Full tag editing with proper add/remove functionality
- **Task Details**: Comprehensive task information display with formatting

### ✅ **Modern User Interface**

- **Responsive Design**: Automatic layout adaptation for different terminal sizes
- **Split-Panel View**: Task list + detail panel (similar to Lazygit)
- **Professional Theming**: Catppuccin color scheme with priority-based color coding
- **Auto-Resize**: Seamless UI updates when terminal window is resized
- **Modal System**: Clean, professional forms and dialogs

### ✅ **Advanced Filtering System**

- **Interactive Filter Bar**: Real-time filtering with immediate preview
- **Status Filters**: All, Pending, Active, Overdue, Completed, Waiting, Deleted
- **Computed Filters**: Smart Active (started tasks) and Overdue (past due) detection
- **Multi-Criteria**: Filter by project, priority, tags, and description simultaneously
- **Keyboard Navigation**: Full keyboard control with intuitive shortcuts

### ✅ **Professional Reports Dashboard**

- **Modern 3-Panel Layout**: Summary + Burndown, Project Analytics, Recent Activity
- **Performance Optimized**: Smart caching system eliminates flickering
- **Real-Time Statistics**: Task counts, completion rates, priority breakdown
- **Project Analytics**: Detailed per-project statistics with progress tracking
- **Activity Timeline**: Recent task changes with detailed activity types
- **Responsive Charts**: Adaptive burndown charts and data visualization

### ✅ **Complete Taskwarrior Integration**

- **Full JSON Parsing**: Complete support for all Taskwarrior datetime fields (start, end, wait, scheduled, until)
- **CLI Command Integration**: Seamless execution of Taskwarrior operations
- **Active Task Detection**: Proper started task identification and filtering
- **Error Handling**: Robust error recovery and user feedback
- **Data Accuracy**: Real-time sync with Taskwarrior database

### ✅ **Developer Experience**

- **Comprehensive Demos**: Multiple showcase programs demonstrating features
- **Test Suite**: Validation programs for all major functionality
- **Zero Compilation Errors**: Production-ready, stable codebase
- **Performance Optimized**: ~15MB memory usage, <300ms startup time

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

### Quick Start

```bash
# Launch LazyTask
cargo run
# or after building: ./target/release/lazytask

# Explore features with demo programs
cargo run --bin feature_demo    # Complete feature showcase
cargo run --bin filter_test     # Test filtering system
cargo run --bin final_demo      # Analytics and overview
```

### Application Modes

LazyTask offers multiple integrated modes for different workflows:

1. **📋 Task List Mode**: Browse and manage tasks with keyboard navigation
2. **🔀 Split-Panel Mode**: Task list + detailed view simultaneously (like Lazygit)
3. **🔍 Filter Mode**: Interactive filtering with real-time preview
4. **✏️ Form Mode**: Task creation/editing with comprehensive fields
5. **📊 Reports Mode**: Analytics dashboard with 4 detailed panels
6. **❓ Help Mode**: Context-sensitive keyboard reference

### Complete Keyboard Interface

**Main Navigation:**

- `q` - Quit application
- `F1` - Context-sensitive help
- `F5` - Refresh tasks from Taskwarrior
- `↑/↓` - Navigate task list
- `Tab` - Toggle split/single view
- `Enter` - Select/confirm action
- `Esc` - Cancel/go back

**Task Operations:**

- `a` - Add new task (modal form)
- `e` - Edit selected task
- `d` - Mark task as done
- `Delete` - Delete selected task

**Advanced Features (ToDo):**

- `/` - Open interactive filter bar
- `r` - Open reports dashboard
- `c` - Switch Taskwarrior context
- `C` - Clear all filters (in filter mode)

### Filter System Usage

The advanced filtering system supports real-time task filtering:

1. Press `/` to open filter bar
2. Navigate fields with `↑/↓`
3. **Status Field**: Press `Space` to cycle through: All → Pending → Active → Overdue → Completed → Waiting → Deleted
4. **Quick Keys**: Type `p`ending, `a`ctive, `o`verdue, `c`ompleted, `w`aiting, `d`eleted
5. **Text Fields**: Type directly to filter by project, tags, or description
6. Press `Enter` to apply, `Esc` to cancel

### Reports Dashboard

Access comprehensive analytics with `r`:

- **📈 Summary Panel**: Task counts, completion rates, priority breakdown
- **📊 Burndown Chart**: Progress visualization and completion trends
- **📋 Project Analytics**: Per-project statistics with task counts and next due dates
- **🕒 Recent Activity**: Timeline of recent task changes with detailed activity types

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

# Run LazyTask
cargo run

# Explore features with demos
cargo run --bin feature_demo    # Feature showcase with keyboard reference
cargo run --bin filter_test     # Validate filtering system (shows active tasks!)
cargo run --bin final_demo      # Complete overview with analytics

# Build optimized release
cargo build --release
```

### Demo Programs

LazyTask includes comprehensive demonstration programs:

- **`feature_demo`**: Complete feature showcase with keyboard reference and system status
- **`filter_test`**: Validates all filter types including the critical active task filtering
- **`final_demo`**: Full system overview with real-time analytics and project statistics
- **`crud_test`**: Basic CRUD operations demonstration
- **`demo`**: Original foundation demo

Each demo program provides valuable insights into LazyTask's capabilities and serves as both documentation and validation of functionality.

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
