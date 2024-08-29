use std::collections::HashMap;

use crate::models::{Exports, Import, ImportMap, Level, Module, ModuleName, Statements};
use rustpython_parser::ast::{StmtClassDef, StmtFunctionDef, StmtImport, StmtImportFrom};

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

fn has_unittest_skip_for_func(func: &StmtFunctionDef) -> bool {
    for deco in func.decorator_list.iter() {
        let is_attr = deco.is_attribute_expr();
        let is_name = deco.is_name_expr();

        if !is_attr && !is_name {
            // println!("continue!! {:?}", deco);
            continue;
        }

        // println!("{:?}", &deco);

        if is_attr {
            let ae = &deco.as_attribute_expr().unwrap();
            let v = &ae.value;
            let a = &ae.attr;
            if v.is_name_expr() {
                let nm = &v.as_name_expr();
                let i = &nm.unwrap().id;
                let id_str = &i.to_string();
                println!("vi: {:?}", id_str);
            }
            let attr_str = &a.to_string();
            println!("a: {:?}", attr_str);

            continue;
        }

        if is_name {
            let nm = &deco.as_name_expr().unwrap();
            let i = &nm.id;
            let id_str = &i.to_string();
            println!("i: {:?}", id_str);
        }
    }

    true
}

fn has_unittest_skip_for_class(class: &StmtClassDef) -> bool {
    for deco in class.decorator_list.iter() {
        let d = deco;
        println!("{:?}", d);
        println!("* {:?} in decorator list", deco);
    }

    true
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

pub fn print_pretty(import: &ImportMap) {
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
