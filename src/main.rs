use std::{
    fs::OpenOptions, io::Read, path::Path
};
use clap::{Parser};
use minit::{
    cli::{Cli, Commands},
    repository::Repository,
    object::{Format, Object},
};


fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Init { path } => init(Path::new(&path)),
        Commands::CatFile { r#type, object } => cat_file(r#type, object),
        Commands::HashObject { r#type, write, path } => hash_object(r#type, write, path),
        _ => {},
    }
}


fn init(path: &Path) {
    match Repository::create(path) {
        Ok(..) => {},
        Err(err) => print!("Error: {:#?}\n", err)
    }
}

fn cat_file(fmt: Format, object: String) {
    let repo = match Repository::find(&Path::new("."), true) {
        Err(err) => panic!("{:#?}", err),
        Ok(Some(repo)) => repo,
        Ok(None) => unreachable!(),
    };
    let object = repo.read_object(repo.find_object(object, fmt, true).unwrap()).unwrap();
    println!("{:#?}", object.serialize());
}

fn hash_object(fmt: Format, write: bool, path: String) {
    let path = Path::new(&path);
    let mut file = OpenOptions::new().read(true).open(path).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let object = match fmt {
        Format::Blob => Object::new(fmt, Some(buf)),
        Format::Tag | Format::Tree | Format::Commit => unimplemented!(),
    };
    if write {
        let repo = Repository::find(path, false).unwrap().unwrap();
        let sha = repo.write_object(object).unwrap();
        println!("{}", sha);
    } else {
        let (sha, _) = object.write().unwrap();
        println!("{}", sha);
    }
}
