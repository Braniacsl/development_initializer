use dialoguer::{Confirm, Input};
use anyhow::{anyhow, Result};

use crate::AddCommand;
use crate::branch::Branch;
use crate::config::{PROJECT_FORMAT, Editor};
use crate::db::{project::Project, alias::Alias};

impl AddCommand {
    pub fn new(
        project: Option<String>, 
        alias: Option<String>, 
        proj_name: Option<String>) 
    -> Self {
        AddCommand { 
            project, 
            alias, 
            proj_name}
    }

    fn add_proj(name: String) -> Result<()> {
        println!("Adding project {}", name);

        let toml = Editor::input_editor(PROJECT_FORMAT)?;

        let proj_id = Project::add(name, toml)?;

        let aliases= if Confirm::new()
            .with_prompt("Would you like to add aliases?")
            .default(false)
            .interact()?
            {
                //Input aliases
                let aliases: String= Input::<String>::new()
                    .with_prompt("Enter aliases separated by commas e.g. <alias1>, <alias2>, <alias3>, ...")
                    .allow_empty(false)
                    .interact()?;

                //Collect aliases
                let aliases: Vec<String> = aliases
                    .split(',')
                    .map(str::trim)
                    .map(String::from)
                    .collect(); 

                //Validate aliases
                let is_valid = aliases
                    .iter()
                    .enumerate()
                    .any(|(i, alias)| {
                        let unique = aliases
                            .iter()
                            .skip(i + 1)
                            .all(|alias2| alias != alias2);

                        let exists = 
                            Alias::check(alias.to_string()).unwrap();

                        unique && !exists
                        }
                    );

                if is_valid {
                    aliases
                }
                else {
                    return Err(anyhow!("Duplicate alias or otherwise invalid."));
                }
            }
            else {
                return Ok(())
            };

        if aliases.is_empty() {
            println!("Successfully added project.");
            return Ok(());
        }

        Alias::add_all(proj_id, aliases)?;

        println!("Successfully added project and aliases.");

        Ok(())
    }

    fn add_alias(alias: String) -> Result<()> {
        println!("Adding alias {}", alias);

        let proj_name = Input::new()
            .with_prompt("Enter the name or an alias of a project: ")
            .interact()?;

        let proj_id = Project::get_id(proj_name)?;

        Alias::add(proj_id, alias)
    }
}

impl Branch for AddCommand {    
    fn execute(&self) -> Result<()> {
        if let Some(project) = &self.project {
            // Explicit project name provided via --project
            Self::add_proj(project.to_string())
        } 
        else if let Some(proj_name) = &self.proj_name {
            // Implicit project name or alias provided as a positional argument
            Self::add_proj(proj_name.to_string())
        } else if let Some(alias) = &self.alias {
            // Explicit alias provided via --alias
            Self::add_alias(alias.to_string())
        } else {
            // No valid input provided
            return Err(anyhow!("No project name or alias provided."));
        }
    }
}