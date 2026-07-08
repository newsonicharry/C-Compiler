use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

use crate::parser::type_parser::TypeNode;

static ANONYMOUS_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug, PartialEq)]
pub enum IdentifierType {
    Typedef(TypeNode),
    Variable(TypeNode),
    EnumConstant,
    Function(TypeNode),
}

// enum TagType {
//     Union,
//     Struct,
//     Enum,
// }

enum MemberType {
    Union,
    Struct,
}

enum LabelType {
    Goto,
    Case,
    Default,
}

#[derive(Default, Debug)]
struct SymbolTable {
    pub identifiers: HashMap<String, IdentifierType>,
    // pub tags: HashMap<String, TagType>,
    // pub members: HashMap<String, MemberType>,
    // pub labels: HashMap<String, LabelType>,
    curr_depth: usize,
    next_tables: Vec<SymbolTable>,
}

impl SymbolTable {
    pub fn new(depth: usize) -> Self {
        let mut symbol_table = SymbolTable::default();
        symbol_table.curr_depth = depth;
        symbol_table
    }

    pub fn get_identifiers(
        &self,
        depth: usize,
        identifier: &str,
        mut found_type: Option<IdentifierType>,
    ) -> Option<IdentifierType> {
        let value = self.identifiers.get(identifier);

        if value.is_some() {
            found_type = value.cloned();
        }

        if self.curr_depth == depth {
            return found_type;
        }

        self.next_tables
            .last()
            .unwrap()
            .get_identifiers(depth, identifier, found_type)
    }

    /// Enters the newest scope created at a given depth
    pub fn enter_scope(&mut self, depth: usize) -> &mut Self {
        if self.curr_depth == depth {
            return self;
        }

        if self.next_tables.is_empty() {
            panic!("Cannot enter scope that does not exist")
        }

        self.next_tables.last_mut().unwrap().enter_scope(depth)
    }

    /// Adds a new scope to an exising depth
    /// This is used when one wants multiple scopes to exist at a given depth
    /// e.g. an if statement scope and a while scope existing together in a function scope
    pub fn add_scope(&mut self, depth: usize) -> &mut Self {
        if depth == 0 {
            panic!("Cannot add another global scope, there can only be one")
        }

        if (self.curr_depth + 1) == depth {
            self.next_tables.push(Self::new(self.curr_depth + 1));
            return self.next_tables.last_mut().unwrap();
        }

        self.next_tables.last_mut().unwrap().add_scope(depth)
    }
}

#[derive(Default, Debug)]
pub struct Semantics {
    full_symbol_table: SymbolTable,
    curr_depth: usize,
}

impl Semantics {
    pub fn push_scope(&mut self) {
        self.curr_depth += 1;
        self.full_symbol_table.add_scope(self.curr_depth);
    }

    pub fn leave_scope(&mut self) {
        self.curr_depth -= 1;
    }

    pub fn check_identifier(&self, identifier: &str) -> Option<IdentifierType> {
        self.full_symbol_table
            .get_identifiers(self.curr_depth, identifier, None)
    }

    pub fn add_identifier_symbol(
        &mut self,
        name: &str,
        identifier_type: &IdentifierType,
    ) -> Result<(), String> {
        let curr_scope = self.full_symbol_table.enter_scope(self.curr_depth);
        let identifiers = &mut curr_scope.identifiers;

        if identifiers.contains_key(name) {
            return Err(format!("Redefinition of {name} in current scope"));
        }

        identifiers.insert(name.to_string(), identifier_type.clone());
        Ok(())
    }

    pub fn add_variable(&mut self, variable: &TypeNode) -> Result<(), String> {
        let TypeNode::Variable { name, .. } = variable else {
            panic!("Expected variable in add_var function to symbol table")
        };

        self.add_identifier_symbol(name, &IdentifierType::Variable(variable.clone()))
    }

    pub fn add_typedef(&mut self, typedef: &TypeNode) -> Result<(), String> {
        let name = match typedef {
            TypeNode::Variable { name, .. } | TypeNode::Function { name, .. } => name,

            _ => panic!("Expected typedef value in add_typedef function to symbol table"),
        };

        self.add_identifier_symbol(name, &IdentifierType::Typedef(typedef.clone()))
    }

    pub fn add_enum_constant(&mut self, constant_name: &str) -> Result<(), String> {
        self.add_identifier_symbol(constant_name, &IdentifierType::EnumConstant)
    }

    // pub fn add_tag_type()

    // pub fn add_scope(&mut self) {
    //     self.full_symbol_table.add_scope(self.curr_depth);
    // }
    // enter scope
    // leave scope

    // check (symbol)
    // push (symbol)
}
