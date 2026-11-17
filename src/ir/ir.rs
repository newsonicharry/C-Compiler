use crate::{
    ir::instructions::{
        AllocationInfo, IRType, Instruction, Module, Program, RValue, SignedIRType, VRegisterID,
        VRegisterIdentifierLookup, VRegisterTypeLookup,
    },
    parser::{
        expression_parser::ExprNode,
        nodes::{
            AST,
            GlobalNode::{self},
        },
        simple_type::{BaseType, Signedness},
        type_parser::TypeNode,
    },
    semantics::semantics::Semantics,
};

pub struct IRParser<'a> {
    pub(super) ast: &'a AST,
    pub(super) semantics: &'a Semantics,
    pub(super) program: Program,
    pub(super) curr_id: VRegisterID,
    pub(super) identifier_lookup: VRegisterIdentifierLookup,
    pub(super) type_lookup: VRegisterTypeLookup,
    // str_container:
}

impl<'a> IRParser<'a> {
    pub fn new(ast: &'a AST, semantics: &'a Semantics) -> IRParser<'a> {
        IRParser {
            ast,
            semantics,
            program: Program(Vec::new()),
            curr_id: VRegisterID(0),
            identifier_lookup: VRegisterIdentifierLookup::default(),
            type_lookup: VRegisterTypeLookup::default(),
        }
    }

    pub fn parse(&mut self) {
        for global_node in &self.ast.0 {
            match global_node {
                GlobalNode::Function { .. } => self.emit_function_node(&global_node),
                _ => todo!(),
            }
        }
    }

    pub fn program_to_llvm_ir(&self) -> String {
        let mut output = String::new();

        output.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128\"\n");
        output.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");

        for module in &self.program.0 {
            match module {
                Module::FunctionDef {
                    return_type,
                    name,
                    instructions,
                    params,
                } => {
                    output.push_str(match instructions {
                        Some(_) => "define dso_local ",
                        _ => "declare ",
                    });

                    output.push_str(&format!("{return_type} @{name}("));

                    for (i, param) in params.iter().enumerate() {
                        output.push_str(&format!("{} %{}", param.ir_type, i + 1));

                        if i != params.len() - 1 {
                            output.push_str(", ");
                        }
                    }

                    output.push_str(")");

                    if let Some(instructions) = instructions {
                        output.push_str("{\nentry:\n");

                        instructions
                            .iter()
                            .for_each(|element| output.push_str(&element.as_llvm_ir()));
                        output.push_str("  unreachable\n}\n\n");
                    } else {
                        output.push_str("\n\n");
                    }
                }
            }
        }

        output
    }

    fn emit_function_node(&mut self, func_node: &GlobalNode) {
        let GlobalNode::Function {
            signature, body, ..
        } = func_node
        else {
            unreachable!();
        };

        let TypeNode::Function {
            name,
            return_type,
            parameters,
            ..
        } = &**signature
        else {
            unreachable!()
        };

        let mut body_instructions = Vec::new();

        if body.is_none() {
            let func_dec = Module::FunctionDef {
                // this takes an instructions as if it could emit something
                // but for a function return type this would be illegal
                return_type: self.lower_type(return_type, &mut body_instructions).ir_type,
                name: name.clone(),
                instructions: None,
                params: parameters
                    .iter()
                    .map(|x| self.lower_type(&x.0, &mut body_instructions))
                    .collect(),
            };

            self.program.0.push(func_dec);

            return;
        }

        self.curr_id = VRegisterID(0);
        let mut params = Vec::new();
        for (parameter, _) in parameters {
            self.curr_id.next();
            let type_info = self.lower_type(parameter, &mut body_instructions);
            params.push(type_info.clone());
            self.type_lookup.add_type(self.curr_id, type_info);
        }

        for (id, (param, (_, semantic_info))) in params.iter().zip(parameters.iter()).enumerate() {
            let param_v_register = VRegisterID(id as u64 + 1);
            let alloc = Instruction::Alloca {
                size: param.ir_type.clone(),
                v_register: self.curr_id.next(),
            };

            let store = Instruction::Store {
                size: param.ir_type.clone(),
                value: RValue::VRegister(param_v_register),
                store_ptr: self.curr_id,
            };

            body_instructions.push(alloc);
            body_instructions.push(store);
            // self.type_lookup.add_type(self.curr_id, param.clone());

            self.identifier_lookup.add_identifier(
                semantic_info.symbol_id.unwrap(),
                AllocationInfo {
                    allocated_id: self.curr_id,
                },
            );
        }

        let Some(statements) = body else {
            unreachable!();
        };
        self.emit_statement_node(statements, &mut body_instructions);

        let func_def = Module::FunctionDef {
            return_type: self.lower_type(return_type, &mut body_instructions).ir_type,
            name: name.clone(),
            instructions: Some(body_instructions),
            params,
        };

        self.program.0.push(func_def);
    }

    pub(super) fn lower_type(
        &mut self,
        type_node: &TypeNode,
        instructions: &mut Vec<Instruction>,
    ) -> SignedIRType {
        let create = |ir_type: IRType, signedness: Signedness| SignedIRType {
            ir_type,
            signedness,
        };

        match type_node {
            TypeNode::Variable { held_value, .. } => self.lower_type(held_value, instructions),
            TypeNode::Pointer { .. } => create(IRType::Ptr, Signedness::NoSign),
            TypeNode::Normal { held_type, .. } => match held_type.base_type.unwrap() {
                BaseType::_Bool => create(IRType::I1, held_type.signedness),
                BaseType::Char => create(IRType::I8, held_type.signedness),
                BaseType::Short => create(IRType::I16, held_type.signedness),
                BaseType::Int => create(IRType::I32, held_type.signedness),
                BaseType::Long => create(IRType::I64, held_type.signedness),

                BaseType::Float => create(IRType::Float, Signedness::NoSign),
                BaseType::Double => create(IRType::Double, Signedness::NoSign),
                BaseType::LongDouble => create(IRType::X86Fp80, Signedness::NoSign),

                BaseType::Void => create(IRType::Void, Signedness::Unsigned),
            },
            TypeNode::Array { expr, held_value } => {
                let num_elements = match expr {
                    ExprNode::Integer { num } => RValue::Literal(num.value as i64),
                    _ => RValue::VRegister(self.emit_expr(expr, instructions)),
                };

                create(
                    IRType::Arr {
                        num_elements,
                        arr_type: Box::new(self.lower_type(held_value, instructions).ir_type),
                    },
                    Signedness::NoSign,
                )
            }

            // TypeNode::Array { .. } => IRType::Ptr,
            _ => todo!(),
        }
    }
}
