use clap::{Parser, Subcommand, ValueEnum};
use std::{
    fs::OpenOptions, io::Read, path::{PathBuf, Path}, fmt,
    io::Write,
};
use indexmap::IndexMap;
use crate::{
    repository::Repository,
    object::Object
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
    /// Checkout a commit inside a directory
    Checkout {
        /// The commit/tree to checkout on
        #[arg()]
        commit: String, 
        /// The **empty** directory to checkout on
        #[arg()]
        directory: String, 
    },
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
    /// Commit history 
    Log {
        /// The commit to start at.
        #[arg(default_value_t=String::from("HEAD"))]
        commit: String
    },
    LsFile {},
    /// Print a tree object
    LsTree {
        /// Recurse into subtrees
        #[arg()]
        recursive: bool,
        /// A tree-ish object
        #[arg()]
        tree: String, 
    },
    /// Retrieve the 
    RevParse {
        /// The expected type
        #[arg(short)]
        r#type: Option<Format>,
        /// The name to parse
        #[arg(short)]
        name: String,
    },
    Rm {},
    /// List references
    ShowRef {},
    Status {},
    /// Create a tag
    Tag {
        /// Whether to create a tag object
        #[arg(short)]
        add: bool, 

        /// The tag's name
        #[arg()]
        name: Option<String>, 

        /// The object the new tag will point to
        #[arg(default_value_t=String::from("HEAD"))]
        object: String, 
    },
}

#[derive(Debug, Clone, ValueEnum)]
#[derive(PartialEq)]
pub enum Format {
    Blob,
    Tree,
    Tag,
    Commit
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Format::Blob => write!(f, "blob"),
            Format::Tree => write!(f, "tree"),
            Format::Tag => write!(f, "tag"),
            Format::Commit => write!(f, "commit"),
        }
    }
}


pub fn init(path: &Path) {
    match Repository::create(path) {
        Ok(..) => {},
        Err(err) => print!("Error: {:#?}\n", err)
    }
}

// - If we have a tag and fmt is anything else, we follow the tag.
// - If we have a commit and fmt is tree, we return this commit’s tree object
// - In all other situations, we bail out: nothing else makes sense.
pub fn checkout(commit: &str, directory: &str, path: Option<&str>) {
    let path = path.as_ref().map_or(".", |p| p);
    let repo = Repository::find(&Path::new(path), true).unwrap().unwrap();
    let mut object = repo.read_object(&repo.find_object(commit, Some(Format::Commit), false).unwrap()).unwrap();
    match object {
        Object::Blob(_) => panic!("commit is not a tree or commit: {}", commit),
        Object::Commit(map) => {
            let tree = map.get("tree").unwrap()[0].as_str();
            object = repo.read_object(tree).unwrap();
        },
        _ => {}
    }
    _ = object;
    let target = Path::new(directory);
    if target.exists() {
        if !target.is_dir() {
            panic!("directory {} must be a directory", directory);
        } 
        if target.read_dir().unwrap().count() > 0 {
            panic!("path {} is non-empty", directory);
        }
    }

    checkout_tree(&repo, object, Path::new(directory).to_path_buf());
}

pub fn checkout_tree(repo: &Repository, object: Object, directory: PathBuf) {
    // if blob, make file
    // if tree, mkdir and recurse
    if let Object::Tree(leaves) = object {
        leaves.iter().for_each(|leaf| {
            let object = repo.read_object(&leaf.sha).unwrap();
            let dest = directory.join(Path::new(&leaf.path));
            match object {
                Object::Tree(_) => {
                    std::fs::create_dir(&dest).unwrap();
                    checkout_tree(repo, object, dest);
                },
                Object::Blob(data) => {
                    // NOTE: handle symlinks
                    let mut file = OpenOptions::new().write(true).open(dest).unwrap();
                    _ = file.write(&data[..]);
                },
                _ => panic!("The object in the tree is neither a blob nor a tree.")
            } 

        }); 
    } else {
        panic!("Object is not a tree");
    }
}

pub fn cat_file(fmt: Format, object: &str, path: Option<&str>) -> String {
    let path = path.as_ref().map_or(".", |p| p);
    let repo = match Repository::find(&Path::new(path), true) {
        Err(err) => panic!("{:#?}", err),
        Ok(Some(repo)) => repo,
        Ok(None) => unreachable!(),
    };
    // TODO: refactor below two errors
    let object = repo.read_object(&repo.find_object(&object, Some(fmt), true).unwrap()).unwrap();
    String::from_utf8(object.serialize().unwrap().clone()).unwrap()
}

pub fn hash_object(fmt: Format, write: bool, path: &str) -> String {
    let path = Path::new(&path);
    let mut file = OpenOptions::new().read(true).open(path).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let object = match fmt {
        Format::Blob => Object::new(Format::Blob, buf),
        Format::Tag | Format::Tree | Format::Commit => unimplemented!(),
    }.unwrap();
    if write {
        // TODO: handle unwraps
        let repo = Repository::find(path, true).unwrap().unwrap();
        let sha = repo.write_object(object).unwrap();
        sha
    } else {
        let (sha, _) = object.write().unwrap();
        sha
    }
}

// TODO: implement `log` after I finish implementing commit!
// Need to parse object to a map. Then print the information and recurse
// If there is a parent, parse the commit object and append to the builder string 
// If there is no parent, return
pub fn log(commit: &str, path: Option<&str>) -> String {
    let path = path.as_ref().map_or(".", |p| p);
    let repo = match Repository::find(&Path::new(path), true) {
        Err(err) => panic!("{:#?}", err),
        Ok(Some(repo)) => repo,
        Ok(None) => unreachable!(),
    };
    // TODO: throw error when commit is *not* a commit
    let object = repo.read_object(&repo.find_object(&commit, Some(Format::Commit), true).unwrap()).unwrap();
    String::from_utf8(object.serialize().unwrap().clone()).unwrap()
    
}

pub fn ls_tree(recursive: bool, tree: &str, path: Option<String>, prefix: PathBuf) {
    let path = path.as_ref().map_or(".", |p| p);
    let repo = match Repository::find(&Path::new(path), true) {
        Err(err) => panic!("{:#?}", err),
        Ok(Some(repo)) => repo,
        Ok(None) => unreachable!(),
    };
    let sha = repo.find_object(&tree, Some(Format::Tree), false).unwrap();
    let object = repo.read_object(&sha).unwrap();
    match object {
        Object::Tree(items) => {
            items.into_iter().for_each(|leaf| {
                let object_type = leaf.get_type();
                if !recursive && object_type == tree {
                    println!("{} {} {}\t{}", leaf.mode, object_type, leaf.sha, prefix.join(&leaf.path).to_str().unwrap());
                } else {
                    ls_tree(recursive, tree, Some(path.to_string()), prefix.join(&leaf.path))
                }
            });
        },
        _ => {
            panic!("[tree] arg {} is not a tree", &tree);
        }
    }
}

pub fn show_ref(path: Option<String>) {
    let path = path.as_ref().map_or(".", |p| p);
    let repo = match Repository::find(&Path::new(path), true) {
        Err(err) => panic!("{:#?}", err),
        Ok(Some(repo)) => repo,
        Ok(None) => unreachable!(),
    };
    let map = repo.ls_ref(None).unwrap();
    map.iter().for_each(|(k, v)| println!("{} {}", k, v));
}

pub fn rev_parse(fmt: Option<Format>, name: String, path: Option<String>) -> String {
    let path = path.as_ref().map_or(".", |p| p);
    let repo = match Repository::find(&Path::new(path), true) {
        Err(err) => panic!("{:#?}", err),
        Ok(Some(repo)) => repo,
        Ok(None) => unreachable!(),
    };
    repo.find_object(&name, fmt, true).unwrap()
}

pub fn tag(add: bool, name: Option<String>, object: String, path: Option<String>) {
    let path = path.as_ref().map_or(".", |p| p);
    let repo = match Repository::find(&Path::new(path), true) {
        Err(err) => panic!("{:#?}", err),
        Ok(Some(repo)) => repo,
        Ok(None) => unreachable!(),
    };
    if let Some(name) = name {
        let _ = repo.create_tag(&name, &object, add);
    } else {
        repo.ls_ref(None)
            .unwrap()
            .into_iter()
            .filter(|(k, v)| k.starts_with("tags"))
            .for_each(|(k, v)| println!("{} {}", k, v));
    }
}
