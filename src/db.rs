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
            .execute("DELETE FROM project WHERE id = ?", [id])?;

        Ok(tx.commit()?)
    }

    pub fn get_id(name: String) -> Result<i32> {
        let conn = DB::connect()?;

        let id: i32 = conn.query_row(
        "SELECT p.id 
            FROM projects p
            WHERE name = ?
            OR ? in (SELECT a.alias FROM alias aWHERE p.id = a.id)", 
            [&name, &name], 
            |row| row.get(0)
            )?;

        Ok(id)
    }
}

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