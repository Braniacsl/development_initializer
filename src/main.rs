mod branch;
mod cli;
mod db;
mod config;

pub use branch::{
    Branch,
    DefaultCommand,
    ListCommand,
    EditCommand
    
};
pub use cli::{
    Cli,
    Commands,
    AddCommand,
    RemoveCommand,
    ViewCommand,
    SetCommand
};

use clap::Parser;

fn main() -> Result<(), String> {
    // Parse the CLI arguments
    let cli = Cli::parse();

    // Dispatch based on the subcommand
    let branch: Box<dyn Branch> = match cli.command {
        Commands::Default { alias } => Box::new(DefaultCommand { alias }),
        Commands::Add(add_command) => Box::new(add_command),
        Commands::Remove(remove_command) => Box::new(remove_command),
        Commands::View ( view ) => Box::new(view),
        Commands::List  => Box::new(ListCommand {}),
        Commands::Set(set_command) => Box::new(set_command),
        Commands::Edit { alias } => Box::new(EditCommand { alias }),
    };

    // Execute the branch's functionality
    branch.execute()
}