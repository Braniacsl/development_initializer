use rusqlite::{params, Connection, ToSql};
use directories::ProjectDirs;
use std::{fs, path::Path};
use anyhow::{anyhow, Context, Result};

pub struct DB {}

impl DB {
    fn connect() -> Result<Connection> {
        // Get the project directory using ProjectDirs
        if let Some(proj_dirs) = ProjectDirs::from("com", "braniacs", "devinit") {
            let data_dir = proj_dirs.data_dir();
            let db_path = data_dir.join("devinit.db");

            if !data_dir.exists() {
                fs::create_dir_all(data_dir).context("Failed to create data directory")?;
            }

            // Open the database connection
            let conn = Connection::open(&db_path).context("Failed to open database connection")?;

            // Check if the database has been initialized
            match Self::is_db_initialized(&conn){
                Ok(true) => (),
                Ok(false) |
                Err(_) => {
                    Self::init_db(&db_path)?;
                }
            }

            // Open the database connection
            Ok(Connection::open(&db_path).context("Failed to open database connection")?)
        } else {
            Err(anyhow!("Failed to find or create project directories."))
        }
    }

    fn is_db_initialized(conn: &Connection) -> Result<bool> {
        // Check if the 'projects' table exists
        let mut stmt_projects = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='projects';",
        )?;
        let projects_table_exists: bool = stmt_projects.exists([])?;
    
        // Check if the 'alias' table exists
        let mut stmt_alias = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='alias';",
        )?;
        let alias_table_exists: bool = stmt_alias.exists([])?;
    
        // The database is initialized only if both tables exist
        Ok(projects_table_exists && alias_table_exists)
    }

    fn init_db(db_path: &Path) -> Result<()> {
        // Open a connection to the database file
        let conn = Connection::open(db_path).context("Failed to open database for initialization")?;

        // Execute SQL to create the tables
        match conn.execute(
            "
            CREATE TABLE projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT, -- Auto-incrementing unique ID
                name TEXT NOT NULL,                   -- Name of the project (string)
                toml TEXT NOT NULL                    -- TOML content (string)
            );"
            ,
            []
        ) {
            Ok(_) => (),
            Err(_) => (),
        };

        conn.execute(
            "
            -- Create the 'alias' table
            CREATE TABLE alias (
                id INTEGER NOT NULL,          -- Foreign key referencing the 'projects' table
                alias TEXT NOT NULL,                  -- Alias name (string)
                FOREIGN KEY (id) REFERENCES projects(id) ON DELETE CASCADE
            );
            ",
            [],
        )?;

        Ok(())
    }
}

pub struct Project {
    pub id: i32,
    pub name: String,
    pub toml: String,
}

impl Project {

    pub fn get(names: &[String]) -> Result<Self, String> {
        if names.is_empty() {
            return Err("At least one alias must be provided.".to_string());
        }

        // Connect to the database
        let conn = DB::connect().map_err(|_| "Failed to connect to the database.".to_string())?;

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
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

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
        }).map_err(|e| format!("Failed to fetch project: {}", e))?;

        Ok(project)
    }

    pub fn get_all() -> Result<Vec<Project>, String> {
        let conn = DB::connect().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT * FROM projects ")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

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
            .map_err(|e| format!("Failed to execute query: {}", e))?;

        Ok(result)
    }

    pub fn add(name: String, toml: String) -> Result<i32, String> {
        let conn = DB::connect().map_err(|_| "Failed to connect to the database.".to_string())?;

        let mut stmt = conn
            .prepare(&format!(
                "
                INSERT INTO projects(name, toml)
                VALUES (?, ?)"
            ))
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        stmt.execute(
            [name, toml]
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid().try_into().unwrap())
    }

    pub fn remove(id: i32) -> Result<(), String> {
        let mut conn = DB::connect().map_err(|e| e.to_string())?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        tx
            .execute("DELETE FROM project WHERE id = ?", [id])
            .map_err(|e| e.to_string())?;

        tx
            .commit()
            .map_err(|e| e.to_string())
    }

    pub fn get_id(name: String) -> Result<i32, String> {
        let conn = DB::connect().map_err(|e| e.to_string())?;

        let id: i32 = conn.query_row(
        "SELECT p.id 
            FROM projects p
            WHERE name = ?
            OR ? in (SELECT a.alias FROM alias aWHERE p.id = a.id)", 
            [&name, &name], 
            |row| row.get(0)
            )
            .map_err(|e| e.to_string())?;

        Ok(id)
    }
}

pub struct Alias {
    pub id: i32,
    pub alias: String,
}

impl Alias {
    pub fn check(alias: String) -> Result<bool, String> {
        let conn = DB::connect().map_err(|e| e.to_string())?;

        let result: bool = conn.query_row(
        "SELECT EXISTS (
                SELECT 1
                FROM alias
                WHERE alias = ?
                )", 
            [alias], 
            |row| row.get(0))
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

        Ok(result)
    }

    pub fn get(id: i32) -> Result<String, String> {
        let conn = DB::connect().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT alias FROM alias WHERE id = ?")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let result: String = stmt
            .query_map([id], |row| {
                row.get(0) 
            }).unwrap()
            .collect::<Result<Vec<String>, _>>() 
            .map_err(|e| format!("Failed to execute query: {}", e))?
            .join(", ");

        Ok(result)
    }

    pub fn get_all() -> Result<Vec<Alias>, String> {
        let conn = DB::connect().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT * FROM alias ")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

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
            .map_err(|e| format!("Failed to execute query: {}", e))?;

        Ok(result)
    }

    pub fn add_all(id: i32, aliases: Vec<String>) -> Result<(), String> {
        let mut conn = DB::connect().map_err(|e| e.to_string())?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        for alias in aliases {
            tx.execute(
            "INSERT INTO alias (id, alias) 
                VALUES(?, ?) ", 
                params![id, alias]
            )
            .map_err(|e| e.to_string())?;
        }

        tx
        .commit()
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn add(id: i32, alias: String) -> Result<(), String> {
        let mut conn = DB::connect().map_err(|e| e.to_string())?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT INTO alias (id, alias) 
                VALUES(?, ?) ", 
                params![id, alias]
            )
            .map_err(|e| e.to_string())?;

        tx
        .commit()
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn remove(alias: String) -> Result<(), String>{
        let mut conn = DB::connect().map_err(|e| e.to_string())?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        tx.execute(
            "REMOVE FROM alias WHERE alias = ?", 
                params![alias]
            )
            .map_err(|e| e.to_string())?;

        tx
        .commit()
        .map_err(|e| e.to_string())?;

        Ok(())

    }
}