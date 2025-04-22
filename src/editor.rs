use anyhow::Result;
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
    name: Box<str>,
    path: Box<str>,
    working_directory: Box<str>,
    commands: Vec<Box<str>>,
    auto_close: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Programs {
    list: Vec<Program>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectsTOML {
    programs: Programs,
}

pub struct Editor {}

impl Editor {
    pub fn input_editor(format: &str) -> Result<Box<str>> {
        let input = edit::edit(format)?;

        Ok(input.into_boxed_str())
    }
}