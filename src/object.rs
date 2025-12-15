use std::fmt;
use sha2::{Sha256, Digest};
use clap::ValueEnum;
use crate::error::Result;

#[derive(Debug, Clone, ValueEnum)]
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

pub struct Object {
    format: Format,
    data: Option<Vec<u8>>,
}

impl Object {
    pub fn serialize<'a>(&'a self) -> Option<&'a Vec<u8>> {
        match self.format {
            Format::Blob => self.data.as_ref(),
            Format::Tree => unimplemented!(),
            Format::Tag => unimplemented!(),
            Format::Commit => unimplemented!(),
        }
    }
    
    pub fn deserialize(&mut self, data: Option<Vec<u8>>) {
        match self.format {
            Format::Blob => self.data = data,
            Format::Tree => unimplemented!(),
            Format::Tag => unimplemented!(),
            Format::Commit => unimplemented!(),
        }
    }

    pub fn new(format: Format, data: Option<Vec<u8>>) -> Self {
        match format {
            Format::Blob => Object{ format, data} ,
            Format::Tree => unimplemented!(),
            Format::Tag => unimplemented!(),
            Format::Commit => unimplemented!(),
        }
    }

    pub fn write(&self) -> Result<(String, String)> {
        let data = str::from_utf8(self.serialize().unwrap())?;
        let size = format!("{}", data.len());
        let result = self.format.to_string() + " " + &size + "\x00" + data;
        let sha = Sha256::digest(&result);
        Ok((format!("{:x}", &sha), result))
    }
}
