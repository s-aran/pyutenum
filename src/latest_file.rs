use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::Args;

pub static LATEST_FILE_NAME: &str = ".latest_rspyast";

pub struct LatestFile {
    file_name: String,
    file_path: PathBuf,

    contents: Vec<String>,
}

impl LatestFile {
    pub fn new(file_name: String, file_path: PathBuf) -> Self {
        Self {
            file_name,
            file_path,

            contents: Vec::new(),
        }
    }

    pub fn new_from_args(args: &Args) -> Self {
        let src = if args.latest_file.is_some() {
            args.latest_file.as_ref().unwrap().to_string()
        } else {
            let p = std::env::temp_dir();
            p.join(LATEST_FILE_NAME).to_str().unwrap().to_string()
        };
        let latest_path = Path::new(&src);

        Self::new(
            latest_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap_or("")
                .to_string(),
            latest_path.to_path_buf(),
        )
    }

    pub fn get_file_name(&self) -> &String {
        &self.file_name
    }

    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn set_contents(&mut self, content: String) {
        self.contents = content.split('\n').map(|s| s.to_string()).collect();
    }

    pub fn get_contents(&self) -> &Vec<String> {
        &self.contents
    }

    pub fn exists(&self) -> bool {
        self.get_file_path().exists()
    }

    pub fn read(&self) -> Result<String, String> {
        if !self.exists() {
            return Err(format!(
                "File not found: {}",
                self.get_file_path().display()
            ));
        }

        let mut f = match File::open(self.get_file_path()) {
            Ok(f) => f,
            Err(e) => return Err(format!("Error opening file: {}", e)),
        };

        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(e) => return Err(format!("Error reading file: {}", e)),
        };

        Ok(contents)
    }

    pub fn write(&mut self, content: String) -> Result<(), String> {
        let mut f = if !self.exists() {
            match File::create(self.get_file_path()) {
                Ok(f) => f,
                Err(e) => return Err(format!("Error creating file: {}", e)),
            }
        } else {
            match File::open(self.get_file_path()) {
                Ok(f) => f,
                Err(e) => return Err(format!("Error opening file: {}", e)),
            }
        };

        match f.write_all(content.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error writing to file: {}", e)),
        }
    }

    pub fn remove(&self) -> Result<(), String> {
        match std::fs::remove_file(self.get_file_path()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error removing file: {}", e)),
        }
    }

    pub fn check(&self, tests: Vec<&String>) -> bool {
        for content in self.get_contents() {
            if !tests.contains(&content) {
                return false;
            }
        }

        true
    }
}
