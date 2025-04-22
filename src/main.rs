mod request;
mod environment;
mod db;
mod editor;
mod errors;

use clap::Parser;
use anyhow::Error;

use request::Request;
use environment::EnvironmentBuilder;


fn main() -> Result<(), Error>{
    //  Collect args into request
    let request = Request::parse();

    // Create environment from request
    let environment = EnvironmentBuilder::new(request)?;

    // Run process with options on environment
    environment.run()
}
