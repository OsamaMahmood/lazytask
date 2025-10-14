// Configurable key mapping

use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct KeyBindings {
    bindings: HashMap<String, KeyBinding>,
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key_code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBindings {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        
        // Default key bindings
        bindings.insert("quit".to_string(), KeyBinding {
            key_code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
        });
        
        bindings.insert("help".to_string(), KeyBinding {
            key_code: KeyCode::F(1),
            modifiers: KeyModifiers::NONE,
        });

        KeyBindings { bindings }
    }

    pub fn get(&self, action: &str) -> Option<&KeyBinding> {
        self.bindings.get(action)
    }

    pub fn matches(&self, key_event: &KeyEvent, action: &str) -> bool {
        if let Some(binding) = self.bindings.get(action) {
            key_event.code == binding.key_code && key_event.modifiers == binding.modifiers
        } else {
            false
        }
    }
}

