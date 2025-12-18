use sha2::{Sha256, Digest};
use crate::error::Result;
use crate::cli::Format;
use indexmap::IndexMap;
use itertools::Itertools;

pub enum Object {
    Blob(Vec<u8>),
    Commit(IndexMap<String, Vec<String>>),
}

impl Object {
    pub fn new(format: Format, data: Vec<u8>) -> Self{
        match format {
            Format::Blob => Object::Blob(data),
            Format::Commit => Object::Commit(key_value_parse(str::from_utf8(&data).unwrap())),
            _ => unimplemented!()
        } 
    }

    pub fn serialize(&self) -> Option<Vec<u8>> {
        match self {
            Object::Blob(data) => Some(data.clone()),
            Object::Commit(map) => Some(key_value_serialize(map)) ,
            _ => unimplemented!()
        }
    }
    
    pub fn deserialize(&mut self, data: Vec<u8>) {
        match self {
            Object::Blob(curr_data) => {
                *curr_data = data;            
            },
            Object::Commit(map) => {
                *map = key_value_parse(str::from_utf8(&data[..]).unwrap());
            },
            _ => unimplemented!()
        }
    }

    pub fn format(&self) -> Format {
        match self {
            Object::Blob(_) => Format::Blob,
            Object::Commit(_) => Format::Commit,
            _ => unimplemented!()
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

