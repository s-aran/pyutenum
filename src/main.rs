use clap::{arg, command, Parser};
use glob::glob;

mod models;
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsStr,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use models::{Exports, Import, ImportMap, Level, Module, ModuleName, Statements};
use rustpython_parser::{
    ast::{
        self, Alias, Stmt, StmtClassDef, StmtFunctionDef, StmtImport, StmtImportFrom, StmtRaise,
    },
    Parse,
};

static UNITTEST_TEST_CASE_CLASS: &str = "TestCase";
static UNITTEST_TEST_CASE_PREFIX: &str = "test_";

// with Django
static UNITTEST_DJANGO_TRANSACTION_TEST_CASE_CLASS: &str = "TransactionTestCase";
static UNITTEST_DJANGO_SIMPLE_TEST_CASE: &str = "SimpleTestCase";
static UNITTEST_DJANGO_LIVE_SERVER_TEST_CASE: &str = "LiveServerTestCase";

const IGNORE_DIR_NAMES: [&str; 1] = ["site-packages"];

fn parse_file(test_py_path: &Path) -> Result<Statements, String> {
    let mut test_file = match File::open(test_py_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            return Err(e.to_string());
        }
    };

    let mut buf = String::new();
    match test_file.read_to_string(&mut buf) {
        Ok(s) => {
            // println!("{} bytes read.", s);
            if s <= 0 {
                eprintln!("{:?} empty", test_file);
                return Ok(Statements::default());
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(e.to_string());
        }
    };

    let test_py_base_path = test_py_path.parent().unwrap();
    let file_name = match test_py_base_path.file_name() {
        Some(f) => f,
        None => {
            return Err(format!("cannot get file name from {:?}", test_py_base_path));
        }
    };
    let result = match ast::Suite::parse(buf.as_str(), file_name.to_str().unwrap())
        .map_err(|e| e.to_string())
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Err(e.to_string());
        }
    };

    // let rl = result.get(result.len() - 1).unwrap();
    // if rl.is_class_def_stmt() {
    //     let class_deco = rl.as_class_def_stmt().unwrap();
    //     for d in class_deco.decorator_list.iter() {
    //         // println!("{:?}", d);
    //     }
    // }

    let states = build_statements(&test_py_path, &result);

    // println!("--------------------------------------------------------------------------------");

    // println!("mods:");
    // print_pretty(&mods);

    // let mods = module_runner(&states.imports, &states.import_from);
    // let leveled_modules: ImportMap = mods
    //     .clone()
    //     .into_iter()
    //     .filter(|(k, _)| k.2 && k.1 > 0)
    //     .collect();

    // for (m, e) in leveled_modules.iter() {
    //     let (name, level, _) = m;

    //     let pb = make_path_from_module(name, level);
    //     let p = test_py_base_path.join(pb);
    //     // println!("{:?}", p);
    //     if p.exists() {
    //         // println!("{:?}", p);
    //         let statements = match parse_file(&p) {
    //             Ok(s) => s,
    //             Err(e) => {
    //                 eprintln!("{}", e);
    //                 continue;
    //             }
    //         };

    //         has_unittest_skip(&statements);
    //     }
    // }

    // // TODO: load script level > 0

    // for c in states.classes.iter() {
    //     // println!("* {} in methods: {}", c.name, c.body.len());
    //     for b in c.body.iter() {
    //         if !b.is_function_def_stmt() {
    //             continue;
    //         }

    //         let func = b.as_function_def_stmt().unwrap();
    //         // println!("* {}", func.name);

    //         for deco in func.decorator_list.iter() {
    //             // println!("* {:?} in decorator list", deco);
    //         }
    //     }

    //     let mut current = String::new();
    //     let mut names = vec![];
    //     make_path_def_name(c, &mut current, &mut names);
    //     // println!("names: {:?}", names);
    // }

    Ok(states)
}

fn main() {
    let args = Args::parse();

    let target_dir = match args.files {
        Some(p) => p,
        None => ".".to_owned(),
    };

    let mut statements_map: HashMap<String, Statements> = HashMap::new();

    let ps = format!("{}/**/{}*.py", target_dir, UNITTEST_TEST_CASE_PREFIX);
    let r = glob(ps.as_str()).expect("failed glob");
    for p in r {
        let current_path = match p {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        if IGNORE_DIR_NAMES
            .iter()
            .any(|d| current_path.to_string_lossy().contains(d))
        {
            continue;
        }

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

        // println!(
        //     "================================================================================"
        // );
        // for (k, v) in statements_map.iter() {
        //     println!("*** {}", k);
        //     // print_pretty(&v.import_table);
        // }
        // //  println!(
        // //      "================================================================================"
        // //  );

        let tests = enumerate_tests(&parsed);
        for t in tests.iter() {
            println!("{}", t);
        }
    }
}

// {module: {exported: (alias, ...), ...}, level}, ...
fn module_runner(imports: &Vec<StmtImport>, import_froms: &Vec<StmtImportFrom>) -> ImportMap {
    // println!("--------------------------------------------------------------------------------");

    // original module name: [exported, ...]
    let mut result: ImportMap = HashMap::new();

    // import ...
    let mut imports2: Vec<Import> = imports.iter().map(|s| From::from(s)).collect();

    // from ... import ... (as ...)
    let import_froms2: Vec<Import> = import_froms.iter().map(|s| From::from(s)).collect();

    imports2.extend(import_froms2);

    for i in imports2.iter() {
        // println!("{:?}", i);
        // i.print_pretty();

        let mut exports: Exports = HashMap::new();
        for a in i.alias.iter() {
            let nm = a.name.to_owned();
            let al = a.alias.to_owned();
            exports.insert(nm, al);
        }

        if !result.contains_key(&(i.name.to_owned(), i.level, i.from_import)) {
            let module: Module = (i.name.to_owned(), i.level, i.from_import);
            result.insert(module, Some(exports));
        } else {
            let q = result
                .get_mut(&(i.name.clone(), i.level, i.from_import))
                .unwrap()
                .as_mut()
                .unwrap();

            // println!("q = {:?}", q);
            q.extend(exports);
        }
    }

    // print_pretty(&result);

    result
}

static UNITTEST_MODULE: &str = "unittest";
static UNITTEST_SKIP_EXCEPTION: &str = "SkipTest";
static UNITTEST_SKIP_DECORATOR: &str = "skip";
static UNITTEST_SKIP_IF_DECORATOR: &str = "skipIf";
static UNITTEST_SKIP_UNLESS_DECORATOR: &str = "skipUnless";

fn has_unittest_skip(statements: &Statements) -> bool {
    // println!("has_unittest_skip ==>");
    let import = &statements.import_table;

    let mut import_unittest = false;
    let mut import_skip_test = false;
    let mut import_skip = false;
    let mut import_skip_if = false;
    let mut import_skip_unless = false;

    for (k, v) in import.iter() {
        let (m, l, f) = k;
        if &m.clone().unwrap_or("".to_string()) != &UNITTEST_MODULE.to_string() {
            continue;
        }

        import_unittest = true;
        // println!("import_unittest => true");

        //  if v.is_some() {
        //      let e = v.as_ref().unwrap();
        //      for (kk, vv) in e.iter() {
        //          println!("  exported: {:?}", kk);
        //          if kk == &UNITTEST_SKIP_EXCEPTION.to_string() {
        //              import_skip_test = true;
        //          }

        //          if vv.is_some() {
        //              println!("    as: {:?}", vv);
        //          }
        //      }
        //  }
    }

    true
}

fn enumerate_tests(statements: &Statements) -> Vec<String> {
    let mut result: Vec<String> = vec![];

    // this module
    {
        let splitted = statements.module.split('.').collect::<Vec<&str>>();
        if splitted.len() > 0 {
            for (i, _) in splitted.iter().enumerate() {
                let s = splitted[..i].to_vec().join(".");
                if result.contains(&s) {
                    continue;
                }
                result.push(s);
            }
        }
        result.push(make_test_name(statements, None, None));

        //     // let mut nodes = [".."].repeat(*level as usize - 1).to_vec();
        //     // match name {
        //     //     Some(n) => nodes.extend(n.split('.').clone()),
        //     //     None => {}
        //     // };

        //     // let path_str = nodes.join("/") + ".py";
        //     // PathBuf::from_str(path_str.as_str()).unwrap()
    }

    // root function
    for f in statements.methods.iter() {
        if f.name.starts_with(UNITTEST_TEST_CASE_PREFIX) {
            result.push(make_test_name(&statements, None, Some(&f)));
        }
    }

    // class methods
    for c in statements.classes.iter() {
        // this class
        result.push(make_test_name(statements, Some(&c), None));

        for f in c.body.iter() {
            if !f.is_function_def_stmt() {
                continue;
            }

            let func = f.as_function_def_stmt().unwrap();
            if func.name.starts_with(UNITTEST_TEST_CASE_PREFIX) {
                result.push(make_test_name(&statements, Some(&c), Some(&func)));
            }
        }
    }

    result
}

fn make_test_name(
    statements: &Statements,
    class: Option<&StmtClassDef>,
    func: Option<&StmtFunctionDef>,
) -> String {
    let class_name = match class {
        Some(c) => format!(".{}", c.name.to_string()),
        None => "".to_string(),
    };

    let func_name = match func {
        Some(f) => format!(".{}", f.name.to_string()),
        None => "".to_string(),
    };

    format!("{}{}{}", statements.module, class_name, func_name,)
}

fn make_path_from_module(name: &ModuleName, level: &Level) -> PathBuf {
    let mut nodes = [".."].repeat(*level as usize - 1).to_vec();
    match name {
        Some(n) => nodes.extend(n.split('.').clone()),
        None => {}
    };

    let path_str = nodes.join("/") + ".py";
    PathBuf::from_str(path_str.as_str()).unwrap()
}

fn print_pretty(import: &ImportMap) {
    for (k, v) in import.iter() {
        println!("module: {:?}", k);
        if v.is_some() {
            let e = v.as_ref().unwrap();
            for (kk, vv) in e.iter() {
                println!("  exported: {:?}", kk);
                if vv.is_some() {
                    println!("    as: {:?}", vv);
                }
            }
        }
    }
}

// (module, level

fn import_stmt(states: &mut Statements, stmt: &StmtImport) {
    states.imports.push(stmt.clone());
}

fn import_from_stmt(states: &mut Statements, stmt: &StmtImportFrom) {
    states.import_from.push(stmt.clone());
}

fn class_def_stmt(states: &mut Statements, stmt: &StmtClassDef) {
    states.classes.push(stmt.clone());
}

fn func_def_stmt(states: &mut Statements, stmt: &StmtFunctionDef) {
    states.methods.push(stmt.clone());
}

fn raise_stmt(states: &mut Statements, stmt: &StmtRaise) {
    states.raises.push(stmt.clone());
}

fn make_path_def_name(stmt: &StmtClassDef, current: &mut String, names: &mut Vec<String>) {
    let my_name = stmt.name.to_string();
    let c = current.clone();
    current.clear();
    current.push_str(&if c.len() > 0 {
        format!("{}.{}", c, &my_name)
    } else {
        my_name
    });

    for body_stmt in stmt.body.iter() {
        if body_stmt.is_class_def_stmt() {
            let class_stmt = body_stmt.as_class_def_stmt().unwrap();
            let tmp = current.clone();
            make_path_def_name(class_stmt, current, names);
            current.clear();
            current.push_str(&tmp);
            continue;
        }

        if body_stmt.is_function_def_stmt() {
            let func_stmt = body_stmt.as_function_def_stmt().unwrap();
            let my_name = func_stmt.name.to_string();

            let tmp = format!("{}.{}", current, my_name);
            names.push(tmp);
            continue;
        }
    }
}

fn _internal_build_statements(rs: &Stmt, statements: &mut Statements) {
    match rs {
        ast::Stmt::Import(stmt) => {
            import_stmt(statements, stmt);
        }
        ast::Stmt::ImportFrom(stmt) => {
            import_from_stmt(statements, stmt);
        }
        ast::Stmt::ClassDef(stmt) => {
            class_def_stmt(statements, &stmt);
            // for c in stmt.body.iter() {
            //     _internal_build_statements(c, statements);
            // }
        }
        ast::Stmt::FunctionDef(stmt) => {
            func_def_stmt(statements, &stmt);
            // for f in stmt.body.iter() {
            //     _internal_build_statements(f, statements);
            // }
        }
        // ast::Stmt::Raise(s) => {
        //     // println!(
        //     //     "--------------------------------------------------------------------------------"
        //     // );
        //     // println!("{:?}", s);
        //     if s.exc.is_none() {
        //         return;
        //     }
        //     let call = &s.exc.clone().unwrap();
        //     // println!("call: {:?}", call);
        //     if !call.is_call_expr() && call.as_call_expr().is_none() {
        //         return;
        //     }
        //     let call_expr = call.as_call_expr().unwrap();
        //     // println!("call_expr: {:?}", call_expr);

        //     let func = call_expr.func.clone();
        //     // println!("func: {:?}", func);

        //     if func.is_attribute_expr() {
        //         // module.name()
        //         let module = func.as_attribute_expr().unwrap();
        //         // println!("attribute: {:?}", module);
        //         let value = &module.value;

        //         let attr = &module.attr;
        //         // println!("attr: {:?}", attr);

        //         let name = &value.as_name_expr().unwrap().id;
        //         // println!("name: {:?}", name);
        //     } else {
        //         // name()
        //         let value = func.as_name_expr().unwrap();
        //         let name = &value.id;
        //         // println!("name: {:?}", name);
        //     }

        //     raise_stmt(statements, &s);
        // }
        _ => {}
    }
}

fn build_statements(module_path: &Path, root: &Vec<Stmt>) -> Statements {
    let mut states = Statements::default();

    let p = module_path.iter().map(|p| p).collect::<Vec<&OsStr>>();
    states.module = p
        .iter()
        .enumerate()
        .map(|(i, e)| {
            if i == p.len() - 1 {
                // remove extension
                Path::new(e)
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            } else {
                e.to_string_lossy().into_owned()
            }
        })
        .collect::<Vec<String>>()
        .join(".");

    for rs in root.iter() {
        _internal_build_statements(rs, &mut states);

        // println!("imports: {}", states.imports.len());
        // for stmt in states.imports.iter() {
        // for stmt in states.raises.iter() {
        //     // println!("stmt: {:?}", stmt.exc);
        //     if stmt.exc.is_none() {
        //         continue;
        //     }

        //     let call = &stmt.exc;
        //     // let func = &call.func;
        //     // println!("{:?}", call);

        //     // for name in stmt.names.iter() {
        //     //     let asname = name.asname.as_ref();
        //     //     if asname.is_some() {
        //     //         println!("* import ==> {:?}", asname.unwrap().as_str());
        //     //     }
        //     // }
        // }
    }

    // let mods = module_runner(&states.imports, &states.import_from);
    // states.import_table = mods;

    states
}

#[derive(Clone, Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(help = "FILE")]
    files: Option<String>,
}
