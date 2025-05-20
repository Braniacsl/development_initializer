use dialoguer::Confirm;
use anyhow::{Result, anyhow};

use crate::RemoveCommand;
use crate::branch::Branch;
use crate::db::{project::Project, alias::Alias};

impl RemoveCommand {
    pub fn new(
        project: Option<String>, 
        alias: Option<String>, 
        proj_name: Option<String>) 
    -> Self {
        RemoveCommand { 
            project, 
            alias, 
            proj_name
        }
    }

    fn remove_proj(alias: String) -> Result<()>{
        let confirm = Confirm::new()
            .with_prompt(format!("Are you sure you want to delete project {}?", &alias))
            .interact()?;

        if !confirm { return Ok(()) }

        let proj_id = Project::get_id(alias.to_string())?;

        Project::remove(proj_id)?;

        println!("Successfully removed project {}", &alias);

        Ok(())
    }

    fn remove_alias(alias: String) -> Result<()> {
        let confirm = Confirm::new()
            .with_prompt(format!("Are you sure you want to delete alias {}?", &alias))
            .interact()?;

        if !confirm { return Ok(()) }

        Alias::remove(alias.to_string())?;

        println!("Successfully removed alias {}", &alias);

        Ok(())
    }
}

impl Branch for RemoveCommand {
    fn execute(&self) -> Result<()> {
        if let Some(project) = &self.project {
            // Explicit project name provided via --project
            Self::remove_proj(project.to_string())
        } 
        else if let Some(proj_name) = &self.proj_name {
            // Implicit project name or alias provided as a positional argument
            Self::remove_proj(proj_name.to_string())
        } else if let Some(alias) = &self.alias {
            // Explicit alias provided via --alias
            Self::remove_alias(alias.to_string())
        } else {
            // No valid input provided
            return Err(anyhow!("No project name or alias provided."));
        }
    }
}