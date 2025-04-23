mod branch;
mod cli;
mod db;
mod config;

pub use branch::{
    Branch,
    DefaultCommand,
    ListCommand,
    EditCommand,
    ViewCommand,
    
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

    // Dispatch based on the subcommand
    let branch: Box<dyn Branch> = match cli.command {
        Commands::Default { alias } => Box::new(DefaultCommand { alias }),
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