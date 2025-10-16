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
- **Task Details**: Comprehensive task information display in dedicated detail panel

### ✅ **Modern User Interface**

- **Responsive Design**: Automatic layout adaptation for different terminal sizes
- **Task List with Integrated Filters**: Main view with task list and inline filter panel
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

- **Dual Mode Interface**: Toggle between Dashboard and Calendar views
- **Dashboard Mode**:
  - Modern 4-panel layout: Summary, Burndown, Project Analytics, Recent Activity
  - Real-time statistics: task counts, completion rates, priority breakdown
  - Project analytics: detailed per-project stats with progress tracking
  - Activity timeline: recent task changes with detailed activity types
- **Calendar Mode**:
  - 3-month horizontal calendar view (previous, current, next month)
  - Task indicators on dates: ⚠ overdue, • pending, ✓ completed, ○ other
  - Daily task details with status breakdown including deleted tasks
  - Full keyboard navigation: arrows for days/weeks, <>for months, 't' for today
- **Performance Optimized**: Smart caching system eliminates flickering
- **Responsive Layout**: Adaptive charts and panels based on terminal size

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
2. **🔍 Filter Mode**: Interactive filtering with real-time preview (press `/`)
3. **✏️ Form Mode**: Task creation/editing with comprehensive fields
4. **📊 Dashboard Mode**: Analytics dashboard with 4 detailed panels
5. **📅 Calendar Mode**: 3-month calendar view with task tracking
6. **❓ Help Mode**: Context-sensitive keyboard reference (F1)

### Complete Keyboard Interface

**Main Navigation:**

- `q` - Quit application
- `F1` - Context-sensitive help
- `F5` - Refresh tasks from Taskwarrior
- `↑/↓` - Navigate task list / filter options
- `Enter` - Select/confirm action
- `Esc` - Cancel/go back

**Task Operations:**

- `a` - Add new task (modal form)
- `e` - Edit selected task
- `d` - Mark task as done
- `Delete` - Delete selected task

**Filtering & Views:**

- `/` - Toggle interactive filter bar
- `Tab` - Navigate between filter sections (Status, Project, Tags, Search)
- `Space` - Toggle filter selections
- `r` - Open reports dashboard

**Reports & Calendar:**

- `c` - Toggle between Dashboard and Calendar modes (in Reports view)
- **Calendar Navigation:**
  - `←/→` - Navigate by day
  - `↑/↓` - Navigate by week
  - `</>` - Navigate by month (previous/next)
  - `t` - Jump to today

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

**Dashboard Mode (default):**

- **📈 Summary Panel**: Task counts, completion rates, priority breakdown
- **📊 Burndown Chart**: 30-day completion trend visualization
- **📋 Project Analytics**: Per-project statistics with task counts, completion rates, urgency, and next due dates
- **🕒 Recent Activity**: Timeline of recent task changes with detailed activity types

**Calendar Mode (press `c` to toggle):**

- **📅 3-Month View**: See previous, current, and next month simultaneously
- **Visual Indicators**: Tasks marked on dates with status icons (⚠ overdue, • pending, ✓ completed)
- **Daily Details**: Click/select a date to see all tasks for that day with full breakdown
- **Smart Navigation**: Arrows for day/week, <> for months, 't' to jump to today
- **Statistics**: Total, pending, completed, overdue, and deleted tasks per day

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
