use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "devinit",
    about = "A tool to help initialize your development!"
)]
pub struct Request {
    /// Initialize a project 
    #[arg(default_value = "")]
    pub default: Box<str>,

    /// Commands (add, remove, list)
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add a new project or alias
    Add {
        /// The type of item to add (e.g., project, alias). Defaults to "project".
        #[arg(value_enum, default_value_t = ItemType::Project)]
        item_type: ItemType,

        /// The name of the item to add
        item: Box<str>,
    },

    /// View details of a project or alias
    View {
        /// The name of the project to view
        item: Box<str>,
    },

    /// Remove a project or alias
    Remove {
        /// The type of item to remove (e.g., project, alias). Defaults to "project".
        #[arg(value_enum, default_value_t = ItemType::Project)]
        item_type: ItemType,

        /// The name of the item to remove
        item: Box<str>,
    },

    /// Set a devinit setting
    Set {
        /// The option to change
        option: Box<str>,

        /// The value to change it to
        value: Box<str>,
    },

    /// List all projects or aliases
    List,
}

/// Enum to represent the type of item being operated on
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ItemType {
    Project,
    Alias,
    All,
}
