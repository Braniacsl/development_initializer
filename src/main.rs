use clap::Parser;
use std::io;
use std::fs;
use serde::Deserialize;
use std::collections::HashMap;
use std::process;

/// A customizable tool which gets you straight to developing
#[derive(Parser, Debug)]
#[command(name="development_initializer", version, about, long_about=None)]
struct Args {
    #[arg(default_value_t = String::new())]
    project_name: String,
    #[arg(short, long, default_value_t = false)]
    add: bool
}

#[derive(Debug, Deserialize)]
struct LocalConfig {
    project_aliases: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ProjectConfig {
    commands: Vec<String>
}

const LOCAL_CONFIG_PATH : &str = "/.config/development_initializer/config.toml";
const PROJECTS_PATH: &str = "/.config/development_initializer/projects/";

fn main() {
    let args: Args = Args::parse();

    let user_home = match std::env::var("HOME") {
        Ok(path) => path,
        Err(_) => {
            println!("$HOME environment variable not set.");
            process::exit(1);
        }
    };

    let local_config_path: &str = &(user_home.clone() + LOCAL_CONFIG_PATH)[..];
    let projects_path: &str = &(user_home + PROJECTS_PATH)[..];

    //Read configs/make folders if not present and check for errors in configs
    let config: String = match read_or_create_file(local_config_path){
        Ok(contents) => contents,
        Err(e) => {
            println!("Unkown Error: {e}");
            process::exit(0);
        }
    };
    
    let config: LocalConfig = match toml::from_str(&config[..]) {
        Ok(content) => content,
        Err(e) => {
            println!("Unable to parse file({}), {}", local_config_path, e);
            process::exit(0);
        },
    };
    
    let path_aliases = invert_alias(config.project_aliases);

    //Read projects toml and parse contents
    let mut project_config_path = match path_aliases.get(&args.project_name) {
        Some(path) => path.clone(),
        None => {
            println!("Project alias not found.");
            process::exit(0);
        }
    };

    project_config_path.insert_str(0, projects_path);
    project_config_path = project_config_path + ".toml";

    let project = match read_or_create_file(&project_config_path[..]){
        Ok(contents) => contents,
        Err(e) => {
            println!("Unkown Error: {e}");
            process::exit(0);
        }
    };

    let project: ProjectConfig = match toml::from_str(&project[..]) {
        Ok(content) => content,
        Err(e) => {
            println!("Unable to parse file({}), {}", projects_path, e);
            process::exit(0);
        },
    };

    //Run commands 
    for command in project.commands {
        println!("EXEC:{command}");
    }

}

fn invert_alias(project_aliases: HashMap<String, String>) -> HashMap<String, String> {
    let mut inverted_alias: HashMap<String, String> = HashMap::new();

    for (path, aliases) in project_aliases {
        for alias in aliases.split(',').map(str::trim) {
            inverted_alias.insert(alias.to_string(), path.to_string());
        }
    }

    inverted_alias
}

fn read_or_create_file(path: &str) -> io::Result<String>{
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(e) if e.kind() == io::ErrorKind::NotFound => handle_not_found(path),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => handle_permission_denied(path),
        Err(e) => Err(e),
    }
}

fn handle_not_found(path: &str) -> io::Result<String> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
        fs::File::create(path)?;
    }

    Ok(String::new())
}

fn handle_permission_denied(path: &str) -> io::Result<String> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if parent.exists() {
            println!("You do not have permission to one of the directories in {path}")
        }
        else {
            println!("You do not have permission to access the config file in {path}")
        }
        process::exit(0);
    }

    Ok(String::new())
}