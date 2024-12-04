use crate::enumerator::enumerate_tests;
use crate::parser::parse_file;
use clap::{arg, command, Parser};
use glob::glob_py;
use latest_file::{LatestFile, LATEST_FILE_NAME};

mod models;
use std::{
    collections::{HashMap, HashSet},
    io::{stdout, BufWriter, Write},
    path::{Path, PathBuf},
};

use rayon::prelude::*;

use models::Statements;

mod enumerator;
mod glob;
mod latest_file;
mod parser;

fn main() {
    let args = Args::parse();

    let latest_file = LatestFile::new_from_args(&args);
    if args.clear_latest {
        match latest_file.remove() {
            Ok(_) => return,
            Err(e) => {
                eprintln!("failed to clear latest file: {}", e);
                return;
            }
        }
    }

    if args.show_latest_filepath {
        println!("{}", latest_file.get_file_path().to_str().unwrap());
        return;
    }

    if latest_file.exists() && !args.force {
        let contents = match latest_file.read() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        // NOTE: avoid readable removed file
        if contents.len() > 0 {
            let split = contents
                .split("\n")
                .map(|l| l.to_owned())
                .collect::<HashSet<String>>();
            let mut sorting = split.iter().map(|s| s.to_owned()).collect::<Vec<String>>();
            sorting.sort();

            for l in sorting.iter() {
                println!("{}", l);
            }

            return;
        }
    }

    let target_dir = match args.dir {
        Some(p) => p,
        None => ".".to_owned(),
    };

    let mut statements_map: HashMap<String, Statements> = HashMap::new();
    let mut enumerated_test = HashSet::<String>::new();

    let path_set = glob_py(target_dir);

    let parsed_vec = path_set
        .par_iter()
        .map(|current_path| {
            let test_py_path = Path::new(&current_path);
            let parsed = parse_file(&test_py_path);
            (current_path, parsed)
        })
        .filter(|e| e.1.is_ok())
        .map(|e| (e.0, e.1.unwrap()))
        .collect::<HashMap<&PathBuf, Statements>>();

    for (test_py_path, parsed) in parsed_vec.iter() {
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
    dir: Option<String>,

    #[arg(short, long, help = format!("latest file, default=<temp dir>/{}", LATEST_FILE_NAME))]
    pub latest_file: Option<String>,

    #[arg(short, long, help = "clear latest file")]
    pub clear_latest: bool,

    #[arg(short, long, help = "show latest file path")]
    pub show_latest_filepath: bool,

    #[arg(short, long, help = "force enumerate")]
    pub force: bool,
}
