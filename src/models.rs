use std::collections::HashMap;

use rustpython_parser::{
    ast::{
        self, Alias, Stmt, StmtClassDef, StmtFunctionDef, StmtImport, StmtImportFrom, StmtRaise,
    },
    Parse,
};

#[derive(Clone, Debug, Default)]
pub struct Statements {
    pub module: String,
    pub import_table: ImportMap,
    pub imports: Vec<StmtImport>,
    pub import_from: Vec<StmtImportFrom>,
    pub classes: Vec<StmtClassDef>,
    pub methods: Vec<StmtFunctionDef>,
    pub raises: Vec<StmtRaise>,
}

#[derive(Debug)]
pub struct Import {
    pub name: Option<String>,
    pub alias: Vec<Name>,
    pub level: u32,
    pub from_import: bool,
}

impl Default for Import {
    fn default() -> Self {
        Self {
            name: Some("".to_owned()),
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
        result.name = Some(first.name.to_string());
        result.alias.push(first.into());

        // println!("from<ImportStmt>: {:?}", stmt);

        result
    }
}

impl From<&StmtImportFrom> for Import {
    fn from(stmt: &StmtImportFrom) -> Self {
        let mut result = Self::default();
        result.from_import = true;

        result.name = match &stmt.module {
            Some(m) => Some(m.to_string()),
            None => None,
        };
        result.level = match stmt.level {
            Some(v) => v.to_u32(),
            None => 0,
        };

        for name in stmt.names.iter() {
            // println!("!! nm: {:?}", name.name);
            // println!("!! as: {:?}", name.asname);
            result.alias.push(name.into());
        }

        // println!("from<FromImportStmt>: {:?}", stmt);

        result
    }
}

impl Import {
    pub fn print_pretty(&self) {
        println!("name: {:?}", self.name);
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

#[derive(Debug)]
pub struct Name {
    pub name: String,
    pub alias: Option<String>,
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

pub type ModuleName = Option<String>;
pub type AliasName = Option<String>;
pub type ExportName = String;
pub type Level = u32;
pub type Module = (ModuleName, Level, bool);
pub type Exports = HashMap<ExportName, AliasName>;
pub type ImportMap = HashMap<Module, Option<Exports>>;
