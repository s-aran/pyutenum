use clap::{arg, command, Parser};

use std::{
    collections::HashMap,
    collections::HashSet,
    error::Error,
    ffi::OsStr,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::models::{Exports, Import, ImportMap, Level, Module, ModuleName, Statements};
use rustpython_parser::{
    ast::{
        self, Alias, Stmt, StmtClassDef, StmtFunctionDef, StmtImport, StmtImportFrom, StmtRaise,
    },
    Parse,
};

pub fn parse_file(test_py_path: &Path) -> Result<Statements, String> {
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

    let states = build_statements(&test_py_path, &result);

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
