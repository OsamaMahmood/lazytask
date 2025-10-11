use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub theme: ThemeConfig,
    pub keybindings: KeyBindingsConfig,
    pub taskwarrior: TaskwarriorConfig,
    pub ui: UIConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeConfig {
    pub name: String,
    pub colors: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyBindingsConfig {
    pub global: HashMap<String, String>,
    pub task_list: HashMap<String, String>,
    pub task_detail: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskwarriorConfig {
    pub taskrc_path: Option<PathBuf>,
    pub data_location: Option<PathBuf>,
    pub sync_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UIConfig {
    pub default_view: String,
    pub show_help_bar: bool,
    pub task_list_columns: Vec<String>,
    pub refresh_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        let mut global_keys = HashMap::new();
        global_keys.insert("quit".to_string(), "q".to_string());
        global_keys.insert("help".to_string(), "F1".to_string());
        global_keys.insert("refresh".to_string(), "F5".to_string());

        let mut task_list_keys = HashMap::new();
        task_list_keys.insert("add_task".to_string(), "a".to_string());
        task_list_keys.insert("edit_task".to_string(), "e".to_string());
        task_list_keys.insert("done_task".to_string(), "d".to_string());
        task_list_keys.insert("delete_task".to_string(), "Delete".to_string());

        let mut colors = HashMap::new();
        colors.insert("background".to_string(), "#1e1e2e".to_string());
        colors.insert("foreground".to_string(), "#cdd6f4".to_string());
        colors.insert("primary".to_string(), "#89b4fa".to_string());
        colors.insert("secondary".to_string(), "#f38ba8".to_string());

        Config {
            theme: ThemeConfig {
                name: "catppuccin-mocha".to_string(),
                colors,
            },
            keybindings: KeyBindingsConfig {
                global: global_keys,
                task_list: task_list_keys,
                task_detail: HashMap::new(),
            },
            taskwarrior: TaskwarriorConfig {
                taskrc_path: None,
                data_location: None,
                sync_enabled: false,
            },
            ui: UIConfig {
                default_view: "task_list".to_string(),
                show_help_bar: true,
                task_list_columns: vec![
                    "id".to_string(),
                    "project".to_string(),
                    "priority".to_string(),
                    "due".to_string(),
                    "description".to_string(),
                ],
                refresh_interval: 1000,
            },
        }
    }
}

impl Config {
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        let config_file_path = if let Some(path) = config_path {
            PathBuf::from(path)
        } else {
            Self::default_config_path()?
        };

        if config_file_path.exists() {
            let config_contents = fs::read_to_string(&config_file_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_file_path))?;
            
            let config: Config = toml::from_str(&config_contents)
                .with_context(|| "Failed to parse config file")?;
            
            Ok(config)
        } else {
            // Create default config file
            let default_config = Config::default();
            default_config.save(&config_file_path)?;
            Ok(default_config)
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let config_string = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;

        fs::write(path, config_string)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }

    fn default_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        Ok(config_dir.join("lazytask").join("config.toml"))
    }
}
