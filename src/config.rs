use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};

pub const PROJECT_FORMAT: &str = "
[programs]
    [[programs.list]]
    name = \"\"
    path = \"\"
    working_directory = \"\"
    args = []
    commands = []
    output_mode = []
    env = []
    auto_close = false
    set_active_window = false
";

#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub name: String,
    pub path: String,
    pub working_directory: Option<String>,
    pub args: Option<Vec<String>>,
    pub commands: Option<Vec<String>>,
    pub output_mode: Option<String>, // e.g., "null", "inherit", "log"
    pub env: Option<HashMap<String, String>>,
    pub auto_close: Option<bool>,
    pub set_active_window: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Programs {
    pub list: Vec<Program>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub programs: Programs,
}

pub struct Editor {}

impl Editor {
    pub fn input_editor(format: &str) -> Result<String> {
        let input = match edit::edit(format) {
            Ok(i) => i,
            Err(e) => return Err(anyhow!(e))
        };

        Ok(input)
    }
}