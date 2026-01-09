use minit::repository::Repository;
use minit::error::Result;
use minit::object::{
    Object
};
use minit::cli;
use minit::cli::Format;
use flate2::read::ZlibDecoder;
use std::{
    io::{Write, Read},
    fs,
    fs::OpenOptions,
    path::{Path}
};
use configparser::ini::Ini;
use indexmap::IndexMap;


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
fn test_full_functionality() {
    let commit_content = "tree 29ff16c9c14e2652b22f8b78bb08a5a07930c147
author Jonathan Min <test@gmail.com> 1527025023 +0200
committer Jonathan Min <test@gmail.com> 1527025044 +0200
gpgsig -----BEGIN PGP SIGNATURE-----
 iQIzBAABCAAdFiEExwXquOM8bWb4Q2zVGxM2FxoLkGQFAlsEjZQACgkQGxM2FxoL
 kGQdcBAAqPP+ln4nGDd2gETXjvOpOxLzIMEw4A9gU6CzWzm+oB8mEIKyaH0UFIPh
 rNUZ1j7/ZGFNeBDtT55LPdPIQw4KKlcf6kC8MPWP3qSu3xHqx12C5zyai2duFZUU
 wqOt9iCFCscFQYqKs3xsHI+ncQb+PGjVZA8+jPw7nrPIkeSXQV2aZb1E68wa2YIL
 3eYgTUKz34cB6tAq9YwHnZpyPx8UJCZGkshpJmgtZ3mCbtQaO17LoihnqPn4UOMr
 V75R/7FjSuPLS8NaZF4wfi52btXMSxO/u7GuoJkzJscP3p4qtwe6Rl9dc1XC8P7k
 NIbGZ5Yg5cEPcfmhgXFOhQZkD0yxcJqBUcoFpnp2vu5XJl2E5I/quIyVxUXi6O6c
 /obspcvace4wy8uO0bdVhc4nJ+Rla4InVSJaUaBeiHTW8kReSFYyMmDCzLjGIu1q
 doU61OM3Zv1ptsLu3gUE6GU27iWYj2RWN3e3HE4Sbd89IFwLXNdSuM0ifDLZk7AQ
 WBhRhipCCgZhkj9g2NEk7jRVslti1NdN5zoQLaJNqSwO1MtxTmJ15Ksk3QP6kfLB
 Q52UWybBzpaP9HEd4XnR+HuQ4k2K0ns2KgNImsNvIyFwbpMUyUWLMPimaV1DWUXo
 5SBjDB/V/W2JBFR+XKHFJeFwYhj7DD/ocsGr4ZMx/lgc8rjIBkI=
 =lgTX
 -----END PGP SIGNATURE-----

The first commit ever!";

    let repo_path = "snapshots/working_dir";
    let path = Path::new(repo_path);
    if path.exists() && path.is_dir() {
        _ = fs::remove_dir_all(path);
    }
    _ = Repository::create(path).unwrap();
    let mut builder = String::from("");
    let helloworld_path = "snapshots/working_dir/helloworld.txt";
    let mut helloworld_file = OpenOptions::new().create(true).write(true).open(helloworld_path).unwrap();
    helloworld_file.write(b"helloworld").unwrap();

    let foobar_path = "snapshots/working_dir/foobar.txt";
    let mut foobar_file = OpenOptions::new().create(true).write(true).open(foobar_path).unwrap();
    foobar_file.write(b"foobar").unwrap();

    let helloworld_path = "snapshots/working_dir/helloworld.txt";
    let mut helloworld_file = OpenOptions::new().create(true).write(true).open(helloworld_path).unwrap();
    helloworld_file.write(b"helloworld").unwrap();

    let helloworld_sha = cli::hash_object(Format::Blob, true, helloworld_path);
    let foobar_sha = cli::hash_object(Format::Blob, true, foobar_path);

    // let tree_content = format!("{}\n", helloworld_sha, foobar_sha);
    println!("{}", foobar_sha);
    println!("{}", helloworld_sha);
    let tree = ls_tree(path, &mut builder);
    insta::assert_snapshot!("hash_object directory tree", tree.unwrap());
    let mut map: IndexMap<String, String> = IndexMap::new();
    map.insert(helloworld_sha.clone(), cli::cat_file(Format::Blob, &helloworld_sha, Some(repo_path)));
    map.insert(foobar_sha.clone(), cli::cat_file(Format::Blob, &foobar_sha, Some(repo_path)));
    insta::assert_snapshot!("cat_file (object_read) unhashed objects", format_index_map(map));
     
    // TODO: create two trees representing two different commits
    // - 2 trees
    // - 2 commits
    // - Validate output of log command for commits 
    // - Validate output of ls tree command
    // - Create refs to each of these and validate whether util functions work
      
}

fn format_index_map(map: IndexMap<String, String>) -> String {
    map.into_iter().map(|(k, v)| k + ": " + &v).collect::<Vec<String>>().join("\n")
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

