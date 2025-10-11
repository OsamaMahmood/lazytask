# LazyTask Development Guide

This guide covers the development workflow, architecture, and contribution guidelines for LazyTask.

## Development Environment Setup

### Prerequisites

- Rust 1.90.0 or later
- Taskwarrior 3.0+ for testing
- Git for version control

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/osamamahmood/lazytask
cd lazytask

# Install dependencies and build
cargo build

# Run in development mode
cargo run

# Run with verbose logging
cargo run -- --verbose

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings
```

## Project Architecture

LazyTask follows a modular architecture with clear separation of concerns:

```
src/
├── main.rs              # Entry point and CLI parsing
├── app.rs               # Main application coordination
├── config.rs            # Configuration management
├── taskwarrior.rs       # Taskwarrior integration layer
│
├── ui/                  # User interface layer
│   ├── mod.rs
│   ├── app_ui.rs        # Main UI coordinator
│   ├── components/      # Reusable UI widgets
│   │   ├── task_list.rs     # Task display widget
│   │   ├── task_detail.rs   # Task edit form
│   │   ├── filter_bar.rs    # Interactive filter builder
│   │   ├── status_bar.rs    # Status and shortcuts
│   │   ├── calendar_view.rs # Calendar widget
│   │   ├── report_panel.rs  # Statistics and charts
│   │   └── modal_dialog.rs  # Confirmation dialogs
│   ├── views/           # Screen layouts
│   │   ├── main_view.rs     # Primary task list
│   │   ├── detail_view.rs   # Task detail/edit
│   │   ├── reports_view.rs  # Reports dashboard
│   │   ├── calendar_view.rs # Calendar interface
│   │   ├── projects_view.rs # Project browser
│   │   └── settings_view.rs # Configuration UI
│   └── themes.rs        # Color schemes
│
├── handlers/            # Event and command processing
│   ├── input.rs         # Key/mouse input processing
│   ├── commands.rs      # Command validation/execution
│   ├── navigation.rs    # View switching logic
│   └── sync.rs          # Background sync operations
│
├── data/                # Data layer
│   ├── models.rs        # Task, Project, Tag structs
│   ├── database.rs      # SQLite TaskChampion access
│   ├── cli_interface.rs # Taskwarrior CLI wrapper
│   ├── filters.rs       # Query engine
│   ├── cache.rs         # Performance caching
│   └── export.rs        # Import/export utilities
│
└── utils/               # Common utilities
    ├── keybindings.rs   # Configurable shortcuts
    ├── formatting.rs    # Date/text formatting
    ├── validation.rs    # Input validation
    └── helpers.rs       # Common utilities
```

### Design Principles

1. **Separation of Concerns**: UI, data, and business logic are clearly separated
2. **Async/Await**: Full async support for responsive UI
3. **Error Handling**: Comprehensive error handling with `anyhow`
4. **Configurability**: Everything should be configurable
5. **Performance**: Optimize for responsiveness and memory usage
6. **Testability**: Code should be easy to unit test

## Key Components

### App Structure

The main `App` struct coordinates all components:

```rust
pub struct App {
    pub config: Config,
    pub terminal: AppTerminal,
    pub ui: AppUI,
    pub input_handler: InputHandler,
    pub should_quit: bool,
}
```

### UI Layer

The UI is built with Ratatui widgets organized into:

- **Components**: Reusable widgets (task lists, forms, dialogs)
- **Views**: Full screen layouts (main view, calendar, reports)
- **Themes**: Color scheme management

### Data Layer

Three-tier data access strategy:

1. **Direct SQLite**: Fast queries via TaskChampion database
2. **CLI Interface**: Execute `task` commands for complex operations
3. **JSON Export/Import**: Bulk operations and compatibility

### Configuration System

TOML-based configuration with:

- Hierarchical settings
- Environment variable support
- Hot reloading (planned)
- Migration support

## Development Workflow

### Code Style

We follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check style
cargo clippy

# Run tests
cargo test

# Check documentation
cargo doc --open
```

### Commit Messages

Use conventional commits:

```
feat: add calendar view navigation
fix: resolve task filtering crash
docs: update configuration guide
refactor: simplify UI event handling
test: add task creation tests
```

### Branch Strategy

- `main`: Stable release branch
- `develop`: Integration branch
- `feature/*`: Feature branches
- `fix/*`: Bug fix branches
- `docs/*`: Documentation branches

### Testing Strategy

#### Unit Tests

Test individual functions and modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test task".to_string());
        assert_eq!(task.description, "Test task");
        assert_eq!(task.status, TaskStatus::Pending);
    }
}
```

#### Integration Tests

Test component interactions:

```rust
#[tokio::test]
async fn test_task_workflow() {
    let mut app = create_test_app().await;

    // Test adding a task
    app.add_task("Test task", &[("project", "test")]).await?;

    // Verify task was added
    let tasks = app.list_tasks(None).await?;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].description, "Test task");
}
```

#### UI Tests

Test UI interactions:

```rust
#[test]
fn test_task_list_navigation() {
    let mut widget = TaskListWidget::new();
    widget.set_tasks(create_test_tasks());

    // Test navigation
    widget.next();
    assert_eq!(widget.selected(), Some(1));

    widget.previous();
    assert_eq!(widget.selected(), Some(0));
}
```

### Performance Guidelines

#### Memory Management

- Use `Rc<RefCell<>>` for shared mutable state
- Prefer borrowing over cloning when possible
- Cache expensive computations
- Implement `Drop` for cleanup when needed

#### UI Performance

- Minimize redraws by tracking state changes
- Use lazy loading for large datasets
- Implement pagination for long lists
- Debounce user input

#### Database Performance

- Use prepared statements for repeated queries
- Implement connection pooling
- Cache frequently accessed data
- Batch operations when possible

## Adding New Features

### 1. UI Components

To add a new UI component:

```rust
// src/ui/components/new_widget.rs
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

pub struct NewWidget {
    // Widget state
}

impl NewWidget {
    pub fn new() -> Self {
        NewWidget {
            // Initialize state
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // Render widget
    }
}
```

Update `src/ui/components/mod.rs`:

```rust
pub mod new_widget;
```

### 2. Views

To add a new view:

```rust
// src/ui/views/new_view.rs
use super::components::NewWidget;

pub struct NewView {
    widget: NewWidget,
}

impl NewView {
    pub fn new() -> Self {
        NewView {
            widget: NewWidget::new(),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        self.widget.render(f, area);
    }
}
```

Update the `AppView` enum in `src/ui/app_ui.rs`:

```rust
pub enum AppView {
    TaskList,
    TaskDetail,
    Reports,
    Settings,
    Help,
    NewView,  // Add here
}
```

### 3. Commands

To add a new command:

```rust
// Update Action enum in src/handlers/input.rs
#[derive(Debug, Clone)]
pub enum Action {
    // ... existing actions
    NewAction,
}

// Update key handler
fn handle_key_event(&self, key: KeyEvent) -> Action {
    match key.code {
        // ... existing mappings
        KeyCode::Char('n') => Action::NewAction,
        _ => Action::None,
    }
}

// Update command handler in src/ui/app_ui.rs
pub async fn handle_action(&mut self, action: Action) -> Result<()> {
    match action {
        // ... existing actions
        Action::NewAction => {
            // Handle new action
        }
        _ => {}
    }
    Ok(())
}
```

### 4. Configuration Options

To add new configuration options:

```rust
// Update Config struct in src/config.rs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    // ... existing fields
    pub new_feature: NewFeatureConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewFeatureConfig {
    pub enabled: bool,
    pub setting: String,
}

// Update Default implementation
impl Default for Config {
    fn default() -> Self {
        Config {
            // ... existing defaults
            new_feature: NewFeatureConfig {
                enabled: false,
                setting: "default".to_string(),
            },
        }
    }
}
```

## Debugging

### Logging

LazyTask uses structured logging:

```rust
use log::{debug, info, warn, error};

// Use appropriate log levels
debug!("Detailed debugging information");
info!("General information");
warn!("Something unusual happened");
error!("Something went wrong: {}", err);
```

Run with logging:

```bash
RUST_LOG=debug cargo run
```

### Debugging UI

For UI debugging, you can:

1. Add debug overlays
2. Log widget state changes
3. Use test data for consistent behavior
4. Add debug keybindings for state inspection

### Performance Profiling

```bash
# Profile memory usage
cargo run --release --features profiling

# Profile CPU usage
perf record cargo run --release
perf report

# Benchmark specific functions
cargo bench
```

## Contributing

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add/update tests
5. Update documentation
6. Run the test suite
7. Submit pull request

### Code Review Guidelines

- Code should be well-documented
- Tests should cover new functionality
- Follow existing code style
- Performance implications should be considered
- UI changes should be tested across terminals

### Issue Reporting

When reporting issues, include:

- LazyTask version
- Rust version
- Operating system
- Terminal type
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs/errors

## Release Process

### Version Numbers

LazyTask follows semantic versioning:

- `MAJOR.MINOR.PATCH`
- Breaking changes increment MAJOR
- New features increment MINOR
- Bug fixes increment PATCH

### Release Steps

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create release tag
5. Build release binaries
6. Publish to crates.io
7. Update documentation

## Resources

### Dependencies

- [Ratatui](https://ratatui.rs/) - TUI framework
- [Crossterm](https://docs.rs/crossterm/) - Cross-platform terminal manipulation
- [Tokio](https://tokio.rs/) - Async runtime
- [Serde](https://serde.rs/) - Serialization
- [Anyhow](https://docs.rs/anyhow/) - Error handling

### References

- [Rust Book](https://doc.rust-lang.org/book/)
- [Ratatui Tutorial](https://ratatui.rs/tutorials/)
- [Taskwarrior Documentation](https://taskwarrior.org/docs/)
- [Terminal Color Standards](https://en.wikipedia.org/wiki/ANSI_escape_code)

### Community

- [GitHub Issues](https://github.com/osamamahmood/lazytask/issues)
- [Discussions](https://github.com/osamamahmood/lazytask/discussions)
- [Contributing Guide](../CONTRIBUTING.md)
