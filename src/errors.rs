use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Invalid args. Type --help for help.")]
    IncorrectArgs,
    #[error("No editor specified")]
    UnspecifiedEditor,
}
