use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use rustpython_parser::{
    ast::{self, Alias, StmtClassDef, StmtFunctionDef, StmtImport, StmtImportFrom, StmtRaise},
    Parse,
};

fn parse_file(test_py_path: &Path) {
    let mut test_file = match File::open(test_py_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let mut buf = String::new();
    match test_file.read_to_string(&mut buf) {
        Ok(s) => {
            println!("{} bytes read.", s);
        }
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    // println!("{}", buf);

    let test_py_base_path = test_py_path.parent().unwrap();
    let result = match ast::Suite::parse(
        buf.as_str(),
        test_py_base_path.file_name().unwrap().to_str().unwrap(),
    )
    .map_err(|e| e.to_string())
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
    };

    // for e in result.iter() {
    //     println!("{:?}", e);
    // }

    println!("");

    // let rf = result.get(0).unwrap();
    // let import_stmt = rf.as_import_stmt().unwrap();
    // let name_first = import_stmt.names.get(0).unwrap();
    // println!("{} as {:?}", name_first.name, name_first.asname);

    let rl = result.get(result.len() - 1).unwrap();
    if rl.is_class_def_stmt() {
        let class_deco = rl.as_class_def_stmt().unwrap();
        for d in class_deco.decorator_list.iter() {
            // println!("{:?}", d);
        }
    }

    let mut states = States::default();
    let tests: Vec<String> = vec![];

    for rs in result.iter() {
        match rs {
            ast::Stmt::Import(stmt) => {
                // imports.push(stmt);
                import_stmt(&mut states, stmt);
            }
            ast::Stmt::ImportFrom(stmt) => {
                import_from_stmt(&mut states, stmt);
            }
            ast::Stmt::ClassDef(stmt) => {
                // test_classes.push(stmt);
                class_def_stmt(&mut states, &stmt);
                // TODO: exploring to functions, recursive calling this function, too fast.
            }
            ast::Stmt::FunctionDef(stmt) => {
                // test_methods.push(stmt);
                func_def_stmt(&mut states, &stmt);
                for e in stmt.body.iter() {
                    match e {
                        ast::Stmt::Raise(s) => {
                            // TODO: REFACTORING
                            println!("{:?}", s);
                            if s.exc.is_none() {
                                continue;
                            }
                            let call = &s.exc.clone().unwrap();
                            if !call.is_call_expr() && call.as_call_expr().is_none() {
                                continue;
                            }
                            let call_expr = call.as_call_expr().unwrap();

                            let aa = call_expr.func.clone();
                            let ee = aa.as_attribute_expr().unwrap();
                            let vv = &ee.value;
                            let attr = &ee.attr;
                            println!("{:?}", attr);
                            println!("{:?}", vv);
                            raise_stmt(&mut states, &s);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        // println!("imports: {}", states.imports.len());
        // for stmt in states.imports.iter() {
        for stmt in states.raises.iter() {
            println!("stmt: {:?}", stmt.exc);
            if stmt.exc.is_none() {
                continue;
            }

            let call = &stmt.exc;
            // let func = &call.func;
            println!("{:?}", call);

            // for name in stmt.names.iter() {
            //     let asname = name.asname.as_ref();
            //     if asname.is_some() {
            //         println!("* import ==> {:?}", asname.unwrap().as_str());
            //     }
            // }
        }
    }

    println!("--------------------------------------------------------------------------------");

    let mods = module_runner(&states.imports, &states.import_from);
    println!("mods:");
    print_pretty(&mods);

    let leveled_modules: ImportMap = mods
        .clone()
        .into_iter()
        .filter(|(k, _)| k.2 && k.1 > 0)
        .collect();

    for (m, e) in leveled_modules.iter() {
        let (name, level, _) = m;

        let pb = make_path_from_module(name, level);
        let p = test_py_base_path.join(pb);
        println!("{:?}", p);
        if p.exists() {
            println!("{:?}", p);
            has_unittest_skip(&p);
        }
    }

    // TODO: load script level > 0

    for c in states.classes.iter() {
        // println!("* {} in methods: {}", c.name, c.body.len());
        for b in c.body.iter() {
            if !b.is_function_def_stmt() {
                continue;
            }

            let func = b.as_function_def_stmt().unwrap();
            // println!("* {}", func.name);

            for deco in func.decorator_list.iter() {
                println!("* {:?} in decorator list", deco);
            }
        }

        let mut current = String::new();
        let mut names = vec![];
        make_path_def_name(c, &mut current, &mut names);
        // println!("names: {:?}", names);
    }
}

fn main() {
    let test_py_base_path = Path::new("test_files");
    let test_py_path = test_py_base_path.join("test_simple.py");

    parse_file(&test_py_path);
}

#[derive(Debug)]
struct Name {
    name: String,
    alias: Option<String>,
}

impl Default for Name {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            alias: None,
        }
    }
}

impl From<&Alias> for Name {
    fn from(a: &Alias) -> Self {
        Self {
            name: a.name.to_string(),
            alias: match &a.asname {
                Some(n) => Some(n.to_string()),
                None => None,
            },
        }
    }
}

#[derive(Debug)]
struct Import {
    name: String,
    alias: Vec<Name>,
    level: u32,
    from_import: bool,
}

impl Default for Import {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            alias: vec![],
            level: 0,
            from_import: false,
        }
    }
}

impl From<&StmtImport> for Import {
    fn from(stmt: &StmtImport) -> Self {
        let mut result = Self::default();

        let first = stmt.names.get(0).unwrap();
        result.name = first.name.to_string();
        result.alias.push(first.into());

        println!("from<ImportStmt>: {:?}", stmt);

        result
    }
}

impl From<&StmtImportFrom> for Import {
    fn from(stmt: &StmtImportFrom) -> Self {
        let mut result = Self::default();
        // println!("!! md: {:?}", stmt.module);
        result.from_import = true;

        result.name = stmt.module.clone().unwrap().to_string();
        // println!(" ------ {:?}", stmt.level.unwrap());
        // println!(" ------ {:?}", stmt.level.unwrap().to_u32());
        result.level = match stmt.level {
            Some(v) => v.to_u32(),
            None => 0,
        };

        for name in stmt.names.iter() {
            // println!("!! nm: {:?}", name.name);
            // println!("!! as: {:?}", name.asname);
            result.alias.push(name.into());
        }

        println!("from<FromImportStmt>: {:?}", stmt);

        result
    }
}

impl Import {
    pub fn print_pretty(&self) {
        println!("name: {}", self.name);
        println!("level: {}", self.level);
        println!("from import: {}", self.from_import);
        for a in self.alias.iter() {
            println!("  alias: {}", a.name);
            if a.alias.is_some() {
                println!("    as: {}", a.alias.as_ref().unwrap());
            }
        }
    }
}

type ModuleName = String;
type AliasName = Option<String>;
type ExportName = String;
type Level = u32;
type Module = (ModuleName, Level, bool);
type Exports = HashMap<ExportName, AliasName>;
type ImportMap = HashMap<Module, Option<Exports>>;

// {module: {exported: (alias, ...), ...}, level}, ...
fn module_runner(imports: &Vec<StmtImport>, import_froms: &Vec<StmtImportFrom>) -> ImportMap {
    println!("--------------------------------------------------------------------------------");

    // original module name: [exported, ...]
    let mut result: ImportMap = HashMap::new();

    // import ...
    let mut imports2: Vec<Import> = imports.iter().map(|s| From::from(s)).collect();

    // from ... import ... (as ...)
    let import_froms2: Vec<Import> = import_froms.iter().map(|s| From::from(s)).collect();

    imports2.extend(import_froms2);

    for i in imports2.iter() {
        // println!("{:?}", i);
        i.print_pretty();

        let mut exports: Exports = HashMap::new();
        for a in i.alias.iter() {
            let nm = a.name.clone();
            let al = a.alias.clone();
            exports.insert(nm, al);
        }

        if !result.contains_key(&(i.name.clone(), i.level, i.from_import)) {
            let module: Module = (i.name.to_owned(), i.level, i.from_import);
            result.insert(module, Some(exports));
        } else {
            let q = result
                .get_mut(&(i.name.clone(), i.level, i.from_import))
                .unwrap()
                .as_mut()
                .unwrap();

            println!("q = {:?}", q);
            q.extend(exports);
        }
    }

    // println!("********************************************************************************");
    // print_pretty(&result);
    // println!("********************************************************************************");

    result
}

fn has_unittest_skip(path: &Path) -> bool {
    parse_file(&path);

    true
}

fn make_path_from_module(name: &ModuleName, level: &Level) -> PathBuf {
    let mut nodes = [".."].repeat(*level as usize - 1).to_vec();
    nodes.extend(name.split('.'));

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

#[derive(Debug, Default)]
struct States {
    imports: Vec<StmtImport>,
    import_from: Vec<StmtImportFrom>,
    classes: Vec<StmtClassDef>,
    methods: Vec<StmtFunctionDef>,
    raises: Vec<StmtRaise>,
}

fn import_stmt(states: &mut States, stmt: &StmtImport) {
    states.imports.push(stmt.clone());
}

fn import_from_stmt(states: &mut States, stmt: &StmtImportFrom) {
    states.import_from.push(stmt.clone());
}

fn class_def_stmt(states: &mut States, stmt: &StmtClassDef) {
    states.classes.push(stmt.clone());
}

fn func_def_stmt(states: &mut States, stmt: &StmtFunctionDef) {
    states.methods.push(stmt.clone());
}

fn raise_stmt(states: &mut States, stmt: &StmtRaise) {}

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
