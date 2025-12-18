use std::{

    fs::{self, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}
};
use crate::error::{
    Error,
    Result
};
use flate2::{
    write::ZlibEncoder,
    read::ZlibDecoder,
    Compression
};
use crate::object::{Object, Format};
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

    pub fn read_object(&self, sha: String) -> Result<Object> {
        let path = self.repo_file(vec!["objects", &sha[0..2], &sha[2..]], false)?
            .ok_or(Error::ObjectNotDefined(sha.clone()))?;

        if !path.is_file() {
            return Err(Error::ObjectNotDefined(sha));
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

        return match fmt.as_str() {
            "blob" => Ok(Object::new(Format::Blob, Some(data[size_idx+1..].to_vec()))),
            "commit" | "tag" | "tree" => unimplemented!(),
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
    pub fn find_object(&self, name: String, format: Format, follow: bool) -> Result<String> {
        Ok(name)
    }
}

