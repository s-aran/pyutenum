use glob::{glob, GlobError};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
const IGNORE_DIR_NAMES: [&str; 1] = ["site-packages"];

fn glob_handler(p: Result<PathBuf, GlobError>) -> Option<PathBuf> {
    let current_path = match p {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    };

    if IGNORE_DIR_NAMES
        .iter()
        .any(|d| current_path.to_string_lossy().contains(d))
    {
        return None;
    }

    if current_path.is_file() {
        let meta = match current_path.metadata() {
            Ok(m) => m,
            Err(_) => return None,
        };
        if meta.len() <= 0 {
            return None;
        }
    }

    return Some(current_path);
}

pub fn glob_py(target_dir: impl Into<String>) -> HashSet<PathBuf> {
    let mut path_set = HashSet::<PathBuf>::new();

    let target_dir_str = target_dir.into();

    let glob_pattenrs = [
        format!("{}/**/test.py", &target_dir_str),
        format!("{}/**/tests.py", &target_dir_str),
        format!("{}/**/test_*.py", &target_dir_str),
    ];

    for ps in glob_pattenrs.iter() {
        let r = glob(ps.as_str()).expect("failed glob");
        for p in r {
            match glob_handler(p) {
                Some(current_path) => {
                    path_set.insert(current_path);
                }
                None => {}
            }
        }
    }

    path_set
}
