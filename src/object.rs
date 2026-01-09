use sha2::{Sha256, Digest};
use crate::error::{Error,Result};
use crate::cli::Format;
use indexmap::IndexMap;
use itertools::Itertools;

pub enum Object {
    Blob(Vec<u8>),
    Commit(IndexMap<String, Vec<String>>),
    Tag(IndexMap<String, Vec<String>>),
    Tree(Vec<Leaf>),
}

#[derive(Clone)]
pub struct Leaf {
    pub mode: String, 
    pub path: String,
    pub sha: String
}

impl Leaf {
    pub fn new(mode: String, path: String, sha: String) -> Self {
        Leaf{ mode, path, sha }
    }

    pub fn parse_line(data: &str) -> Result<Self> {
        let row = data;
        let space_idx = row.find(" ")
            .ok_or(Error::StringNotFound(data.to_string(), " ".to_string()))?;
        let null_idx = row.find("\x00")
            .ok_or(Error::StringNotFound(data.to_string(), "\x00".to_string()))?;

        let mode = row[0..space_idx].to_string();
        let path = row[space_idx+1..null_idx].to_string();
        let sha = row[null_idx+1..].to_string();
        Ok(Leaf::new(mode, path, sha))
    }

    pub fn remap_dirs(&self) -> String {
        if self.mode.starts_with("10") {
            self.path.clone()
        } else {
            self.path.clone() + "/"
        }
    }

    pub fn get_type(&self) -> &str {
        match &self.mode[0..2] {
            "10" | "12" => "blob",
            "04" => "tree",
            "16" => "commit",
            other => panic!("Invalid leaf mode {}", other)
        }
    }
}

impl Object {
    pub fn new(format: Format, data: Vec<u8>) -> Result<Self> {
        match format {
            Format::Blob => Ok(Object::Blob(data)),
            Format::Commit => {
                Ok(Object::Commit(key_value_parse(str::from_utf8(&data)?)))
            },
            Format::Tag => {
                Ok(Object::Tag(key_value_parse(str::from_utf8(&data)?)))
            },
            Format::Tree => {
                let tree = str::from_utf8(&data)?
                    .split("\n")
                    .map(|line| Leaf::parse_line(line))
                    .collect::<Result<Vec<Leaf>>>()?;
                Ok(Object::Tree(tree))
            },
        } 
    }

    pub fn serialize(&self) -> Option<Vec<u8>> {
        match self {
            Object::Blob(data) => Some(data.clone()),
            Object::Tag(map) | Object::Commit(map) => Some(key_value_serialize(map)) ,
            Object::Tree(tree) => {
                let mut cloned_tree = tree.clone();
                cloned_tree.sort_by_key(|leaf| leaf.remap_dirs());
                let serialized_tree = tree.into_iter() 
                    .map(|leaf| leaf.mode.clone() + " " + &leaf.path + "\x00" + &leaf.sha)
                    .join("\n")
                    .as_bytes()
                    .to_vec();
                Some(serialized_tree)
            }

        }
    }
    
    pub fn deserialize(&mut self, data: Vec<u8>) -> Result<()>{
        match self {
            Object::Blob(curr_data) => {
                *curr_data = data;            
            },
            Object::Tag(map) | Object::Commit(map) => {
                *map = key_value_parse(str::from_utf8(&data[..])?);
            },
            Object::Tree(tree) => {
                *tree = str::from_utf8(&data)?
                    .split("\n")
                    .map(|line| Leaf::parse_line(line))
                    .collect::<Result<Vec<Leaf>>>()?;
            },
        }

        Ok(())
    }

    pub fn format(&self) -> Format {
        match self {
            Object::Blob(_) => Format::Blob,
            Object::Commit(_) => Format::Commit,
            Object::Tag(_) => Format::Tag,
            Object::Tree(_) => Format::Tree,
        }
    }

    pub fn write(&self) -> Result<(String, String)> {
        let raw = self.serialize().unwrap();
        let data = str::from_utf8(&raw[..])?;
        let size = format!("{}", data.len());
        let result = format!("{}", self.format()) + " " + &size + "\x00" + data;
        let sha = Sha256::digest(&result);
        Ok((format!("{:x}", &sha), result))
    }
}

/// TODO:
fn key_value_serialize(map: &IndexMap<String, Vec<String>>) -> Vec<u8> {
    map.iter().map(|(k, v)| {
        if k == "message" {
            return String::from("\n") + &v.join("");
        } 
        v.iter()
            .map(|v| k.clone() + " " + v)
            .collect::<Vec<String>>()
            .join("\n")
    }).join("\n").as_bytes().to_vec()
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

