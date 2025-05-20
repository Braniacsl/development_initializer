use anyhow::{Result, Context};
use std::process::Stdio;
use std::process::Command;
use std::io::Write;

use crate::branch::Branch;
use crate::db::{
    project::Project,
    settings::Settings
};
use crate::config::{ProjectConfig, Program};

#[derive(Debug)]
pub struct DefaultCommand {
    pub alias: Option<String>,
}

impl Branch for DefaultCommand {
    fn execute(&self) -> Result<()> {
        // Ensure alias is provided
        let alias = self.alias
            .as_ref()
            .context("No alias input.")?;

        // Fetch the project based on the alias
        let project: Project = Project::get(&[alias.to_string()]).context("Failed to fetch project.")?;

        // Deserialize the TOML configuration
        let config: ProjectConfig = toml::from_str(&project.toml)
            .context("Failed to deserialize TOML configuration.")?;

        project.execute(config)?;

        Ok(())
    }
}

struct ProcessManager;

impl ProcessManager {
    fn run_program(program: &Program) -> Result<()> {
        let mut cmd: Command = 
            if let Some(settings) = &program.settings {
                if settings.uwsm {
                    let mut temp = Command::new("uwsm");
                    temp.args(["app", "--", &program.path]);
                    temp
                }
                else {
                    Command::new(&program.path)
                }
            }
            else {
                Command::new(&program.path)
            };
        

        // Set working directory
        if let Some(dir) = &program.working_directory {
            cmd.current_dir(dir);
        }
    
        // Add arguments
        if let Some(args) = &program.args {
            cmd.args(args);
        }
    
        // Add environment variables
        if let Some(env_vars) = &program.env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        // Handle output mode
        match program.output_mode.as_deref() {
            Some("null") => {
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());
            }
            Some("inherit") => {
                cmd.stdout(Stdio::inherit());
                cmd.stderr(Stdio::inherit());
            }
            Some("log") => {
                // TODO: Implement logging to a file
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());
            }
            _ => {}
        }
    
        // Execute the command
        let mut child = cmd
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to execute program:")?;

        // Write commands to stdin
        if let Some(commands) = &program.commands {
            if let Some(mut stdin) = child.stdin.take() {
                for command in commands {
                    stdin.write(format!("{}\n", command).as_bytes())?;
                    stdin.flush()?;
                }
            }
        }
    
        Ok(())      
    }
}

impl Project {
    fn execute(&self, config: ProjectConfig) -> Result<()> {
        for mut program in config.programs.list {
            if program.settings.is_none() {
                program.settings = Some(Settings::get_all()?);
            }

            ProcessManager::run_program(&program)?;
        }
        Ok(())
    }
}