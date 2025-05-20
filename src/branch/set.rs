use anyhow::{anyhow, Result};
use std::process::Command;

use crate::branch::Branch;
use crate::SetCommand;
use crate::db::settings::Settings;

impl SetCommand {
    fn find_editor_path(editor_name: &str) -> Result<String> {
        let output = Command::new("which")
            .arg(editor_name)
            .output()?;

        if output.status.success() {
            // Extract and trim the path from the command output
            let path = String::from_utf8(output.stdout)?
                .trim()
                .to_string();
            Ok(path)
        } else {
            Err(anyhow!("Editor '{}' not found", editor_name))
        }
    }
}

impl Branch for SetCommand {
    fn execute(&self) -> Result<()> {
        match &self.option {
            Some(op) if op == "editor"  => {
                let editor_name = 
                    self.value
                    .as_ref()
                    .ok_or(anyhow!("No editor specified"))?;

                let editor_path = Self::find_editor_path(editor_name)?;

                unsafe {
                    std::env::set_var("EDITOR", &editor_path);
                }

                println!(
                    "Default editor set to '{}' with path '{}'",
                    editor_name, editor_path
                );
            Ok(())
            },
            Some(op) if op == "uwsm" => {
                Settings::set_uwsm(self.value.clone())
            },
            Some(default) => Err(anyhow!("Unknown option given: {}.", default)),
            None => Err(anyhow!("No option given to set."))
        }
    }
}