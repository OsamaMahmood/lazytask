use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io::{self, Stdout};
use tokio::sync::mpsc;

use crate::config::Config;
use crate::handlers::input::InputHandler;
use crate::ui::app_ui::AppUI;

pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

pub struct App {
    pub config: Config,
    pub terminal: AppTerminal,
    pub ui: AppUI,
    pub input_handler: InputHandler,
    pub should_quit: bool,
}

impl App {
    pub fn new(config_path: Option<&str>, _verbose: bool) -> Result<Self> {
        // Initialize terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Load configuration
        let config = Config::load(config_path)?;
        
        // Initialize components
        let ui = AppUI::new(&config)?;
        let input_handler = InputHandler::new(&config);

        Ok(App {
            config,
            terminal,
            ui,
            input_handler,
            should_quit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Create channels for async communication
        let (_tx, mut _rx) = mpsc::channel::<String>(32);

        loop {
            // Draw UI
            self.terminal.draw(|f| self.ui.draw(f))?;

            // Handle input
            if let Some(action) = self.input_handler.handle_events().await? {
                match action {
                    crate::handlers::input::Action::Quit => {
                        self.should_quit = true;
                    }
                    _ => {
                        // Handle other actions
                        self.ui.handle_action(action).await?;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
