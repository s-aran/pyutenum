use crate::ast::Alias;
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
        // println!("{:?}", e);
    }

    println!("");

    let rf = result.get(0).unwrap();
    // let import_stmt = rf.as_import_stmt().unwrap();
    // let name_first = import_stmt.names.get(0).unwrap();
    // println!("{} as {:?}", name_first.name, name_first.asname);

    let rl = result.get(result.len() - 1).unwrap();
    let class_deco = rl.as_class_def_stmt().unwrap();
    for d in class_deco.decorator_list.iter() {
        // println!("{:?}", d);
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
            println!("stmt: {:?}", stmt);
            for name in stmt.names.iter() {
                let asname = name.asname.as_ref();
                if asname.is_some() {
                    println!("* import ==> {:?}", asname.unwrap().as_str());
                }
            }
        }
    }

    println!("--------------------------------------------------------------------------------");

    let mods = module_runner2(&states.imports, &states.import_from);
    println!("mods:");
    print_pretty(&mods);

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



#[derive(Debug)]
struct Import {
    module: String,
    aliases: Vec<String>,
    level: u8,
    exports: Vec<Box<Import>>,
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
struct Import2 {
    name: String,
    alias: Vec<Name>,
    level: u32,
    from_import: bool,
}

impl Default for Import2 {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            alias: vec![],
            level: 0,
            from_import: false,
        }
    }
}

impl From<&StmtImport> for Import2 {
    fn from(stmt: &StmtImport) -> Self {
        let mut result = Self::default();

        let first = stmt.names.get(0).unwrap();
        result.name = first.name.to_string();
        result.alias.push(first.into());

        println!("from<ImportStmt>: {:?}", stmt);

        result
    }
}

impl From<&StmtImportFrom> for Import2 {
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

        // let first = stmt.names.get(0).unwrap();
        // result.name = first.name.to_string();

        // if first.asname.is_some() {
        //     // result.alias = Some(first.asname.clone().unwrap().to_string());
        // }

        // let hoge = &stmt.names[1..];
        // for e in hoge.iter() {
        //     let mut qqq = Import2::default();
        //     qqq.name = e.name.to_string();
        //     if e.asname.is_some() {
        //         qqq.alias = Some(e.asname.clone().unwrap().to_string());
        //     }
        //     result.exports.push(Box::new(qqq));
        // }

        println!("from<FromImportStmt>: {:?}", stmt);

        result
    }
}

impl Import2 {
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

impl Default for Import {
    fn default() -> Self {
        Self {
            module: "".to_owned(),
            aliases: vec![],
            level: 0,
            exports: vec![],
        }
    }
}

impl From<&StmtImport> for Import {
    fn from(stmt: &StmtImport) -> Self {
        let mut result = Self::default();
        result.module = stmt.names.get(0).unwrap().name.to_string();

        println!("{:?}", stmt);

        // for e in stmt.names.iter() {
        //     println!("from<StmtImport>: {:?}", e);

        //     // export
        //     result.
        // }

        result
    }
}

impl From<&StmtImportFrom> for Import {
    fn from(stmt: &StmtImportFrom) -> Self {
        let mut result = Self::default();

        let first = &stmt.names.get(0).unwrap();
        println!("from<StmtImportFrom> first: {:?}", first);
        result.module = first.name.to_string();

        if first.asname.is_some() {
            let asname = first.asname.as_ref().unwrap();
            result.aliases.push(asname.to_string());
        }

        let hoge = &stmt.names[1..];
        for e in hoge.iter() {
            // println!("from<StmtImportFrom>: {:?}", e);

            // println!("nm: {:?}", e.name.to_string());
            // println!("as: {:?}", e.asname);

            let mut qqq = Import::default();
            qqq.module = e.name.to_string();
            if e.asname.is_some() {
                qqq.aliases.push(e.asname.clone().unwrap().to_string());
            }
            result.exports.push(Box::new(qqq));

            // let export = &e.to_string();
            // println!("ex: {}", export);

            // let name = &e.to_string();
            // println!("nm: {}", name);
            // result.aliases.append(name);
        }

        println!("result: {:?}", result);

        result
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
    println!("--------------------------------------------------------------------------------");
    //
    // original module name: [exported, ...]
    let mut result: HashMap<String, Option<HashMap<String, HashSet<Option<String>>>>> =
        HashMap::new();

    // import ...
    for stmt in imports.iter() {
        let module = stmt.names.get(0).unwrap();
        result.insert(module.name.to_string(), None);

        let aaa: Import = stmt.into();
    }

    // from ... import ... (as ...)
    for stmt in import_froms.iter() {
        let module = stmt.module.as_ref().unwrap().to_string();
        let mut exports: HashMap<String, HashSet<Option<String>>> = HashMap::new();
        let aaa: Import = stmt.into();

        if result.contains_key(&module) {
            // println!("contains: {}", module);

            // merge exports
            let tmp = match result.get(&module).unwrap() {
                Some(e) => e.to_owned(),
                None => HashMap::new(),
            };

            for (e, a) in tmp.iter() {
                // println!("e: {:?}", e);
                // println!("a: {:?}", a);
            }

            // let exports = result.get(&module).unwrap().to_owned();
            // tmp.extend(exports);
        }
        // tmp.extend(stmt.names.iter().map(|e| e.name.to_string()));
        result.insert(module, Some(exports));
    }

    result
}

type ModuleName = String;
type AliasName = Option<String>;
type ExportName = String;
type Level = u32;
type Module = (ModuleName, Level, bool);
type Exports = HashMap<ExportName, AliasName>;
type ImportMap = HashMap<Module, Option<Exports>>;

// {module: {exported: (alias, ...), ...}, level}, ...
fn module_runner2(imports: &Vec<StmtImport>, import_froms: &Vec<StmtImportFrom>) -> ImportMap {
    println!("--------------------------------------------------------------------------------");

    // original module name: [exported, ...]
    let mut result: ImportMap = HashMap::new();

    // import ...
    let mut imports2: Vec<Import2> = imports.iter().map(|s| From::from(s)).collect();

    // from ... import ... (as ...)
    let import_froms2: Vec<Import2> = import_froms.iter().map(|s| From::from(s)).collect();

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

    println!("********************************************************************************");
    print_pretty(&result);
    println!("********************************************************************************");

    result
}

fn is_valid_exported(import: &ImportMap, exported: &String) {
    // split module*.class?.func

    let splitted = exported.split(".");
    let top_module = splitted.get(0).unwrap();
    
    let leveled_modules = import.filter(|e| {
        e.1 > 0 && e.2
    });

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
