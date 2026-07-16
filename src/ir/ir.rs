use crate::{
    parser::{
        nodes::{AST, GlobalNode},
        type_parser::TypeNode,
    },
    semantics::semantics::Semantics,
};

struct Program(pub Vec<Module>);

enum IRType {
    I1,
    I8,
    I16,
    I32,
    I64,
    Float,
    Double,
    X86Fp80,
    Ptr,
}

enum Module {
    FunctionDef {
        return_type: IRType,
        name: String,
        instructions: Vec<Instruction>,
    },
}

enum Instruction {
    Return,
    Branch,
}

struct IRParser<'a> {
    ast: &'a AST,
    semantics: Semantics,
    program: Program,
}

impl<'a> IRParser<'a> {
    pub fn parse(&mut self) {
        for global_node in &self.ast.0 {
            match global_node {
                GlobalNode::Function { .. } => self.emit_function_node(&global_node),
                _ => todo!(),
            }
        }
    }

    fn emit_function_node(&mut self, func_node: &GlobalNode) {
        let GlobalNode::Function {
            signature,
            body,
            semantic_info,
        } = func_node
        else {
            unreachable!();
        };

        let TypeNode::Function {
            name,
            return_type,
            parameters,
            is_variadic,
        } = &**signature
        else {
            unreachable!()
        };

        let func_def = Module::FunctionDef {
            return_type: IRType::I32,
            name: name.clone(),
            instructions: Vec::new(),
        };

        self.program.0.push(func_def);
    }
}
