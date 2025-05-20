use rusqlite::params;
use anyhow::{Result, Context};
use crate::db::DB;

pub struct Alias {
    pub id: i32,
    pub alias: String,
}

impl Alias {
    pub fn check(alias: String) -> Result<bool> {
        let conn = DB::connect()?;

        let result: bool = conn.query_row(
        "SELECT EXISTS (
                SELECT 1
                FROM alias
                WHERE alias = ?
                )", 
            [alias], 
            |row| row.get(0))
        .context("Failed to prepare alias check query.")?;

        Ok(result)
    }

    pub fn get(id: i32) -> Result<String> {
        let conn = DB::connect()?;

        let mut stmt = conn
            .prepare("SELECT alias FROM alias WHERE id = ?")
            .context("Failed to prepare alias get query.")?;

        let result: String = stmt
            .query_map([id], |row| {
                row.get(0) 
            }).unwrap()
            .collect::<Result<Vec<String>, _>>() 
            .context("Failed to execute alias get query")?
            .join(", ");

        Ok(result)
    }

    pub fn get_all() -> Result<Vec<Alias>> {
        let conn = DB::connect()?;

        let mut stmt = conn
            .prepare("SELECT * FROM alias ")
            .context("Failed to prepare alias get all query.")?;

        let result: Vec<Alias> = stmt
            .query_map([], |row| {
                Ok(
                    Alias { 
                        id: row.get(0)?,
                        alias: row.get(1)?,
                    }                    
                )
            }).unwrap()
            .collect::<Result<Vec<Alias>, _>>() 
            .context("Failed to execute alias get all query.")?;

        Ok(result)
    }

    pub fn add_all(id: i32, aliases: Vec<String>) -> Result<()> {
        let mut conn = DB::connect()?;

        let tx = conn.transaction()?;

        for alias in aliases {
            tx.execute(
            "INSERT INTO alias (id, alias) 
                VALUES(?, ?) ", 
                params![id, alias]
            )?;
        }

        tx
        .commit()?;

        Ok(())
    }

    pub fn add(id: i32, alias: String) -> Result<()> {
        let mut conn = DB::connect()?;

        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO alias (id, alias) 
                VALUES(?, ?) ", 
                params![id, alias]
            )?;

        tx
        .commit()?;

        Ok(())
    }

    pub fn remove(alias: String) -> Result<()>{
        let mut conn = DB::connect()?;

        let tx = conn.transaction()?;

        tx.execute(
            "REMOVE FROM alias WHERE alias = ?", 
                params![alias]
            )?;

        tx
        .commit()?;

        Ok(())

    }
}