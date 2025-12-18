use clap::{Parser, Subcommand};
use crate::object::Format;
use std::{
    fs::OpenOptions, io::Read, path::Path
};
use crate::{
    repository::Repository,
    object::Object,
};

/// Minit. A bare bones version control system
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
    Add {},
    /// Provide contents of repository objects
    CatFile {
        /// The object to display
        #[arg(required=true)]
        object: String,
        /// Specify the type of object
        #[arg(default_value_t=Format::Blob)]
        r#type: Format,
    },
    CheckIgnore {},
    Checkout {},
    Commit {},
    /// Compute the object ID and optionally create a blob from a file
    HashObject {
        /// Flag to write the object to the database
        #[arg(short)]
        write: bool,
        /// Read object from <file>
        #[arg(required=true)]
        path: String,
        /// Specify the type of object
        #[arg(short, default_value_t=Format::Blob)]
        r#type: Format,
    },
    /// Initialize a new, empty minit repository.
    Init {
        /// Where to create the repository
        #[arg(default_value_t=String::from("."))]
        path: String
    },
    Log {},
    LsFile {},
    LsTree {},
    RevParse {},
    Rm {},
    ShowRef {},
    Status {},
    Tag {},
}

pub fn init(path: &Path) {
    match Repository::create(path) {
        Ok(..) => {},
        Err(err) => print!("Error: {:#?}\n", err)
    }
}

pub fn cat_file(fmt: Format, object: String) -> String {
    let repo = match Repository::find(&Path::new("."), true) {
        Err(err) => panic!("{:#?}", err),
        Ok(Some(repo)) => repo,
        Ok(None) => unreachable!(),
    };
    // TODO: refactor below two errors
    let object = repo.read_object(repo.find_object(object, fmt, true).unwrap()).unwrap();
    String::from_utf8(object.serialize().unwrap().clone()).unwrap()
}

pub fn hash_object(fmt: Format, write: bool, path: String) -> String {
    let path = Path::new(&path);
    let mut file = OpenOptions::new().read(true).open(path).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let object = match fmt {
        Format::Blob => Object::new(fmt, Some(buf)),
        Format::Tag | Format::Tree | Format::Commit => unimplemented!(),
    };
    if write {
        let repo = Repository::find(path, true).unwrap().unwrap();
        let sha = repo.write_object(object).unwrap();
        sha
    } else {
        let (sha, _) = object.write().unwrap();
        sha
    }
}
