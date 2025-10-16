use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Refresh,
    Help,
    AddTask,
    EditTask,
    DoneTask,
    DeleteTask,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Select,
    Back,
    Filter,
    Context,
    Reports,
    Character(char),
    Backspace,
    None,
    Space,
    Tab,
}

pub struct InputHandler {
    config: Config,
}

impl InputHandler {
    pub fn new(config: &Config) -> Self {
        InputHandler {
            config: config.clone(),
        }
    }

    pub async fn handle_events(&self) -> Result<Option<Action>> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                return Ok(Some(self.handle_key_event(key)));
            }
        }
        Ok(None)
    }

    fn handle_key_event(&self, key: KeyEvent) -> Action {
        self.handle_key_event_with_context(key, false)
    }

    pub fn handle_key_event_with_context(&self, key: KeyEvent, in_form: bool) -> Action {
        if in_form {
            match key.code {
                KeyCode::Esc => Action::Back,
                KeyCode::Enter => Action::Select,
                KeyCode::Up => Action::MoveUp,
                KeyCode::Down => Action::MoveDown,
                KeyCode::Left => Action::MoveLeft,   // Enable cursor movement in forms
                KeyCode::Right => Action::MoveRight, // Enable cursor movement in forms
                KeyCode::Tab => Action::Tab, // Tab for section navigation in filters
                KeyCode::BackTab => Action::MoveUp, // Shift+Tab moves to previous field (same as up arrow)
                KeyCode::Backspace => Action::Backspace,
                KeyCode::Char(' ') => Action::Space, // Space for toggling filters
                KeyCode::Char(c) => Action::Character(c),
                _ => Action::None,
            }
        } else {
            match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::F(1) => Action::Help,
                KeyCode::F(5) => Action::Refresh,
                KeyCode::Char('a') => Action::AddTask,
                KeyCode::Char('e') => Action::EditTask,
                KeyCode::Char('d') => Action::DoneTask,
                KeyCode::Delete => Action::DeleteTask,
                KeyCode::Up => Action::MoveUp,
                KeyCode::Down => Action::MoveDown,
                KeyCode::Left => Action::MoveLeft,
                KeyCode::Right => Action::MoveRight,
                KeyCode::Enter => Action::Select,
                KeyCode::Esc => Action::Back,
                KeyCode::Char('/') => Action::Filter,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
                KeyCode::Char('c') => Action::Context,
                KeyCode::Char('r') => Action::Reports,
                KeyCode::Tab => Action::Tab,
                KeyCode::Backspace => Action::Backspace,
                KeyCode::Char(' ') => Action::Space,
                KeyCode::Char(c) => Action::Character(c), // Catch-all for other characters (t, <, >, etc)
                _ => Action::None,
            }
        }
    }
}
