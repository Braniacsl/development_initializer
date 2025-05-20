use rusqlite::ToSql;
use anyhow::{Result, anyhow, Context};
use crate::db::DB;

pub struct Project {
    pub id: i32,
    pub name: String, 
    pub toml: String,
}

impl Project {

    pub fn get(names: &[String]) -> Result<Self> {
        if names.is_empty() {
            return Err(anyhow!("At least one alias must be provided."));
        }

        // Connect to the database
        let conn = DB::connect().context("Unable to get project.")?;

        // Create placeholders for the query
        let placeholders: Vec<_> = names.iter().map(|_| "?").collect();
        let placeholders_str = placeholders.join(",");

        // Prepare the SQL query
        let query = format!(
            "
            SELECT p.id, p.name, p.toml 
            FROM projects p
            LEFT JOIN alias a ON p.id = a.id
            WHERE a.alias IN ({}) OR p.name IN ({})
            GROUP BY p.id
            HAVING COUNT(DISTINCT CASE WHEN a.alias IN ({}) THEN a.alias ELSE NULL END) + 
                COUNT(DISTINCT CASE WHEN p.name IN ({}) THEN p.name ELSE NULL END) = ?
            LIMIT 1",
            placeholders_str, placeholders_str, placeholders_str, placeholders_str
        );

        let mut stmt = conn
            .prepare(&query)
            .context("Failed to prepare get project statement.")?;

        let names_len = &names.len();
        
        // Construct the parameters for the query
        let params: Vec<&dyn ToSql> = names
            .iter()
            .flat_map(|name| std::iter::repeat(name as &dyn ToSql).take(4)) // Repeat each name 4 times
            .chain(std::iter::once(&names_len as &dyn ToSql)) // Append the count parameter
            .collect();

        // Execute the query and map the result to a `Project`
        let project = stmt.query_row(params.as_slice(), |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                toml: row.get(2)?,
            })
        })
        .context("Failed to map query to project.")?;

        Ok(project)
    }

    pub fn get_all() -> Result<Vec<Project>> {
        let conn = DB::connect()?;

        let mut stmt = conn
            .prepare("SELECT * FROM projects ")
            .context("Failed to prepare project get_all query.")?;

        let result: Vec<Project> = stmt
            .query_map([], |row| {
                Ok(
                    Project { 
                        id: row.get(0)?,
                        name: row.get(1)?,
                        toml: row.get(2)?,
                    }                    
                )
            }).unwrap()
            .collect::<Result<Vec<Project>, _>>() 
            .context("Failed to execute project get all branch.")?;

        Ok(result)
    }

    pub fn add(name: String, toml: String) -> Result<i32> {
        let conn = DB::connect()?;

        let mut stmt = conn
            .prepare(&format!(
                "
                INSERT INTO projects(name, toml)
                VALUES (?, ?)"
            ))
            .context("Failed to prepare project add query.")?;

        stmt.execute(
            [name, toml]
        )?;

        Ok(conn.last_insert_rowid().try_into().unwrap())
    }

    pub fn remove(id: i32) -> Result<()> {
        let mut conn = DB::connect()?;

        let tx = conn.transaction()?;

        tx
            .execute("DELETE FROM projects WHERE id = ?", [id])?;

        tx.commit()?;

        Ok(())
    }

    pub fn replace_toml(id: i32, toml: String) -> Result<()> {
        let mut conn = DB::connect()?;

        let tx = conn.transaction()?;

        tx.execute(
            "UPDATE projects SET toml = ? WHERE id = ?", 
        [toml, id.to_string()])?;

        tx.commit()?;

        Ok(())
    }

    pub fn get_id(name: String) -> Result<i32> {
        let conn = DB::connect()?;

        let id: i32 = conn.query_row(
        "SELECT p.id 
            FROM projects p
            WHERE name = ?
            OR ? in (SELECT a.alias FROM alias a WHERE p.id = a.id)", 
            [&name, &name], 
            |row| row.get(0)
            )?;

        Ok(id)
    }
}