use minit::repository::Repository;
use minit::error::Result;
use minit::object::Format;
use minit::cli;
use flate2::read::ZlibDecoder;
use std::{
    io::{Write, Read},
    fs,
    fs::OpenOptions,
    path::{Path}
};
use configparser::ini::Ini;


#[test]
fn test_init() {
    let path = Path::new("snapshots/init");
    if path.exists() && path.is_dir() {
        _ = fs::remove_dir_all(path);
    }
    _ = Repository::create(path).unwrap();
    let mut builder = String::from("");
    let tree = ls_tree(path, &mut builder);
    insta::assert_snapshot!("init directory tree", tree.unwrap());

    assert!(Repository::find(path, false).unwrap().is_some());

    let falsy_path = Path::new(".");
    assert!(Repository::find(falsy_path, false).unwrap().is_none());
}

#[test]
pub fn test_cat_file() {}

#[test]
fn test_full_functionality() {
    let path = Path::new("snapshots/working_dir");
    if path.exists() && path.is_dir() {
        _ = fs::remove_dir_all(path);
    }
    _ = Repository::create(path).unwrap();
    let mut builder = String::from("");
    let p1 = "snapshots/working_dir/helloworld.txt";
    let mut f1 = OpenOptions::new().create(true).write(true).open(p1).unwrap();
    f1.write(b"helloworld").unwrap();

    let p2 = "snapshots/working_dir/foobar.txt";
    let mut f2 = OpenOptions::new().create(true).write(true).open(p2).unwrap();
    f2.write(b"foobar").unwrap();

    cli::hash_object(Format::Blob, true, String::from(p1));
    cli::hash_object(Format::Blob, true, String::from(p2));

    let tree = ls_tree(path, &mut builder);
    insta::assert_snapshot!("hash_object directory tree", tree.unwrap());
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
        println!("{:#?}", path);
        *builder = builder.clone() + "directory: " + path.to_str().unwrap() + "\n\n";
        for path in path.read_dir()? {
            let new_path = path?.path();
            *builder = ls_tree(&new_path, builder)?.clone();
        }
        return Ok(builder);
    }
    if path.is_file() {
        println!("{:#?}", path);
        if path.file_name().unwrap() == "config" {
            let content = standardize_config(path);
            *builder = builder.clone() + "file: " +
                path.to_str().unwrap() + "\n" +
                ">>>>>\n" +
                &content + 
                "\n<<<<<\n\n";
        } else {
            let content = match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(..) => {
                    let mut file = OpenOptions::new().read(true).open(&path)?;
                    let mut buf: Vec<u8> = Vec::new();
                    file.read_to_end(&mut buf)?;

                    let mut decoder = ZlibDecoder::new(&buf[..]);
                    let mut data: Vec<u8> = Vec::new();
                    decoder.read_to_end(&mut data)?;
                    format!("{:?}", String::from_utf8(data)?)
                },
            };
           *builder = builder.clone() + "file: " +
                path.to_str().unwrap() + "\n" +
                ">>>>>\n" +
                &content + 
                "\n<<<<<\n\n";
        }
        return Ok(builder);
    }
    
    return Ok(builder)
} 

