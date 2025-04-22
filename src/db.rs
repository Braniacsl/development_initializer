use rusqlite::{Connection, params};
use directories::ProjectDirs;
use std::path::Path;
use anyhow::{Error, Result, anyhow};

pub struct Database {}

impl Database {
    fn connect() -> Result<Connection, Error> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "braniacs", "devinit"){
            let db_path = proj_dirs.data_dir().join("devinit.db");
        
            if !db_path.exists() {
                Self::init_db(&db_path)?;
            }

            Ok(Connection::open("devinit.db")?)
        }
        else {
            Err(anyhow!("Failed to Connect."))
        }
    }

    fn init_db(db_path: &Path) -> Result<()>{
        let conn = Connection::open(db_path)?;

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                toml TEXT NOT NULL
            );
    
            CREATE TABLE IF NOT EXISTS alias (
                project_id INTEGER NOT NULL, -- Foreign key referencing projects.id
                name TEXT NOT NULL,          -- Alias name
                is_primary BOOLEAN NOT NULL DEFAULT 0, -- Indicates if the alias is primary
                PRIMARY KEY (project_id, name),        -- Composite primary key
                UNIQUE (project_id, is_primary),       -- Ensure only one primary alias per project
                CHECK (is_primary IN (0, 1)),          -- Ensure is_primary is either 0 or 1
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );
            ",
            [],
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Alias {
    names: Vec<Box<str>>,
    id: i32,
}

impl std::fmt::Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let s = self.names.join(", ");

        write!(f, "{}", s)
    }
}

impl Alias {
    pub fn retrieve(name: Box<str>) -> Result<Self> {
        let conn = Database::connect()?;

        // Query all aliases matching the given name
        let mut stmt = conn.prepare(
            "SELECT * FROM alias WHERE id = (SELECT id FROM alias WHERE name = ?)",
        )?;

        let rows = stmt.query_map([name], |row| {
            Ok(AliasRow {
                project_id: row.get(0)?,
                name: row.get(1)?,
                is_primary: row.get(2)?,
            })
        })?;

        let mut names = Vec::new();
        let mut project_id: i32 = 0;

        for row in rows {
            let alias_row = row?;
            names.push(alias_row.name.into_boxed_str());
            project_id = alias_row.project_id;
        }

        Ok(Alias { 
            names, 
            id: project_id })
    }


    pub fn db_add_aliases(project_id: i32, aliases: Vec<Box<str>>) -> Result<()> {
        let mut conn = Database::connect()?;
        let tx = conn.transaction()?; // Begin a transaction

        // Check if the project exists
        let project_exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)",
            [&project_id],
            |row| row.get(0),
        )?;

        if !project_exists {
            return Err(anyhow!("Project with ID {} does not exist.", project_id));
        }

        // Determine if this is the first set of aliases for the project
        let alias_count: i32 = tx.query_row(
            "SELECT COUNT(*) FROM alias WHERE project_id = ?",
            [&project_id],
            |row| row.get(0),
        )?;

        for (index, name) in aliases.iter().enumerate() {
            let is_primary = alias_count == 0 && index == 0;

            // Insert the alias
            tx.execute(
                "INSERT INTO alias (project_id, name, is_primary) VALUES (?1, ?2, ?3)",
                params![project_id, name, is_primary],
            )?;
        }

        tx.commit()?; // Commit the transaction if everything succeeds

        Ok(())
    }


    pub fn db_add(&self, name: Box<str>) -> Result<()> {
        let mut conn = Database::connect()?;
        let tx = conn.transaction()?; // Begin a transaction

        // Check if the project exists
        let project_id: i32 = tx.query_row(
            "SELECT product_id FROM alias WHERE name = ?",
            [self.names.first()],
            |row| row.get(0),
        )?;

        // Insert the alias
        tx.execute(
            "INSERT INTO alias (project_id, name, is_primary) VALUES (?1, ?2, ?3)",
            params![project_id, name, false],
        )?;

        tx.commit()?; // Commit the transaction if everything succeeds

        Ok(())
    }

    pub fn db_remove(alias: Alias, name: &Box<str>) -> Result<()> {
        let mut conn = Database::connect()?;
        let tx = conn.transaction()?; // Begin a transaction
    
        // Find whether the alias exists and is primary
        let (exists, is_primary): (bool, bool) = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM alias WHERE project_id = ? AND name = ?),
                    is_primary
             FROM alias
             WHERE project_id = ? AND name = ?",
            params![alias.id, name, alias.id, name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
    
        if !exists {
            return Err(anyhow!("Alias '{}' not found in project {}.", name, alias.id));
        }
    
        if is_primary {
            // Check if there are other aliases in the project
            let alias_count: i32 = tx.query_row(
                "SELECT COUNT(*) FROM alias WHERE project_id = ?",
                [&alias.id],
                |row| row.get(0),
            )?;
    
            if alias_count > 1 {
                return Err(anyhow!(
                    "Cannot remove primary alias '{}'. Other aliases exist in the project.",
                    name
                ));
            }
        }
    
        // Remove the alias
        tx.execute(
            "DELETE FROM alias WHERE project_id = ? AND name = ?",
            params![alias.id, name],
        )?;
    
        tx.commit()?; // Commit the transaction if everything succeeds
    
        Ok(())
    }    
    

    pub fn db_remove_group(project_id: i32) -> Result<()> {
        let mut conn = Database::connect()?;
        let tx = conn.transaction()?; // Begin a transaction
    
        // Check if the project exists
        let project_exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?)",
            [&project_id],
            |row| row.get(0),
        )?;
    
        if !project_exists {
            return Err(anyhow!("Project with ID {} does not exist.", project_id));
        }
    
        // Remove all aliases for the project
        tx.execute("DELETE FROM alias WHERE project_id = ?", [&project_id])?;
    
        tx.commit()?; // Commit the transaction if everything succeeds
    
        Ok(())
    }

    pub fn get_primary_name_from_id(id: i32) -> Result<Box<str>> {
        let mut conn = Database::connect()?;

        let primary_name = conn.query_row(
            "SELECT name FROM alias WHERE id = ? AND is_primary = 1",
            [id],
            |row| row.get(0)
        )?;

        Ok(primary_name)
    }

    pub fn new() -> Self {
        Alias {
            names: Vec::new(),
            id: 0,
        }
    }

    pub fn from(str: Box<str>) -> Self {
        Alias {
            names: vec!(str),
            id: 0,
        }
    }

    pub fn add(&mut self, name: Box<str>) {
        self.names.push(name);
    }

    pub fn to_vec(&self) -> Vec<Box<str>> {
        self.names.clone()
    }
}

pub struct Projects {
    id: i32,
    name: Box<str>,
}

impl Projects {
    pub fn add(aliases: Alias, project_toml: &Box<str>) -> Result<()> {
        let mut conn = Database::connect()?;
        let tx = conn.transaction()?; // Begin a transaction

        // Insert the project into the projects table
        let project_id: i32 = tx.query_row(
            "INSERT INTO projects (toml) VALUES (?1) RETURNING id",
            params![*project_toml],
            |row| row.get(0),
        )?;

        tx.commit()?;

        Alias::db_add_aliases(project_id, aliases.to_vec())
    }

    pub fn remove(aliases: Alias) -> Result<()> {
        let project_id = Self::get_id(aliases)?;

        let mut conn = Database::connect()?;
        let tx = conn.transaction()?;

        tx.execute(
            "DELETE FROM projects WHERE id = ?", 
            params!(project_id)
        );

        tx.commit()?;

        Ok(())
    }

    pub fn get_id(alias: Alias) -> Result<i32> {
        let conn = Database::connect()?;

        let project_id: i32 = conn.query_row(
            "SELECT project_id FROM alias WHERE name = ?", 
            params!(alias.to_vec().first()), 
            |row| row.get(0),
        )?;

        Ok(project_id)
    }

    pub fn get_toml(alias: Alias) -> Result<Box<str>> {
        let project_id = Self::get_id(alias)?;

        let conn = Database::connect()?;

        let project: ProjectRow = conn.query_row(
            "SELECT * FROM projects WHERE id = ?", 
            params!(&project_id), 
            |row| Ok(ProjectRow {
                id: row.get(0)?,
                toml: row.get(1)?,
            })
        )?;

        Ok(project.toml.into_boxed_str())
    }

    pub fn get_all() -> Result<Projects>{
        let conn = Database::connect()?;
        
        let stmt = conn.prepare("SELECT * FROM projects")?;
        
        let projects = stmt
            .query_map(
                [], 
            |row| {
                let id = row.get(0)?;
                let name = Alias::get_primary_name_from_id(id)?;

                Ok(Projects {
                    id,
                    name
                })
            });

    }
}

#[derive(Debug)]
struct AliasRow {
    project_id: i32,
    name: String,
    is_primary: bool,
}

#[derive(Debug)]
struct ProjectRow {
    id: i32,
    toml: String,
}

#[derive(Debug)]
struct GeneralRow {
    editor: String,
}