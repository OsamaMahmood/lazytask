# LazyTask Keybindings

LazyTask uses keyboard shortcuts for efficient task management. All keybindings are configurable through the `~/.config/lazytask/config.toml` file.

## Global Keybindings

These work in any view:

| Key      | Action     | Description                   |
| -------- | ---------- | ----------------------------- |
| `q`      | Quit       | Exit the application          |
| `Ctrl+C` | Force Quit | Force exit the application    |
| `F1`     | Help       | Show context-sensitive help   |
| `F5`     | Refresh    | Refresh data from Taskwarrior |

## Task List View

The main task management interface:

### Navigation

| Key         | Action     | Description             |
| ----------- | ---------- | ----------------------- |
| `↑`/`k`     | Move Up    | Select previous task    |
| `↓`/`j`     | Move Down  | Select next task        |
| `←`/`h`     | Move Left  | Navigate to left panel  |
| `→`/`l`     | Move Right | Navigate to right panel |
| `Home`      | First Task | Jump to first task      |
| `End`       | Last Task  | Jump to last task       |
| `Page Up`   | Page Up    | Scroll up one page      |
| `Page Down` | Page Down  | Scroll down one page    |

### Task Operations

| Key      | Action      | Description              |
| -------- | ----------- | ------------------------ |
| `a`      | Add Task    | Create a new task        |
| `e`      | Edit Task   | Edit the selected task   |
| `d`      | Done Task   | Mark task as completed   |
| `Delete` | Delete Task | Delete the selected task |
| `s`      | Start Task  | Start working on task    |
| `S`      | Stop Task   | Stop working on task     |
| `n`      | Annotate    | Add annotation to task   |
| `D`      | Duplicate   | Create copy of task      |

### Selection and Interaction

| Key      | Action        | Description              |
| -------- | ------------- | ------------------------ |
| `Enter`  | Select        | Open task detail view    |
| `Space`  | Toggle Select | Toggle task selection    |
| `Ctrl+A` | Select All    | Select all visible tasks |
| `Esc`    | Back          | Return to previous view  |

### Views and Filters

| Key      | Action       | Description        |
| -------- | ------------ | ------------------ |
| `/`      | Filter       | Open filter dialog |
| `Ctrl+/` | Clear Filter | Remove all filters |
| `c`      | Context      | Switch context     |
| `r`      | Reports      | Open reports view  |
| `C`      | Calendar     | Open calendar view |
| `p`      | Projects     | Browse projects    |
| `t`      | Tags         | Browse tags        |

### Sorting

| Key | Action           | Description            |
| --- | ---------------- | ---------------------- |
| `1` | Sort by Due      | Sort tasks by due date |
| `2` | Sort by Priority | Sort by task priority  |
| `3` | Sort by Project  | Sort by project name   |
| `4` | Sort by Urgency  | Sort by urgency value  |

## Task Detail View

When viewing or editing a specific task:

### Navigation

| Key         | Action         | Description                 |
| ----------- | -------------- | --------------------------- |
| `Tab`       | Next Field     | Move to next editable field |
| `Shift+Tab` | Previous Field | Move to previous field      |
| `Esc`       | Cancel         | Cancel changes and return   |
| `Ctrl+S`    | Save           | Save changes                |

### Field Editing

| Key     | Action           | Description           |
| ------- | ---------------- | --------------------- |
| `Enter` | Edit Description | Edit task description |
| `p`     | Edit Project     | Change task project   |
| `P`     | Edit Priority    | Change task priority  |
| `d`     | Edit Due Date    | Set due date          |
| `t`     | Add Tag          | Add a tag             |
| `T`     | Remove Tag       | Remove selected tag   |
| `a`     | Add Annotation   | Add new annotation    |

## Calendar View

Interactive calendar for viewing tasks by date:

| Key     | Action         | Description                   |
| ------- | -------------- | ----------------------------- |
| `←`     | Previous Month | Go to previous month          |
| `→`     | Next Month     | Go to next month              |
| `↑`     | Previous Year  | Go to previous year           |
| `↓`     | Next Year      | Go to next year               |
| `t`     | Today          | Jump to current date          |
| `g`     | Go to Date     | Jump to specific date         |
| `a`     | Add Task       | Create task for selected date |
| `Enter` | View Day       | Show tasks for selected day   |
| `Esc`   | Back           | Return to previous view       |

## Reports View

Navigate through various task reports:

| Key   | Action          | Description               |
| ----- | --------------- | ------------------------- |
| `←`   | Previous Report | Switch to previous report |
| `→`   | Next Report     | Switch to next report     |
| `e`   | Export Report   | Export report data        |
| `r`   | Refresh Data    | Refresh report data       |
| `Esc` | Back            | Return to previous view   |

## Filter Builder

Build complex task filters interactively:

| Key     | Action        | Description                |
| ------- | ------------- | -------------------------- |
| `a`     | Add Filter    | Add new filter criterion   |
| `d`     | Remove Filter | Remove selected filter     |
| `c`     | Clear All     | Clear all filters          |
| `Enter` | Apply Filters | Apply filters to task list |
| `s`     | Save Preset   | Save filter as preset      |
| `l`     | Load Preset   | Load saved filter preset   |
| `Esc`   | Cancel        | Cancel filter changes      |

## Context Switcher

Manage Taskwarrior contexts:

| Key     | Action         | Description                     |
| ------- | -------------- | ------------------------------- |
| `Enter` | Select Context | Switch to selected context      |
| `c`     | Create Context | Create new context              |
| `e`     | Edit Context   | Edit context definition         |
| `d`     | Delete Context | Delete selected context         |
| `n`     | None Context   | Clear current context           |
| `Esc`   | Back           | Return without changing context |

## Customization

You can customize any keybinding by editing `~/.config/lazytask/config.toml`:

```toml
[keybindings.task_list]
add_task = "Insert"      # Change from 'a' to 'Insert'
done_task = "Enter"      # Change from 'd' to 'Enter'

[keybindings.global]
quit = "Ctrl+q"          # Change from 'q' to 'Ctrl+q'
```

### Key Notation

- Single keys: `a`, `1`, `Enter`, `Esc`, `Space`
- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Function keys: `F1`, `F2`, etc.
- Modifier combinations: `Ctrl+c`, `Shift+Tab`, `Alt+Enter`
- Special keys: `Home`, `End`, `PageUp`, `PageDown`, `Delete`

## Tips

1. **Vim-style Navigation**: Use `hjkl` for navigation if you prefer Vim-style keys
2. **Context Help**: Press `F1` in any view to see context-specific help
3. **Quick Actions**: Most common actions have single-key shortcuts
4. **Consistent Patterns**: Similar actions use similar keys across views
5. **Escape to Safety**: `Esc` always takes you back or cancels the current action
