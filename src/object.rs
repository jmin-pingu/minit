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

pub struct Object {
    format: Format,
    data: Option<Vec<u8>>,
    map: Option<IndexMap<String, Vec<String>>>,
}

impl Object {
    pub fn serialize(&self) -> Option<Vec<u8>> {
        match self.format {
            Format::Blob => self.data.clone(),
            Format::Tree => unimplemented!(),
            Format::Tag => unimplemented!(),
            Format::Commit => Some(
                Object::key_value_serialize(
                    self.map.as_ref().unwrap()
                ).as_str().as_bytes().to_vec()
                ),
        }
    }
    
    pub fn deserialize(&mut self, data: Option<Vec<u8>>) {
        match self.format {
            Format::Blob => self.data = data,
            Format::Tree => unimplemented!(),
            Format::Tag => unimplemented!(),
            Format::Commit => self.map = Some(Object::key_value_parse(str::from_utf8(self.data.as_ref().unwrap()).unwrap())),
        }
    }

    pub fn new(format: Format, data: Option<Vec<u8>>) -> Self {
        match format {
            Format::Blob => Object{ format, data, map: None } ,
            Format::Tree => unimplemented!(),
            Format::Tag => unimplemented!(),
            Format::Commit => Object { format, data, map: None},
        }
    }

    pub fn write(&self) -> Result<(String, String)> {
        let raw = self.serialize().unwrap();
        let data = str::from_utf8(&raw[..])?;
        let size = format!("{}", data.len());
        let result = self.format.to_string() + " " + &size + "\x00" + data;
        let sha = Sha256::digest(&result);
        Ok((format!("{:x}", &sha), result))
    }

    pub fn key_value_serialize(map: &IndexMap<String, Vec<String>>) -> String {
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

    pub fn key_value_parse(raw: &str) -> IndexMap<String, Vec<String>> {
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
}
