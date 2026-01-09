use core::str;
use std::{

    fs::{self, DirEntry, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}
};
use crate::error::{
    Error,
    Result
};
use regex::Regex;
use flate2::{
    write::ZlibEncoder,
    read::ZlibDecoder,
    Compression
};
use indexmap::IndexMap;
use crate::object::Object;
use crate::cli::Format;
use configparser::ini::Ini;

#[derive(Debug)]
pub struct Repository {
    worktree: PathBuf,
    pub minit_dir: PathBuf,
    conf: Ini,
}

impl Repository {
    pub fn create(path: &Path) -> Result<Self> {
        let mut repo = Repository::new(path, true)?;

        if repo.worktree.exists() {
            if !repo.worktree.is_dir() {
                return Err(Error::RepositoryConfigurationIssue);
            }
            if repo.minit_dir.exists() && repo.minit_dir.read_dir().is_ok() {
                return Err(Error::RepositoryConfigurationIssue);
            }
        } else {
            fs::create_dir_all(&repo.worktree)?;
        }

        repo.repo_dir(vec!["branches"], true)?;
        repo.repo_dir(vec!["objects"], true)?;
        repo.repo_dir(vec!["refs", "tags"], true)?;
        repo.repo_dir(vec!["refs", "heads"], true)?;
        
        let mut file_path = repo.repo_file(vec!["description"], false)?.unwrap();
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&file_path)?
            .write("Unnamed repository; edit this file 'description' to name the repository.\n".as_bytes())?;

        file_path = repo.repo_file(vec!["HEAD"], false)?.unwrap();
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&file_path)?
            .write("ref: refs/heads/master\n".as_bytes())?;

        file_path = repo.repo_file(vec!["config"], false)?.unwrap();
        repo.init_config(&file_path)?;

        Ok(repo)
    }

    fn init_config(&mut self, path: &Path) -> Result<()> {
        self.conf.set("core", "repositoryformatversion", Some(String::from("0")));
        self.conf.set("core", "filemode", Some(String::from("false")));
        self.conf.set("core", "bare", Some(String::from("false")));
        self.conf.write(path)?;
        Ok(())
    }

    /// 
    pub fn new(path: &Path, force: bool) -> Result<Self> {
        let worktree = path.to_path_buf();
        let minit_dir = worktree.join(".minit");
        if !(force || minit_dir.is_dir()) {
            return Err(Error::InvalidFilePath(minit_dir));
        }
        let mut repo = Repository {
            worktree, minit_dir, conf: Ini::new()
        };
        let config_path = repo.repo_file(vec!["config"], false)?.unwrap();
        if config_path.exists() {
            repo.conf.load(&config_path).unwrap();
        } else {
            if !force { panic!("Configuration file missing"); }
        }
               
        if !force {
            _ = repo.conf
                .get("core", "repositoryformatversion")
                .ok_or(Error::ConfigKeyDoesntExist(String::from("core")))
                .map(|version|
                    if version != "0" {
                        Err(Error::UnsupportedRepositoryVersion)
                    } else {
                        Ok(())
                    }
                )?;
        }
        Ok(repo)
    }

    ///
    pub fn repo_path(&self, paths: Vec<&str>) -> PathBuf {
        assert!(paths.len() > 0);
        let mut builder_string = self.minit_dir.clone();
        paths.iter().for_each(|&p| builder_string = builder_string.join(p));
        builder_string
    }

    /// ONLY CREATES THE DIRECTORIES
    pub fn repo_dir(&self, paths: Vec<&str>, mkdir: bool) -> Result<Option<PathBuf>> {
        assert!(paths.len() > 0);
        let path = self.repo_path(paths);
        // NOTE: is there a way to tidy this up?
        if path.exists() {
            if path.is_dir() {
                return Ok(Some(path));
            } else {
                return Err(Error::InvalidFilePath(path));
            }
        }

        if mkdir {
            fs::create_dir_all(&path)?;
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }

    ///
    pub fn repo_file(&self, paths: Vec<&str>, mkdir: bool) -> Result<Option<PathBuf>> {
        assert!(paths.len() > 0);
        if paths.len() == 1 {
            return Ok(Some(self.minit_dir.join(paths[0])));
        }
        
        self.repo_dir(paths[0..paths.len()-1].to_vec(), mkdir)
            .map(|op| op.map(|_| self.repo_path(paths)))
    }

    pub fn find(path: &Path, required: bool) -> Result<Option<Repository>> {
        let abs_path = path.canonicalize()?;
        let minit_path = abs_path.join(".minit");
        if minit_path.is_dir() {
            Some(Repository::new(&abs_path, false)).transpose()
        } else {
            let parent = abs_path
                .join("..")
                .canonicalize()?;
            if parent == abs_path {
                if required {
                    Err(Error::NoMinitRepository)
                } else {
                    Ok(None)
                }
            } else {
                Repository::find(&parent, required)
            }
        }
    }

    pub fn read_object(&self, sha: &str) -> Result<Object> {
        let path = self.repo_file(vec!["objects", &sha[0..2], &sha[2..]], false)?
            .ok_or(Error::ObjectNotDefined(sha.to_string()))?;

        if !path.is_file() {
            return Err(Error::ObjectNotDefined(sha.to_string()));
        }

        let mut file = OpenOptions::new().read(true).open(&path)?;
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf)?;

        let mut decoder = ZlibDecoder::new(&buf[..]);
        let mut data: Vec<u8> = Vec::new();
        decoder.read_to_end(&mut data)?;

        let mut it = data.iter();
        let obj_idx = it.position(|&b| b == b' ').unwrap();
        let size_idx = obj_idx + 1 + it.position(|&b| b == b'\x00').unwrap();
        let fmt = String::from_utf8(data[0..obj_idx].to_vec())?;
        let size = String::from_utf8(data[obj_idx+1..size_idx].to_vec())?.parse::<usize>()?;
        if size != data.len() - size_idx - 1 {
            panic!("Malformed object {}: bad length", sha);
        }

        let data = data[size_idx+1..].to_vec();
        return match fmt.as_str() {
            "commit" => Ok(Object::new(Format::Commit, data)?),
            "blob" => Ok(Object::new(Format::Blob, data)?),
            "tree" => Ok(Object::new(Format::Tree, data)?),
            "tag" => Ok(Object::new(Format::Tag, data)?),
            _ => panic!("Unknown object type {} for object {}", fmt, sha),
        }
    }

    /// Return the hash
    pub fn write_object(&self, obj: Object) -> Result<String>{
        let (sha, result) = obj.write()?;
        let path = self.repo_file(vec!["objects", &sha[0..2], &sha[2..]], true)?.unwrap();
        if !path.exists() {
            let mut file = OpenOptions::new().create(true).write(true).open(&path)?;
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());

            encoder.write_all(result.as_bytes())?;
            let compressed = encoder.finish()?; 
            file.write_all(&compressed[..])?;
        }
        return Ok(sha);
    }

    /// Return the hash
    pub fn find_object(&self, name: &str, format: Option<Format>, follow: bool) -> Result<String> {
        // NOTE: throw error when the found object is not the appropriate format
        let sha = self.resolve_object(name)?;
        if sha.len() != 1 {
            return Err(Error::AmbiguousReference(sha))
        }
        let mut sha: String = sha.get(0).unwrap().clone();

        if format.is_none() {
            return Ok(sha.clone());
        } 
        let format = format.unwrap();

        while true {
            let obj = self.read_object(&sha)?;
            if obj.format() == format {
                return Ok(sha);
            }
            if !follow {
                return Err(Error::ObjectNotFound);
            }
            match obj {
                Object::Tag(map) => {
                    sha = map.get("object").unwrap().get(0).unwrap().clone();
                },
                Object::Commit(map) => {
                    sha = map.get("tree").unwrap().get(0).unwrap().clone();
                },
                _ => return Err(Error::ObjectNotFound)
            }
            break;
        }
        Ok(name.to_string())
    }

    pub fn resolve_ref(&self, reference: &str) -> Result<String> {
        let path = self.repo_file(vec![reference], false)?.unwrap();

        if !path.is_file() {
            return Err(Error::InvalidFilePath(path));
        }

        let mut file = OpenOptions::new().read(true).open(&path)?;
        let mut buf = Vec::new();
        file.read(&mut buf)?;
        let data= str::from_utf8(&buf)?;
        if data.starts_with("ref: ") {
            self.resolve_ref(&data[5..]) 
        } else {
            Ok(data.to_string())
        }
    }

    pub fn ls_ref(&self, path: Option<&Path>) -> Result<IndexMap<String, String>> {
        let mut map: IndexMap<String, String> = IndexMap::new();
        self.populate_ref_map(&mut map, path)?;
        Ok(map)
    }

    fn populate_ref_map(&self, map: &mut IndexMap<String, String>, path: Option<&Path>) -> Result<()> {
        let ref_path = path
            .map(|p| p.to_path_buf())
            .unwrap_or(self.repo_dir(vec!["refs"], false)?.expect("refs directory must be defined"));
        let mut v = ref_path
            .read_dir()?
            .flatten()
            .collect::<Vec<DirEntry>>();
        v.sort_by_key(|d| d.path());
        let _ = v.iter().map(|p| -> Result<()> {
            let dir_entry = p.path();
            if dir_entry.is_dir() {
                let dir_path = ref_path.join(dir_entry);
                _ = self.populate_ref_map(map, Some(&dir_path));
            } else {
                let reference = ref_path.to_str().unwrap();
                let sha = self.resolve_ref(reference)?;
                map.insert(ref_path.to_str().unwrap().to_string(), sha);
            }
            Ok(())
        });
        Ok(())
    }

    pub fn create_tag(&self, name: &str, reference: &str, add: bool) -> Result<()> {
        let sha = self.find_object(reference, None, false)?;
        let sha = if add {
            let mut map: IndexMap<String, Vec<String>> = IndexMap::new();
            map.insert(String::from("object"), vec![sha]);
            map.insert(String::from("type"), vec![String::from("commit")]);
            map.insert(String::from("tag"), vec![String::from(name)]);
            map.insert(String::from("tagger"), vec![String::from("Wyag <wyag@example.com>")]);
            map.insert(String::from("message"), vec![String::from("A tag generated by wyag, which won't let you customize the message!\n")]);
            let object = Object::Tag(map);
            self.write_object(object)?
        } else {
            sha
        };
        let ref_name = String::from("tags/") + name;
        self.create_ref(&ref_name, &sha)?;
        Ok(())
    }

    fn create_ref(&self, ref_name: &str, sha: &str) -> Result<()> {
        let path = self.repo_file(vec!["refs/", ref_name], false)?.unwrap();
        let mut file = OpenOptions::new().write(true).open(&path)?;
        file.write(sha.as_bytes())?;
        file.write("\n".as_bytes())?;
        Ok(())
    }

    pub fn resolve_object(&self, name: &str) -> Result<Vec<String>> {
        let mut candidates: Vec<String> = Vec::new();
        let re = Regex::new(r"^([a-fA-F0-9]{4,64})$").unwrap();
        if name.len() == 0 {
            return Err(Error::NameNotDefined);
        }

        if name == "HEAD" {
            return Ok(vec![self.resolve_ref("HEAD")?])
        }
        
        if re.is_match(&name) {
            let name = name.to_lowercase();
            let prefix = &name[0..2];
            match self.repo_dir(vec!["objects", prefix], false)? {
                Some(path) => {
                    let rem = &name[2..];
                    path.read_dir()?
                        .map(|p| p.map(|entry| entry.file_name()))
                        .flatten()
                        .filter(|p| p.to_str().unwrap().starts_with(rem))
                        .for_each(|p| {
                            candidates.push(prefix.to_string() + p.to_str().unwrap());
                        });
                },
                _ => {}
            }
        }

        let reference_path = "refs/tags/".to_string() + name;
        _ = self.resolve_ref(&reference_path)
            .inspect(|tag| candidates.push(tag.clone()));

        let branch_path = "refs/heads/".to_string() + name;
        _ = self.resolve_ref(&branch_path)
            .inspect(|tag| candidates.push(tag.clone()));

        let remote_branch_path = "refs/remotes/".to_string() + name;
        _ = self.resolve_ref(&remote_branch_path)
            .inspect(|tag| candidates.push(tag.clone()));

        Ok(candidates) 
    }
}

