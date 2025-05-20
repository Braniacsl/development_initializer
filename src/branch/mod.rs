use anyhow::Result;


pub(super) mod add;
pub(super) mod default;
pub(super) mod edit;
pub(super) mod list;
pub(super) mod remove;
pub(super) mod set;
pub(super) mod view;

pub trait Branch {
    fn execute(&self) -> Result<()>;
}