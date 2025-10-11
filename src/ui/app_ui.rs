use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::config::Config;
use crate::handlers::input::Action;

pub enum AppView {
    TaskList,
    TaskDetail,
    Reports,
    Settings,
    Help,
}

pub struct AppUI {
    config: Config,
    current_view: AppView,
    show_help_bar: bool,
}

impl AppUI {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(AppUI {
            config: config.clone(),
            current_view: AppView::TaskList,
            show_help_bar: config.ui.show_help_bar,
        })
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let size = f.size();
        
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),    // Header
                Constraint::Min(0),       // Main content
                Constraint::Length(1),    // Help bar
            ].as_ref())
            .split(size);

        // Draw header
        self.draw_header(f, chunks[0]);

        // Draw main content based on current view
        match self.current_view {
            AppView::TaskList => self.draw_task_list(f, chunks[1]),
            AppView::TaskDetail => self.draw_task_detail(f, chunks[1]),
            AppView::Reports => self.draw_reports(f, chunks[1]),
            AppView::Settings => self.draw_settings(f, chunks[1]),
            AppView::Help => self.draw_help(f, chunks[1]),
        }

        // Draw help bar
        if self.show_help_bar {
            self.draw_help_bar(f, chunks[2]);
        }
    }

    pub async fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                // This will be handled by the main app loop
            }
            Action::Help => {
                self.current_view = AppView::Help;
            }
            Action::Reports => {
                self.current_view = AppView::Reports;
            }
            Action::Back => {
                self.current_view = AppView::TaskList;
            }
            _ => {
                // Handle other actions based on current view
                match self.current_view {
                    AppView::TaskList => self.handle_task_list_action(action).await?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let title = match self.current_view {
            AppView::TaskList => "Taskwarrior TUI - Task List",
            AppView::TaskDetail => "Taskwarrior TUI - Task Detail",
            AppView::Reports => "Taskwarrior TUI - Reports",
            AppView::Settings => "Taskwarrior TUI - Settings",
            AppView::Help => "Taskwarrior TUI - Help",
        };

        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        
        f.render_widget(header, area);
    }

    fn draw_task_list(&self, f: &mut Frame, area: Rect) {
        // Placeholder task list - will be populated with real data later
        let items = vec![
            ListItem::new("Task 1 - Buy groceries [Home] H"),
            ListItem::new("Task 2 - Fix bug [Work] M"),
            ListItem::new("Task 3 - Call mom [Personal] L"),
        ];

        let tasks = List::new(items)
            .block(Block::default().title("Tasks").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol("> ");

        f.render_widget(tasks, area);
    }

    fn draw_task_detail(&self, f: &mut Frame, area: Rect) {
        let detail = Paragraph::new("Task Detail View - Coming Soon")
            .block(Block::default().title("Task Detail").borders(Borders::ALL));
        
        f.render_widget(detail, area);
    }

    fn draw_reports(&self, f: &mut Frame, area: Rect) {
        let reports = Paragraph::new("Reports View - Coming Soon")
            .block(Block::default().title("Reports").borders(Borders::ALL));
        
        f.render_widget(reports, area);
    }

    fn draw_settings(&self, f: &mut Frame, area: Rect) {
        let settings = Paragraph::new("Settings View - Coming Soon")
            .block(Block::default().title("Settings").borders(Borders::ALL));
        
        f.render_widget(settings, area);
    }

    fn draw_help(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from("Keyboard Shortcuts:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw("     - Quit application"),
            ]),
            Line::from(vec![
                Span::styled("F1", Style::default().fg(Color::Yellow)),
                Span::raw("    - Show this help"),
            ]),
            Line::from(vec![
                Span::styled("a", Style::default().fg(Color::Yellow)),
                Span::raw("     - Add new task"),
            ]),
            Line::from(vec![
                Span::styled("e", Style::default().fg(Color::Yellow)),
                Span::raw("     - Edit selected task"),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw("     - Mark task as done"),
            ]),
            Line::from(vec![
                Span::styled("Del", Style::default().fg(Color::Yellow)),
                Span::raw("   - Delete selected task"),
            ]),
            Line::from(""),
            Line::from("Press ESC to go back"),
        ];

        let help = Paragraph::new(help_text)
            .block(Block::default().title("Help").borders(Borders::ALL));
        
        f.render_widget(help, area);
    }

    fn draw_help_bar(&self, f: &mut Frame, area: Rect) {
        let help_text = match self.current_view {
            AppView::TaskList => "[a]dd [e]dit [d]one [Del]ete [/]filter [r]eports [q]uit",
            AppView::Help => "[ESC] back",
            _ => "[ESC] back [q]uit",
        };

        let help_bar = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray));
        
        f.render_widget(help_bar, area);
    }

    async fn handle_task_list_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::AddTask => {
                // TODO: Open add task dialog
            }
            Action::EditTask => {
                // TODO: Open edit task dialog
            }
            Action::DoneTask => {
                // TODO: Mark selected task as done
            }
            Action::DeleteTask => {
                // TODO: Delete selected task
            }
            _ => {}
        }
        Ok(())
    }
}
