use anyhow::{Result, anyhow};
use dialoguer::{Confirm, Input};

use crate::{
    db::{Alias, Projects},
    editor::{Editor, ProjectsTOML, PROJECT_FORMAT}, 
    errors::BuildError, 
    request::{Command, Request, ItemType}
};

#[derive(Debug)]
enum Process {
    Add(Box<str>),
    AddAlias(Box<str>),
    Remove,
    RemoveAlias(Box<str>),
    View,
    List,
}

#[derive(Debug)]
pub struct Environment {
    name: Alias,
    process: Process,
}

impl Environment {
    pub fn run(&self) -> Result<()>{
        match &self.process {
            Process::Add(raw_toml) => Projects::add(self.name.clone(), raw_toml)?,
            Process::AddAlias(to_add) => self.name.db_add(to_add.clone())?,
            Process::Remove => Projects::remove(self.name.clone())?,
            Process::RemoveAlias(name) => Alias::db_remove(self.name.clone(), name)?,
            Process::View => {
                println!("Viewing {}\n", self.name.to_vec().first().expect("No names in Alias"));
                println!("Aliases: {}\n", self.name);
                println!("Project TOML: \n {}", Projects::get_toml(self.name.clone())?)
            }
            Process::List => {
                let projects: Vec<Projects> = Projects::get_all();
                let aliases = Alias::get_all();

                println!("Listing all Projects and Aliases:\n");
                

                println!("Projects")
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct EnvironmentBuilder;

impl EnvironmentBuilder {
    pub fn new(request: Request) -> Result<Environment> {
        match request.command {
            Some(Command::Add { item_type, item }) => {
                if item_type == ItemType::Project { return Self::add_project(item) }
                else if item_type == ItemType::Alias { return Self::add_alias(item) }
                else { return Err(anyhow!("Unkown --add option"))}
            },
            Some(Command::Remove { item_type, item }) => {
                if item_type == ItemType::Project { return Self::remove(item) }
                else if item_type == ItemType::Alias { return Self::remove_alias(item) }
                else { return Err(anyhow!("Unkown --remove option"))}
            },
            Some(Command::View { item }) => {
                Self::view(item)
            },
            Some(Command::List) => {
                Self::list()
            },
            Some(Command::Set { option, value }) => {
                Self::set(option, value)
            }
            None if request.default.is_empty() => {
                Err(BuildError::IncorrectArgs)?
            },
            None => {
                Self::initialize(Alias::retrieve(request.default)?)
            }
        }
    }

    fn add_project(name: Box<str>) -> Result<Environment> {
        println!("Adding a new project: '{}'", name);

        // Open Editor to retrieve project toml
        let project_toml_raw = loop {
            let project_toml_raw = Editor::input_editor(PROJECT_FORMAT)?;

            match toml::from_str::<ProjectsTOML>(&project_toml_raw) {
                Ok(_) => break project_toml_raw,
                Err(e) => {
                    println!("Invalid TOML. Please correct the errors and try again.");
                    println!("Error: {}", e);
                    continue;
                }
            }
        };

        println!("Project TOML retrieved successfully!");

        // Retrieve aliases
        let alias_prompt = "Would you like to add aliases for this project?";
        let mut aliases = Alias::from(name.into());

        if Confirm::new()
            .with_prompt(alias_prompt)
            .default(true)
            .interact()?
        {
            loop {
                let alias = Input::<String>::new()
                    .with_prompt("Enter an alias (or leave blank to finish)")
                    .allow_empty(true)
                    .interact()?;
                
                if alias.is_empty() { break }

                aliases.add(alias.into_boxed_str());

            }
        }

        // Create Environment
        Ok(Environment {
            name: aliases,
            process: Process::Add(project_toml_raw)
        })

    }

    fn add_alias(name: Box<str>) -> Result<Environment> {
        let alias = Input::<String>::new()
            .with_prompt("Enter an alias in the group to add to.")
            .allow_empty(false)
            .interact()?;

        let alias = Alias::retrieve(alias.into_boxed_str())?;
        
        Ok(Environment { 
            name: alias, 
            process: Process::AddAlias(name.into()) 
        })
    }

    fn remove(name: Box<str>) -> Result<Environment> {
        let name = Alias::retrieve(name.into())?;

        Ok(Environment { 
            name: name, 
            process: Process::Remove
        })
    }

    fn remove_alias(name: Box<str>) -> Result<Environment> {
        let alias = Alias::retrieve(name.clone().into())?;

        Ok(Environment { 
            name: alias, 
            process: Process::RemoveAlias(name)
        })
    }

    fn view(name: Box<str>) -> Result<Environment> {
        let name = Alias::retrieve(name.into())?;

        Ok(Environment { 
            name: name, 
            process: Process::View })
    }

    fn list() -> Result<Environment> {
        Ok(Environment {
            name: Alias::new(),
            process: Process::List
        })
    }

    fn set(option: Box<str>, item: Box<str>) -> Result<Environment> {
        unimplemented!()
    }

    fn initialize(name: Alias) -> Result<Environment> {
        unimplemented!()
    }
}