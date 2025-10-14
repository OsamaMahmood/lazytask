// Color schemes and styling

use ratatui::style::{Color, Style};
use std::collections::HashMap;

pub struct Theme {
    pub name: String,
    pub colors: HashMap<String, Color>,
}

impl Theme {
    pub fn catppuccin_mocha() -> Self {
        let mut colors = HashMap::new();
        colors.insert("background".to_string(), Color::Rgb(30, 30, 46));
        colors.insert("foreground".to_string(), Color::Rgb(205, 214, 244));
        colors.insert("primary".to_string(), Color::Rgb(137, 180, 250));
        colors.insert("secondary".to_string(), Color::Rgb(243, 139, 168));
        colors.insert("success".to_string(), Color::Rgb(166, 227, 161));
        colors.insert("warning".to_string(), Color::Rgb(249, 226, 175));
        colors.insert("error".to_string(), Color::Rgb(243, 139, 168));

        Theme {
            name: "Catppuccin Mocha".to_string(),
            colors,
        }
    }

    pub fn get_color(&self, name: &str) -> Color {
        self.colors.get(name)
            .copied()
            .unwrap_or(Color::White)
    }

    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.get_color("primary"))
    }

    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.get_color("secondary"))
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.get_color("success"))
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.get_color("warning"))
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.get_color("error"))
    }
}

