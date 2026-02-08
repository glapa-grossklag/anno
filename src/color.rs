use std::env;
use std::io::{self, IsTerminal};

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const PURPLE: &str = "\x1b[35m";
const RESET: &str = "\x1b[0m";

pub struct ColorScheme {
    use_color: bool,
}

impl ColorScheme {
    pub fn new() -> Self {
        Self {
            use_color: should_use_color(),
        }
    }

    pub fn addr(&self, text: &str) -> String {
        if self.use_color {
            format!("{}{}{}", GREEN, text, RESET)
        } else {
            text.to_string()
        }
    }

    pub fn annotation(&self, text: &str) -> String {
        if self.use_color {
            format!("{}{}{}", BLUE, text, RESET)
        } else {
            text.to_string()
        }
    }

    pub fn label(&self, label: &str) -> String {
        if !self.use_color {
            return label.to_string();
        }

        // Format is "type: value" - color type purple, colon uncolored, value blue
        if let Some(colon_pos) = label.find(": ") {
            let type_part = &label[..colon_pos];
            let value_part = &label[colon_pos + 2..];
            format!("{}{}{}: {}{}{}", PURPLE, type_part, RESET, BLUE, value_part, RESET)
        } else {
            // Fallback: just color it blue if format doesn't match
            format!("{}{}{}", BLUE, label, RESET)
        }
    }
}

fn should_use_color() -> bool {
    // Check NO_COLOR environment variable
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check if TERM is dumb
    if let Ok(term) = env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Check if stdout is a terminal
    io::stdout().is_terminal()
}
