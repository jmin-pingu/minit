use std::{
    path::Path
};
use clap::{Parser};
use minit::{
    cli::{Cli, Commands},
    cli
};

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Init { path } => cli::init(Path::new(&path)),
        Commands::CatFile { r#type, object } => println!("{:#?}", cli::cat_file(r#type, object)),
        Commands::HashObject { r#type, write, path } => println!("{:#?}", cli::hash_object(r#type, write, path)),
        Commands::Log { commit } => println!("{:#?}", cli::log(commit)),
        _ => {},
    }
}

