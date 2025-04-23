use serde::{Deserialize, Serialize};

pub const PROJECT_FORMAT: &str = "
[programs]
    [[programs.list]]
    name = \"\"
    path = \"\"
    working_directory = \"\"
    commands = []
    auto_close = false
    set_active_window = false
";

#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    name: String,
    path: String,
    working_directory: String,
    commands: Vec<String>,
    auto_close: bool,
    set_active_window: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Programs {
    list: Vec<Program>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    programs: Programs,
}

pub struct Editor {}

impl Editor {
    pub fn input_editor(format: &str) -> Result<String, String> {
        let input = match edit::edit(format) {
            Ok(i) => i,
            Err(e) => return Err(e.to_string())
        };

        Ok(input)
    }
}