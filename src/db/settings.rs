use anyhow::{Result,Context};
use serde::{Deserialize, Serialize};

use crate::db::DB;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub uwsm: bool,
}

impl Settings {
    pub fn get_all() -> Result<Self> {
        let conn = DB::connect()?;

        Ok(conn.query_row(
            "SELECT * FROM settings",
            [],
            |row| Ok(Settings {
                uwsm: row.get(0)?,
            }))
            .context("Failed to retrieve general settings.")?
        )
    }

    pub fn set_uwsm(value: Option<String>) -> Result<()> {
        let conn = DB::connect()?;

        match value {
            Some(val) => {
                let mut stmt = conn.prepare(
                    "
                    UPDATE settings
                    SET uwsm = ?
                    ")?;

                stmt.execute([val])?;

                Ok(())
            },
            None => {
                let mut stmt = conn.prepare("
                UPDATE settings
                SET uwsm = ((uwsm | 1) - (uwsm & 1))
                ")?;

                stmt.execute([])?;

                Ok(())
            }
        }
    }
}