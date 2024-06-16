use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::Read,
    path::Path,
};

use rustpython_parser::{
    ast::{
        self, located::Stmt, Identifier, StmtClassDef, StmtFunctionDef, StmtImport, StmtImportFrom,
    },
    Parse,
};

fn main() {
    let test_py_base_path = Path::new("test_files");
    let test_py_path = test_py_base_path.join("test_simple.py");

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

    for e in result.iter() {
        println!("{:?}", e);
    }

    println!("");

    let rf = result.get(0).unwrap();
    // let import_stmt = rf.as_import_stmt().unwrap();
    // let name_first = import_stmt.names.get(0).unwrap();
    // println!("{} as {:?}", name_first.name, name_first.asname);

    let rl = result.get(result.len() - 1).unwrap();
    let class_deco = rl.as_class_def_stmt().unwrap();
    for d in class_deco.decorator_list.iter() {
        println!("{:?}", d);
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
            }
            ast::Stmt::FunctionDef(stmt) => {
                // test_methods.push(stmt);
                func_def_stmt(&mut states, &stmt);
            }
            _ => {}
        }

        println!("imports: {}", states.imports.len());
        for stmt in states.imports.iter() {
            println!("{:?}", stmt);
            for name in stmt.names.iter() {
                let asname = name.asname.as_ref();
                if asname.is_some() {
                    println!("* import ==> {:?}", asname.unwrap().as_str());
                }
            }
        }
    }

    println!("--------------------------------------------------------------------------------");

    let mods = module_runner(&states.imports, &states.import_from);
    println!("{:?}", mods);

    for c in states.classes.iter() {
        // println!("* {} in methods: {}", c.name, c.body.len());
        for b in c.body.iter() {
            if !b.is_function_def_stmt() {
                continue;
            }

            let func = b.as_function_def_stmt().unwrap();
            // println!("* {}", func.name);

            for deco in func.decorator_list.iter() {
                // println!("* {} in decorator list", deco);
            }
        }

        let mut current = String::new();
        let mut names = vec![];
        make_path_def_name(c, &mut current, &mut names);
        // println!("names: {:?}", names);
    }
}

struct Import {
    module: String,
    alias: Vec<String>,
    level: u8,
    exports: Vec<Box<Import>>,
}

impl Default for Import {
    fn default() -> Self {
        Self {
            module: "".to_owned(),
            alias: vec![],
            level: 0,
            exports: vec![],
        }
    }
}

impl From<Import> for StmtImport {
    fn from(stmt: StmtImport) -> Self {
        println!("{:?}", stmt);

        Import::new("hoge".to_owned())
    }
}

impl Import {
    pub fn new(module: String) -> Self {
        let mut result = Self::default();
        result.module = module;
        result
    }
}

// {module: {exported: (alias, ...), ...}, ...}
fn module_runner(
    imports: &Vec<StmtImport>,
    import_froms: &Vec<StmtImportFrom>,
) -> HashMap<String, Option<HashMap<String, HashSet<Option<String>>>>> {
    // original module name: [exported, ...]
    let mut result: HashMap<String, Option<HashMap<String, HashSet<Option<String>>>>> =
        HashMap::new();

    // import ...
    for stmt in imports.iter() {
        let module = stmt.names.get(0).unwrap();
        result.insert(module.name.to_string(), None);

        let aaa: Import = Import::from(stmt);
    }

    // from ... import ... (as ...)
    for stmt in import_froms.iter() {
        let module = stmt.module.as_ref().unwrap().to_string();
        let mut exports: HashMap<String, HashSet<Option<String>>> = HashMap::new();
        if result.contains_key(&module) {
            println!("contains: {}", module);

            // merge exports
            let tmp = match result.get(&module).unwrap() {
                Some(e) => e.to_owned(),
                None => HashMap::new(),
            };

            for (e, a) in tmp.iter() {
                println!("e: {:?}", e);
                println!("a: {:?}", a);
            }

            // let exports = result.get(&module).unwrap().to_owned();
            // tmp.extend(exports);
        }
        // tmp.extend(stmt.names.iter().map(|e| e.name.to_string()));
        result.insert(module, Some(exports));
    }

    result
}

#[derive(Debug, Default)]
struct States {
    imports: Vec<StmtImport>,
    import_from: Vec<StmtImportFrom>,
    classes: Vec<StmtClassDef>,
    methods: Vec<StmtFunctionDef>,
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
