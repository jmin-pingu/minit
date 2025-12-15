use minit::repository::Repository;
use minit::error::Result;
use std::{
    path::Path,
    fs
};
use configparser::ini::Ini;

#[test]
pub fn test_init() {
    let path = Path::new("snapshots/init");
    if path.exists() && path.is_dir() {
        _ = fs::remove_dir_all(path);
    }
    _ = Repository::create(path).unwrap();
    let mut builder = String::from("");
    let tree = ls_tree(path, &mut builder);
    insta::assert_snapshot!("{:#?}", tree.unwrap());

    assert!(Repository::find(path, false).unwrap().is_some());

    let falsy_path = Path::new(".");
    assert!(Repository::find(falsy_path, false).unwrap().is_none());
}

#[test]
pub fn test_cat_file() {
    let path = Path::new("snapshots/working_dir");
    if path.exists() && path.is_dir() {
        _ = fs::remove_dir_all(path);
    }
    _ = Repository::create(path).unwrap();
    let mut builder = String::from("");
    let tree = ls_tree(path, &mut builder);
    insta::assert_snapshot!("{:#?}", tree.unwrap());
}

#[test]
pub fn test_hash_object() {

}

#[test]
pub fn test_rm() {

}

#[cfg(test)]
fn standardize_config(path: &Path) -> String {
    Ini::new()
        .load(path)
        .unwrap()
        .into_iter()
        .map(|(k, subsection)| {
            let mut key_values = subsection.into_iter()
                .map(|(k, v)| format!("{}: {}", k, v.unwrap_or(String::from(""))))
                .collect::<Vec<String>>();
            key_values.sort_by(|a, b| a.cmp(b));
            format!("[{}]\n{}", k, key_values.join("\n"))
        })
    .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
fn ls_tree<'a>(path: &Path, builder: &'a mut String) -> Result<&'a mut String> {
    if path.is_dir() {
        *builder = builder.clone() + "directory: " + path.to_str().unwrap() + "\n\n";
        for path in path.read_dir()? {
            let new_path = path?.path();
            *builder = ls_tree(&new_path, builder)?.clone();
        }
        return Ok(builder);
    }
    if path.is_file() {
        if path.file_name().unwrap() == "config" {
            let content = standardize_config(path);
            *builder = builder.clone() + "file: " +
                path.to_str().unwrap() + "\n" +
                ">>>>>\n" +
                &content + 
                "\n<<<<<\n\n";
        } else {
            let content = std::fs::read_to_string(&path)?;
            *builder = builder.clone() + "file: " +
                path.to_str().unwrap() + "\n" +
                ">>>>>\n" +
                &content + 
                "<<<<<\n\n";
        }
        return Ok(builder);
    }
    
    return Ok(builder)
} 

