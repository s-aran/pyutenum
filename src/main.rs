use crate::enumerator::enumerate_tests;
use crate::parser::parse_file;
use clap::{arg, command, Parser};
use glob::glob_py;

mod models;
use std::{collections::HashMap, collections::HashSet, path::Path};

use models::Statements;

mod enumerator;
mod glob;
mod parser;
mod skip;

fn main() {
    let args = Args::parse();

    let target_dir = match args.files {
        Some(p) => p,
        None => ".".to_owned(),
    };

    let mut statements_map: HashMap<String, Statements> = HashMap::new();
    let mut enumerated_test = HashSet::<String>::new();

    let path_set = glob_py(target_dir);

    for current_path in path_set.iter() {
        let test_py_path = Path::new(&current_path);
        let parsed: Statements = match parse_file(&test_py_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        let py_path_str = test_py_path.to_string_lossy().to_string();
        if !statements_map.contains_key(&py_path_str) {
            if parsed.import_table.len() > 0 {
                statements_map.insert(py_path_str, parsed.clone());
            }
        }

        let tests = enumerate_tests(&parsed);
        enumerated_test.extend(tests);
    }

    let mut sorting = enumerated_test
        .iter()
        .map(|e| e.to_owned())
        .collect::<Vec<String>>();
    sorting.sort();

    let output = sorting
        .iter()
        .map(|p| p.to_owned())
        .collect::<Vec<String>>()
        .join("\n");

    println!("{}", output);
}

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(help = "FILE")]
    files: Option<String>,
}
