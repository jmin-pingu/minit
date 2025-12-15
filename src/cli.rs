use clap::{Parser, Subcommand};
use crate::object::Format;

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
