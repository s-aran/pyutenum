use crate::enumerator::enumerate_tests;
use crate::parser::parse_file;
use clap::{arg, command, Parser};
use glob::glob_py;

mod models;
use std::{
    collections::{HashMap, HashSet},
    io::{stdout, BufWriter, Write},
    path::Path,
};

use rayon::prelude::*;

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

    let mut sorting = enumerated_test.iter().collect::<Vec<&String>>();
    sorting.sort();

    let output = sorting
        .iter()
        .map(|p| p.as_bytes())
        .chain(Some("".as_bytes()).into_iter())
        .collect::<Vec<&[u8]>>()
        .join("\n".as_bytes());

    let stdout = stdout();
    let mut out = BufWriter::new(stdout.lock());

    for b in output.iter() {
        let _ = out.write_all(&[*b]);
    }

    let _ = out.flush();
}

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(help = "DIR")]
    files: Option<String>,
}
