use std::collections::HashMap;
use std::process::Command;

use anyhow::{anyhow, Result};
use dialoguer::{Confirm, Input};

use crate::cli::{AddCommand, SetCommand};
use crate::db::{Project, Alias};
use crate::config::{Editor, PROJECT_FORMAT};
use crate::RemoveCommand;

pub trait Branch {
    fn execute(&self) -> Result<()>;
}

#[derive(Debug)]
pub struct DefaultCommand {
    pub alias: Option<String>,
}

impl Branch for DefaultCommand {
    fn execute(&self) -> Result<()> {
        unimplemented!()
        // // Ensure alias is provided
        // let alias = self.alias.as_ref().context("No alias input.")?;

        // // Fetch the project based on the alias
        // let project: Project = Project::get(&[alias.to_string()]).context("Failed to fetch project.")?;

        // // Deserialize the TOML configuration
        // let config: Config = toml::from_str(&project.toml)
        //     .context("Failed to deserialize TOML configuration.")?;

        // // Use the deserialized configuration (if needed)
        // println!("Deserialized configuration: {:?}", config);

        // Ok(())
    }
}

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

                        println!("unique {}", unique);
                        println!("exists {}", exists);

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

#[derive(Debug)]
pub struct ViewCommand {
    pub alias: String,
}

impl ViewCommand {
    pub fn new(alias: String) -> Self {
        ViewCommand { alias }
    }
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

pub struct ListCommand {}

impl Branch for ListCommand {
    fn execute(&self) -> Result<()> {
        println!("Viewing all projects:\n");

        let projects = Project::get_all()?;

        let aliases = Alias::get_all()?;

        let mut alias_map: HashMap<i32, Vec<String>> = HashMap::new();
        for alias in aliases {
            alias_map.entry(alias.id).or_default().push(alias.alias);
        }

        for project in projects {
            println!("Project Name: {}", project.name);

            if let Some(project_aliases) = alias_map.get(&project.id) {
                println!("Aliases: {}", project_aliases.join(", "));
            } else {
                println!("Aliases: None");
            }

            println!("TOML Content:\n{}\n", project.toml);
        }

        Ok(())
    }
}

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
        let editor_name = 
            self.option
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
    }
}

#[derive(Debug)]
pub struct EditCommand {
    pub alias: String,
}

impl Branch for EditCommand {
    fn execute(&self) -> Result<()> {
        unimplemented!();
    }
}