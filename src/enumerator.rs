use std::collections::HashSet;

use crate::models::Statements;
use rustpython_parser::ast::{StmtClassDef, StmtFunctionDef};

static UNITTEST_TEST_CASE_CLASS: &str = "TestCase";
static UNITTEST_TEST_CASE_PREFIX: &str = "test_";

// with Django
static UNITTEST_DJANGO_TRANSACTION_TEST_CASE_CLASS: &str = "TransactionTestCase";
static UNITTEST_DJANGO_SIMPLE_TEST_CASE: &str = "SimpleTestCase";
static UNITTEST_DJANGO_LIVE_SERVER_TEST_CASE: &str = "LiveServerTestCase";

pub fn enumerate_tests(statements: &Statements) -> HashSet<String> {
    let mut result: HashSet<String> = HashSet::new();

    // this module
    {
        let splitted = statements.module.split('.').collect::<Vec<&str>>();
        if splitted.len() > 0 {
            for (i, _) in splitted.iter().enumerate() {
                let s = splitted[..i].to_vec().join(".");
                if result.contains(&s) || s.len() <= 0 {
                    continue;
                }
                result.insert(s);
            }
        }
        result.insert(make_test_name(statements, None, None));

        //     // let mut nodes = [".."].repeat(*level as usize - 1).to_vec();
        //     // match name {
        //     //     Some(n) => nodes.extend(n.split('.').clone()),
        //     //     None => {}
        //     // };

        //     // let path_str = nodes.join("/") + ".py";
        //     // PathBuf::from_str(path_str.as_str()).unwrap()
    }

    // println!("{:?}", statements.import_table);

    // root function
    for f in statements.methods.iter() {
        if f.name.starts_with(UNITTEST_TEST_CASE_PREFIX) {
            // has_unittest_skip_for_func(&f);
            result.insert(make_test_name(&statements, None, Some(&f)));
        }
    }

    // class methods
    for c in statements.classes.iter() {
        // this class
        result.insert(make_test_name(statements, Some(&c), None));

        for f in c.body.iter() {
            if !f.is_function_def_stmt() {
                continue;
            }

            let func = f.as_function_def_stmt().unwrap();
            if func.name.starts_with(UNITTEST_TEST_CASE_PREFIX) {
                // has_unittest_skip_for_func(&func);
                result.insert(make_test_name(&statements, Some(&c), Some(&func)));
            }
        }
    }

    // result.iter().map(|e| e.to_owned()).collect()
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
