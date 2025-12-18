use std::fmt;
use sha2::{Sha256, Digest};
use clap::ValueEnum;
use crate::error::Result;
use indexmap::IndexMap;
use itertools::Itertools;

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


pub trait Object {
    fn serialize(&self) -> Option<Vec<u8>>;
    
    fn deserialize(&mut self, data: Vec<u8>);

    fn format(&self) -> Format;

    fn write(&self) -> Result<(String, String)> {
        let raw = self.serialize().unwrap();
        let data = str::from_utf8(&raw[..])?;
        let size = format!("{}", data.len());
        let result = format!("{}", self.format()) + " " + &size + "\x00" + data;
        let sha = Sha256::digest(&result);
        Ok((format!("{:x}", &sha), result))
    }
}

pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        Blob { data } 
    }
}

impl Object for Blob {
    fn serialize(&self) -> Option<Vec<u8>> {
        Some(self.data.clone())
    }
    
    fn deserialize(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    fn format(&self) -> Format {
        Format::Blob
    }
}

pub struct Commit {
    map: IndexMap<String, Vec<String>>,
}

impl Object for Commit {
    fn serialize(&self) -> Option<Vec<u8>> {
        Some(key_value_serialize(&self.map).as_str().as_bytes().to_vec())
    }
    
    fn deserialize(&mut self, data: Vec<u8>) {
        self.map = key_value_parse(str::from_utf8(&data).unwrap());
    }

    fn format(&self) -> Format {
        Format::Commit
    }
}

/// TODO:
fn key_value_serialize(map: &IndexMap<String, Vec<String>>) -> String {
    map.iter().map(|(k, v)| {
        if k == "message" {
            return String::from("\n") + &v.join("");
        } 
        v.iter()
            .map(|v| k.clone() + " " + v)
            .collect::<Vec<String>>()
            .join("\n")
    }).join("\n")
}

/// TODO:
fn key_value_parse(raw: &str) -> IndexMap<String, Vec<String>> {
    let mut map: IndexMap<String, Vec<String>> = IndexMap::new();
    raw.split("\n").into_iter()
        .map(|x| x.to_string())
        .coalesce(|x, y| {
            if let Some(ch) = y.get(..1) {
                if ch == " " {
                    Ok(x + "\n" + &y[1..])
                } else {
                    Err((x, y))
                }
            } else {
                Err((x, y))
            }
        })
    .coalesce(|x, y| {
        if x == "" {
            Ok(String::from("message ") + &y)
        } else {
            Err((x, y))
        }
    })
    .for_each(|line| {
        let space_idx = line.find(" ").expect("Tag or commit is incorrectly formatted");
        let key = String::from(&line[..space_idx]);
        let value = String::from(&line[space_idx+1..]);
        if let Some(val) = map.get_mut(&key) {
            val.push(value);
        } else {
            map.insert(key, vec![value]);
        }
    });
    map
}

