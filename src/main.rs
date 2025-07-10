use clap::Parser;
use ipak::modules::{messages, pkg, project, system, utils};
use ipak::utils::args::{Args, Commands};
use ipak::utils::error::Error;

fn main() -> Result<(), Error> {
    let args = Args::parse();

    match args.command {
        Commands::Manual => messages::manual()?,
        Commands::Project(args) => project::project(args)?,
        Commands::System(args) => system::system(args)?,
        Commands::Pkg(args) => pkg::pkg(args)?,
        Commands::Utils(args) => utils::utils(args)?,
    };
    Ok(()) 
}
