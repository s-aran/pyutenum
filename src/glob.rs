use glob::{glob, GlobError};
use rayon::prelude::*;
use std::{collections::HashSet, path::PathBuf};
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
    let target_dir_str = target_dir.into();

    let glob_pattenrs = [
        format!("{}/**/test.py", &target_dir_str),
        format!("{}/**/tests.py", &target_dir_str),
        format!("{}/**/test_*.py", &target_dir_str),
    ];

    let path_vec = glob_pattenrs
        .iter()
        .map(|ps| {
            let r = glob(ps.as_str()).expect("failed glob");
            r.map(|p| glob_handler(p))
                .filter(|p| p.is_some())
                .map(|p| p.unwrap())
                .collect()
        })
        .collect::<Vec<Vec<PathBuf>>>();

    let mut path_set = HashSet::<PathBuf>::new();

    for v in path_vec.iter() {
        for p in v.iter() {
            path_set.insert(p.clone());
        }
    }

    path_set
}
