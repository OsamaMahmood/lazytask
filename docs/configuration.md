# LazyTask Configuration

LazyTask is highly configurable through TOML configuration files. The main configuration is stored at `~/.config/lazytask/config.toml` and will be created with sensible defaults on first run.

## Configuration File Locations

LazyTask looks for configuration in the following locations (in order of precedence):

1. `--config` command line argument
2. `$XDG_CONFIG_HOME/lazytask/config.toml`
3. `~/.config/lazytask/config.toml`
4. Built-in defaults

## Main Configuration File

### Theme Configuration

```toml
[theme]
name = "catppuccin-mocha"  # Available: catppuccin-mocha, catppuccin-latte, dracula, gruvbox

[theme.colors]
# Override specific theme colors
background = "#1e1e2e"
foreground = "#cdd6f4"
primary = "#89b4fa"
secondary = "#f38ba8"
success = "#a6e3a1"
warning = "#f9e2af"
error = "#f38ba8"
```

### UI Configuration

```toml
[ui]
default_view = "task_list"           # Initial view: task_list, calendar, reports
show_help_bar = true                 # Show keybinding hints at bottom
refresh_interval = 1000              # Auto-refresh interval (milliseconds)
task_list_columns = [                # Columns to show in task list
    "id",
    "project",
    "priority",
    "due",
    "description"
]
```

Available columns:

- `id` - Task ID number
- `uuid` - Task UUID (shortened)
- `project` - Project name
- `priority` - Priority (H/M/L)
- `due` - Due date
- `description` - Task description
- `tags` - Task tags
- `urgency` - Calculated urgency
- `entry` - Creation date
- `modified` - Last modified date
- `status` - Task status

### Taskwarrior Integration

```toml
[taskwarrior]
taskrc_path = "/path/to/.taskrc"     # Leave empty for auto-detection
data_location = "/path/to/data"      # Leave empty for auto-detection
sync_enabled = false                 # Enable automatic synchronization
sync_interval = 300                  # Sync interval in seconds (when enabled)
```

### Keybindings

Keybindings are organized by context:

```toml
[keybindings.global]
quit = "q"
help = "F1"
refresh = "F5"
force_quit = "Ctrl+c"

[keybindings.task_list]
add_task = "a"
edit_task = "e"
done_task = "d"
delete_task = "Delete"
# ... more keybindings

[keybindings.task_detail]
save = "Ctrl+s"
cancel = "Esc"
# ... more keybindings
```

See [keybindings.md](keybindings.md) for complete keybinding reference.

## Advanced Configuration

### Custom Themes

You can define custom themes in the configuration:

```toml
[theme.custom]
name = "My Custom Theme"

[theme.custom.colors]
background = "#000000"
foreground = "#ffffff"
primary = "#0066cc"
secondary = "#cc6600"
success = "#00cc66"
warning = "#cccc00"
error = "#cc0000"
border = "#666666"
selected = "#333333"
highlight = "#444444"
inactive = "#777777"

# Priority colors
priority_high = "#ff0000"
priority_medium = "#ffaa00"
priority_low = "#00aa00"

# Project colors (cycling through for different projects)
project_1 = "#ff6666"
project_2 = "#66ff66"
project_3 = "#6666ff"
project_4 = "#ffff66"
project_5 = "#ff66ff"
project_6 = "#66ffff"
project_7 = "#ffffff"
project_8 = "#aaaaaa"
```

### Filter Presets

Define commonly used filters:

```toml
[filters.work]
name = "Work Tasks"
status = "pending"
project = "work"

[filters.urgent]
name = "Urgent Tasks"
status = "pending"
priority = "H"
due_before = "eow"  # End of week

[filters.today]
name = "Today's Tasks"
status = "pending"
due = "today"

[filters.overdue]
name = "Overdue Tasks"
status = "pending"
due_before = "now"
```

### Report Configuration

Customize built-in reports or define new ones:

```toml
[reports.next]
description = "Next tasks to work on"
columns = ["id", "project", "priority", "due", "description"]
filter = "status:pending limit:10"
sort = "urgency-"

[reports.weekly]
description = "This week's tasks"
columns = ["id", "project", "due", "description"]
filter = "status:pending due.before:eow"
sort = "due"

[reports.projects]
description = "Tasks by project"
columns = ["project", "count", "pending", "completed"]
filter = ""
sort = "project"
```

### Context Configuration

Define Taskwarrior contexts within LazyTask:

```toml
[contexts.work]
name = "Work"
read_filter = "project:work"
write_filter = "project:work"
description = "Work-related tasks"

[contexts.home]
name = "Home"
read_filter = "project:home"
write_filter = "project:home"
description = "Personal tasks"

[contexts.urgent]
name = "Urgent"
read_filter = "+urgent"
write_filter = ""
description = "Urgent tasks only"
```

## Environment Variables

LazyTask respects these environment variables:

- `TASKRC` - Path to taskrc file
- `TASKDATA` - Path to task data directory
- `XDG_CONFIG_HOME` - Alternative config directory
- `NO_COLOR` - Disable colors when set

## Configuration Examples

### Minimal Configuration

```toml
[theme]
name = "dracula"

[ui]
show_help_bar = false

[keybindings.global]
quit = "Ctrl+q"
```

### Power User Configuration

```toml
[theme]
name = "gruvbox"

[ui]
default_view = "calendar"
refresh_interval = 5000
task_list_columns = ["id", "project", "priority", "due", "urgency", "description", "tags"]

[taskwarrior]
sync_enabled = true
sync_interval = 600

# Vim-style navigation
[keybindings.task_list]
move_up = "k"
move_down = "j"
move_left = "h"
move_right = "l"
first_task = "gg"
last_task = "G"

# Custom filters
[filters.critical]
name = "Critical Tasks"
status = "pending"
priority = "H"
due_before = "tomorrow"

[filters.waiting]
name = "Waiting Tasks"
status = "waiting"

# Custom reports
[reports.burndown]
description = "Completion rate over time"
columns = ["date", "completed", "added", "net"]
filter = "end.after:30days"
```

## Validation and Errors

LazyTask validates configuration on startup and will show helpful error messages for:

- Invalid TOML syntax
- Unknown configuration keys
- Invalid color values
- Invalid keybinding syntax
- Missing required values

## Configuration Migration

When upgrading LazyTask, configuration files are automatically migrated:

1. Backup of old config is created
2. New fields are added with defaults
3. Deprecated fields are marked but preserved
4. Migration summary is shown

## Troubleshooting

### Configuration Not Loading

1. Check file path: `~/.config/lazytask/config.toml`
2. Verify TOML syntax with `toml-validate config.toml`
3. Check file permissions (should be readable)
4. Run with `--verbose` to see config loading messages

### Keybindings Not Working

1. Check for conflicting keybindings
2. Ensure key names are correct (case-sensitive)
3. Some terminals may not support all key combinations
4. Use `F1` to see current active keybindings

### Colors Not Showing

1. Check terminal color support (`echo $TERM`)
2. Try different theme
3. Check if `NO_COLOR` environment variable is set
4. Some terminals may not support all colors

### Taskwarrior Integration Issues

1. Verify Taskwarrior is installed and working
2. Check `taskrc_path` and `data_location` settings
3. Test with `task version` command
4. Check file permissions on Taskwarrior data directory

## Best Practices

1. **Start Simple**: Begin with minimal configuration and add as needed
2. **Backup Configs**: Keep backups of working configurations
3. **Test Changes**: Test configuration changes in development
4. **Use Comments**: Document custom settings with comments
5. **Version Control**: Consider versioning your config files
6. **Share Configs**: Share useful configurations with the community
