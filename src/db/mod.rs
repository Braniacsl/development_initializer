use rusqlite::Connection;
use directories::ProjectDirs;
use std::{fs, path::Path};
use anyhow::{anyhow, Context, Result};

pub(super) mod alias;
pub(super) mod project;
pub(super) mod settings;

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
            let conn = Connection::open(&db_path).context("Failed to open database connection")?;

            Self::update_db(&conn, None)?;
            
            Ok(conn)
        } else {
            Err(anyhow!("Failed to find or create project directories."))
        }
    }

    fn update_db(conn: &Connection, version: Option<u32>) -> Result<bool> {
        let version: u32 = match version {
            Some(v) => v,
            None => conn.query_row(
                "PRAGMA user_version;", 
                [], 
                |row| row.get::<_, u32>(0))?,
        };
        

        match version {
            0 => {
                conn.execute("
                CREATE TABLE IF NOT EXISTS settings (
                    uwsm BOOLEAN NOT NULL DEFAULT 0 CHECK (uwsm IN (0, 1))
                );

                PRAGMA user_version = 1;
                ", [])?;

                conn.execute("
                INSERT INTO settings (uwsm) VALUES (0);
                ", [])?;

                Self::update_db(conn, Some(1))
            },
            num if num >= 1 => Ok(true),
            other => Err(anyhow!("No Database Version Env Set. Received {}", other))
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

        conn.execute("PRAGMA foreign_keys = ON;", []).unwrap();
        
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