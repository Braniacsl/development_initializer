use anyhow::Result;
use std::collections::HashMap;

use crate::branch::Branch;
use crate::db::{project::Project, alias::Alias};

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