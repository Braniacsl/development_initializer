use clap::{Parser, Subcommand};

/// The main CLI struct for the `devinit` application.
#[derive(Parser, Debug)]
#[command(name = "devinit", about = "A tool to help initialize your development!", version)]
pub struct Cli {
    /// The option to run (a project, add a project, view, etc.)
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// The project name to run
    #[arg()]
    pub project_name: Option<String>,
}

/// Enum representing the subcommands of the `devinit` application.
#[derive(Subcommand, Debug)]
pub enum Commands {
    Default {
        alias: String,
    },

    /// Add a project or alias to the program.
    Add(AddCommand),

    /// Remove a project or alias from the program.
    Remove(RemoveCommand),

    /// View details of a project or its aliases.
    View {
        alias: String,
    },

    /// List all projects and their aliases.
    List,

    /// Set options like preferred editor (not expanded yet).
    Set (SetCommand),

    /// Edit a project or alias.
    Edit {
        /// The project name or alias to edit.
        alias: String,
    },
}

/// Subcommand for `add` operations.
#[derive(Parser, Debug)]
pub struct AddCommand {
    /// Explicitly specify the project name.
    #[arg(long, conflicts_with = "alias")]
    pub project: Option<String>,

    /// Explicitly specify the alias for a project.
    #[arg(long, conflicts_with = "project")]
    pub alias: Option<String>,

    /// Implicit project name or alias (positional argument).
    #[arg(conflicts_with_all = &["project", "alias"])]
    pub proj_name: Option<String>,
}

/// Subcommand for `remove` operations.
#[derive(Parser, Debug)]
pub struct RemoveCommand {
    /// Explicitly specify the project name to remove.
    #[arg(long, conflicts_with = "alias")]
    pub project: Option<String>,

    /// Explicitly specify the alias to remove.
    #[arg(long, conflicts_with = "project")]
    pub alias: Option<String>,

    /// Implicit project name or alias (positional argument).
    #[arg(conflicts_with_all = &["project", "alias"])]
    pub proj_name: Option<String>,
}

#[derive(Parser, Debug)]
pub struct SetCommand {
    pub option: Option<String>,
    pub value: Option<String>
}
