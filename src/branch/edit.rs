use anyhow::Result;

use crate::branch::Branch;
use crate::db::project::Project;
use crate::config::Editor;

#[derive(Debug)]
pub struct EditCommand {
    pub alias: String,
}

impl Branch for EditCommand {
    fn execute(&self) -> Result<()> {
        let project = Project::get(&[self.alias.to_string()])?;

        let new_toml = Editor::input_editor(&project.toml)?;

        Project::replace_toml(project.id, new_toml) 
    }
}