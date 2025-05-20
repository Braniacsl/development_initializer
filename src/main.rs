mod branch;
mod cli;
mod db;
mod config;

use branch::{
    Branch,
    default::DefaultCommand,
    list::ListCommand,
    edit::EditCommand,
    view::ViewCommand,
};
pub use cli::{
    Cli,
    Commands,
    AddCommand,
    RemoveCommand,
    SetCommand
};

use clap::Parser;
use anyhow::Result;

fn main() -> Result<()> {
    // Parse the CLI arguments
    let cli = Cli::parse();

    let command = match cli.command {
        Some(cmd) => cmd, // 
        None => {
            if let Some(project_name) = cli.project_name {
                Commands::Default { alias: project_name }
            } else {
                eprintln!("Error: You must provide a project name or a valid subcommand.");
                std::process::exit(1);
            }
        }
    };

    // Dispatch based on the subcommand
    let branch: Box<dyn Branch> = match command {
            Commands::Default { alias } => Box::new(DefaultCommand { alias: Some(alias) }),
            Commands::Add(add_command) => Box::new(add_command),
            Commands::Remove(remove_command) => Box::new(remove_command),
            Commands::View { alias} => Box::new(ViewCommand { alias }),
            Commands::List  => Box::new(ListCommand {}),
            Commands::Set(set_command) => Box::new(set_command),
            Commands::Edit { alias } => Box::new(EditCommand { alias }),
        };
    

    // Execute the branch's functionality
    branch.execute()
}