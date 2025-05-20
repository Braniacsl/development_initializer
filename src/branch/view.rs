use anyhow::Result;

use crate::db::{
    project::Project,
    alias::Alias
};
use crate::branch::Branch;

#[derive(Debug)]
pub struct ViewCommand {
    pub alias: String,
}

impl Branch for ViewCommand {
    fn execute(&self) -> Result<()> {
        println!("Viewing details of {}", &self.alias);

        let project = Project::get(&[self.alias.to_string()])?;

        let aliases = Alias::get(project.id)?;

        println!("Project Name: {}\nAliases: {}\nProject TOML: \n{}", project.name, aliases, project.toml, );

        Ok(())
    }
}