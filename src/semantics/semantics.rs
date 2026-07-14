use std::{collections::HashMap, fmt::format, thread::Scope};

use crate::parser::{jump_label::JumpLabel, tag_types::helper::TagTypeData, type_parser::TypeNode};

#[derive(Clone, Default, Debug)]
pub struct SemanticInfo {
    type_id: Option<u32>,
    symbol_id: Option<u32>,
}

macro_rules! create_id {
    ($name:tt) => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $name(u32);

        impl $name {
            pub fn as_usize(&self) -> usize {
                self.0 as usize
            }
        }
    };
}

create_id!(SymbolId);
create_id!(ScopeId);
create_id!(TypeId);

#[derive(Clone, Debug)]
pub enum SymbolKind {
    Typedef,
    Variable,
    EnumConstant,
    Function,
    Union,
    Struct,
    Enum,
    Goto,
    Case,
    Default,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub type_id: TypeId,
}

#[derive(Default, Debug)]
struct SymbolTable {
    table: Vec<Symbol>,
}

impl SymbolTable {
    pub fn next_symbol_id(&self) -> SymbolId {
        SymbolId(self.table.len() as u32)
    }

    pub fn add_symbol(&mut self, name: &str, kind: SymbolKind, type_id: TypeId) {
        let symbol = Symbol {
            name: name.to_string(),
            kind,
            type_id,
        };

        self.table.push(symbol);
    }

    fn lookup(&self, symbol_id: SymbolId) -> &Symbol {
        &self.table[symbol_id.as_usize()]
    }
}

#[derive(Clone, Debug)]
pub enum TypeTableValue {
    // where each type corresponds to a namespace,
    // except for members but members are stored within tag type
    Identifier(Box<TypeNode>),
    TagType(TagTypeData),
    Label(JumpLabel),
}

#[derive(Default, Debug)]
struct TypeTable {
    table: Vec<TypeTableValue>,
}

impl TypeTable {
    pub fn next_type_id(&self) -> TypeId {
        TypeId(self.table.len() as u32)
    }

    pub fn add_type(&mut self, type_value: &TypeTableValue) {
        self.table.push(type_value.clone());
    }

    fn lookup(&self, type_id: TypeId) -> &TypeTableValue {
        &self.table[type_id.as_usize()]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Namespace {
    Label,
    TagType,
    Member,
    Identifier,
}

#[derive(Default, Debug)]
struct ScopeTable {
    table: HashMap<(Namespace, String), SymbolId>,

    scope_id: ScopeId,
    parent_id: Option<ScopeId>,
    children_id: Vec<ScopeId>,
}

impl ScopeTable {
    pub fn new(new_id: ScopeId, parent: Option<ScopeId>) -> ScopeTable {
        Self {
            table: HashMap::new(),
            scope_id: new_id,
            parent_id: parent,
            children_id: Vec::new(),
        }
    }

    pub fn add_identifier(
        &mut self,
        namespace: &Namespace,
        name: &str,
        symbol_id: SymbolId,
    ) -> Result<(), String> {
        let key = (namespace.clone(), name.to_string());

        if self.table.contains_key(&key) {
            return Err(format!(
                "Identifier of name {} already exists in scope",
                name
            ));
        }

        self.table.insert(key, symbol_id);

        Ok(())
    }

    fn add_child(&mut self, child_id: ScopeId) {
        self.children_id.push(child_id);
    }

    fn last_child(&self) -> ScopeId {
        *self.children_id.last().unwrap()
    }

    fn has_no_children(&self) -> bool {
        self.children_id.is_empty()
    }

    fn lookup(&self, namespace: &Namespace, name: &str) -> Option<SymbolId> {
        self.table.get(&(*namespace, name.to_string())).cloned()
    }
}

#[derive(Debug)]
pub struct Semantics {
    scopes: Vec<ScopeTable>,
    curr_scope_id: ScopeId,

    symbols: SymbolTable,
    types: TypeTable,

    anonymous_id: u32,
}

impl Default for Semantics {
    fn default() -> Self {
        Self {
            scopes: vec![ScopeTable::new(ScopeId(0), None)],
            curr_scope_id: ScopeId(0),
            symbols: SymbolTable::default(),
            types: TypeTable::default(),
            anonymous_id: 0,
        }
    }
}

impl Semantics {
    // (WARNING: THIS CODE MIGHT BE UNNECESSARY)
    /// Returns the new name of an anonymous tag type
    /// Internally all struct must have a name, and an anonymous struct will be given one
    pub fn generate_new_name(&mut self) -> String {
        // dashes are used to differentiate between user defined variables / tag types and compiler defined tag types
        let name = format!("Anon-TagType-{}", self.anonymous_id);
        self.anonymous_id += 1;
        name
    }

    fn get_next_scope_id(&self) -> ScopeId {
        ScopeId(self.scopes.len() as u32)
    }

    fn curr_scope(&mut self) -> &mut ScopeTable {
        &mut self.scopes[self.curr_scope_id.as_usize()]
    }

    /// Enters a deeper layer of scope
    /// If no deeper layer exists then one will be created
    pub fn enter_scope(&mut self) {
        if self.curr_scope().has_no_children() {
            self.expand_scope();
            return;
        }

        self.curr_scope_id = self.curr_scope().last_child();
    }

    fn expand_scope(&mut self) {
        let next_id = self.get_next_scope_id();
        let parent_id = self.curr_scope_id;

        self.scopes.push(ScopeTable::new(next_id, Some(parent_id)));

        self.curr_scope().add_child(next_id);
        self.curr_scope_id = next_id;
    }

    pub fn leave_scope(&mut self) {
        if let Some(parent_id) = self.curr_scope().parent_id {
            self.curr_scope_id = parent_id;
        }
    }

    fn get_name_from_type_value(&mut self, type_value: &TypeTableValue) -> String {
        match type_value {
            TypeTableValue::Identifier(type_node) => match &**type_node {
                TypeNode::Variable { name, .. }
                | TypeNode::Function { name, .. }
                | TypeNode::TagType { name, .. } => name.to_owned(),

                _ => panic!("Given invalid type node"),
            },

            TypeTableValue::TagType(tag_type_data) => match &tag_type_data.name {
                Some(name) => name.to_owned(),
                _ => self.generate_new_name(),
            },

            TypeTableValue::Label(jump_label) => match jump_label {
                _ => todo!(), // JumpLabel::Goto(goto_name) => goto_name.to_owned(),
            },
        }
    }

    pub fn add_identifier(
        &mut self,
        name: &str,
        type_value: &TypeTableValue,
        symbol_kind: SymbolKind,
    ) {
        let namespace = match type_value {
            TypeTableValue::Identifier(..) => Namespace::Identifier,
            TypeTableValue::TagType(..) => Namespace::TagType,
            TypeTableValue::Label(..) => Namespace::Label,
        };

        let symbol_id = self.symbols.next_symbol_id();
        // let name = self.get_name_from_type_value(type_value);

        self.curr_scope()
            .add_identifier(&namespace, &name, symbol_id);

        let type_id = self.types.next_type_id();
        self.types.add_type(type_value);

        self.symbols.add_symbol(&name, symbol_kind, type_id);
    }

    pub fn check_typedef(&mut self, identifier: &str) -> Option<&TypeNode> {
        if let Some(symbol) = self
            .check_symbol(identifier, Namespace::Identifier)
            .cloned()
        {
            if matches!(symbol.kind, SymbolKind::Typedef) {
                let type_value = self.types.lookup(symbol.type_id);

                let TypeTableValue::Identifier(type_node) = type_value else {
                    unreachable!()
                };

                return Some(type_node);
            }

            // the identifier exists but its not a typedef meaning the identifier is not a type
            return None;
        }

        return None;
    }

    pub fn check_symbol(&mut self, identifier: &str, namespace: Namespace) -> Option<&Symbol> {
        let original_scope_id = self.curr_scope_id;

        loop {
            // check if the potential typedef exists in the current scope
            if let Some(symbol_id) = self.curr_scope().lookup(&namespace, identifier) {
                self.curr_scope_id = original_scope_id;

                let symbol = self.symbols.lookup(symbol_id);

                return Some(symbol);
            }

            // if the potential typedef doenst exist in this scope check its parents scope
            if let Some(parent_id) = self.curr_scope().parent_id {
                self.curr_scope_id = parent_id;
            }
            // if its the global scope and no typedef has yet been found return None
            else {
                self.curr_scope_id = original_scope_id;
                return None;
            }
        }
    }
}
