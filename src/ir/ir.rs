use std::fmt::Display;

use crate::{
    lexer::{language_features::OperatorTypes, lexer::TokenTypes},
    parser::{
        expression_parser::ExprNode,
        nodes::{AST, GlobalNode, StatementNode},
        type_parser::TypeNode,
    },
    semantics::semantics::Semantics,
};

struct Program(pub Vec<Module>);

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct VRegisterID(pub u32);

impl Display for VRegisterID {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(display, "{}", self.0)
    }
}

impl VRegisterID {
    pub fn next(&mut self) -> VRegisterID {
        self.0 += 1;
        *self
    }

    pub fn prev(&self) -> VRegisterID {
        VRegisterID(self.0 - 1)
    }
}

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
    Void,
}

impl IRType {
    fn alignment(&self) -> u8 {
        match self {
            Self::I1 => 1,
            Self::I8 => 1,
            Self::I16 => 2,
            Self::I32 => 4,
            Self::I64 => 8,
            Self::Float => 4,
            Self::Double => 8,
            Self::X86Fp80 => 16,
            Self::Ptr => 8,
            Self::Void => 0,
        }
    }
}

impl Display for IRType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::I1 => "i1",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::Float => "float",
            Self::Double => "double",
            Self::X86Fp80 => "x86_fp80",
            Self::Ptr => "ptr",
            Self::Void => "void",
        };

        write!(display, "{output}")
    }
}

enum Module {
    FunctionDef {
        return_type: IRType,
        name: String,
        instructions: Vec<Instruction>,
    },
}

enum RValue {
    Pointer(VRegisterID),
    Literal(u64),
    Void,
}

impl Display for RValue {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Pointer(id) => format!("%{}", id.0),
            Self::Literal(value) => value.to_string(),
            Self::Void => String::new(),
        };

        write!(display, "{output}")
    }
}

enum Instruction {
    Return {
        size: IRType,
        value: RValue,
    },
    Alloca {
        size: IRType,
        v_register: VRegisterID,
    },
    Store {
        size: IRType,
        value: RValue,
        store_ptr: VRegisterID,
    },
    Load {
        size: IRType,
        value: VRegisterID,
        v_register: VRegisterID,
    },
    Add {
        size: IRType,
        left: RValue,
        right: RValue,
        v_register: VRegisterID,
    },
    Sub {
        size: IRType,
        left: RValue,
        right: RValue,
        v_register: VRegisterID,
    },
    Mul {
        size: IRType,
        left: RValue,
        right: RValue,
        v_register: VRegisterID,
    },
    Branch,
    AllocaConstArray {},
    AllocaVLAArray {},
}

impl Instruction {
    fn as_llvm_ir(&self) -> String {
        match self {
            Self::Return { size, value } => match size {
                IRType::Void => format!("  ret {size}\n"),
                _ => format!("  ret {size} {value}\n"),
            },

            Self::Alloca { size, v_register } => {
                format!(
                    "  %{v_register} = alloca {size}, align {}\n",
                    size.alignment()
                )
            }

            Self::Store {
                size,
                value,
                store_ptr,
            } => format!(
                "  store {size} {value}, ptr %{store_ptr}, align {}\n",
                size.alignment()
            ),

            Self::Load {
                size,
                value,
                v_register,
            } => format!(
                "  %{v_register} = load {size}, ptr %{value}, align {}\n",
                size.alignment()
            ),

            Self::Add {
                size,
                left,
                right,
                v_register,
            } => format!("  %{v_register} = add {size} {left}, {right}\n"),

            Self::Sub {
                size,
                left,
                right,
                v_register,
            } => format!("  %{v_register} = sub {size} {left}, {right}\n"),

            Self::Mul {
                size,
                left,
                right,
                v_register,
            } => format!("  %{v_register} = mul {size} {left}, {right}\n"),

            _ => todo!(),
        }
    }
}

pub struct IRParser<'a> {
    ast: &'a AST,
    semantics: &'a Semantics,
    program: Program,
    curr_ptr_id: VRegisterID,
}

impl<'a> IRParser<'a> {
    pub fn new(ast: &'a AST, semantics: &'a Semantics) -> IRParser<'a> {
        IRParser {
            ast,
            semantics,
            program: Program(Vec::new()),
            curr_ptr_id: VRegisterID(0),
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

        for module in &self.program.0 {
            match module {
                Module::FunctionDef {
                    return_type,
                    name,
                    instructions,
                } => {
                    output.push_str(&format!(
                        "define dso_local {} @{}() {{\n",
                        return_type, name
                    ));
                    instructions
                        .iter()
                        .for_each(|element| output.push_str(&element.as_llvm_ir()));
                    output.push_str("}\n\n");
                }
            }
        }

        output
    }

    fn emit_binary_expr(&mut self, expr: &ExprNode, instructions: &mut Vec<Instruction>) {
        let ExprNode::Binary {
            left,
            operator,
            right,
        } = expr
        else {
            unreachable!()
        };

        self.emit_expr(left, instructions);
        let left_ptr_id = self.curr_ptr_id;

        self.emit_expr(right, instructions);
        let right_ptr_id = self.curr_ptr_id;

        let result_instruction = match operator {
            TokenTypes::Operator(operator) => match operator {
                OperatorTypes::Plus => Instruction::Add {
                    size: IRType::I32,
                    left: RValue::Pointer(left_ptr_id),
                    right: RValue::Pointer(right_ptr_id),
                    v_register: self.curr_ptr_id.next(),
                },

                OperatorTypes::Minus => Instruction::Sub {
                    size: IRType::I32,
                    left: RValue::Pointer(left_ptr_id),
                    right: RValue::Pointer(right_ptr_id),
                    v_register: self.curr_ptr_id.next(),
                },

                OperatorTypes::Star => Instruction::Mul {
                    size: IRType::I32,
                    left: RValue::Pointer(left_ptr_id),
                    right: RValue::Pointer(right_ptr_id),
                    v_register: self.curr_ptr_id.next(),
                },

                _ => todo!(),
            },
            _ => todo!(),
        };

        instructions.push(result_instruction);
    }

    fn emit_iteral_expr(&mut self, expr: &ExprNode, instructions: &mut Vec<Instruction>) {
        let literal = match expr {
            ExprNode::Integer { num } => RValue::Literal(num.value as u64),
            _ => todo!(),
        };

        let alloca = Instruction::Alloca {
            size: IRType::I32,
            v_register: self.curr_ptr_id.next(),
        };

        let store = Instruction::Store {
            size: IRType::I32,
            value: literal,
            store_ptr: self.curr_ptr_id,
        };

        let load = Instruction::Load {
            size: IRType::I32,
            value: self.curr_ptr_id,
            v_register: self.curr_ptr_id.next(),
        };

        instructions.push(alloca);
        instructions.push(store);
        instructions.push(load);
    }

    fn emit_expr(&mut self, expr: &ExprNode, instructions: &mut Vec<Instruction>) {
        match expr {
            ExprNode::Binary { .. } => self.emit_binary_expr(expr, instructions),
            ExprNode::Integer { .. } | ExprNode::Float { .. } | ExprNode::Char { .. } => {
                self.emit_iteral_expr(expr, instructions)
            }
            _ => todo!(),
        }
    }

    fn emit_return(&mut self, statement: &StatementNode, instructions: &mut Vec<Instruction>) {
        let StatementNode::Return(Some(expr)) = statement else {
            instructions.push(Instruction::Return {
                size: IRType::Void,
                value: RValue::Void,
            });

            return;
        };

        self.emit_expr(expr, instructions);

        instructions.push(Instruction::Return {
            size: IRType::I32,
            value: RValue::Pointer(self.curr_ptr_id),
        });
    }

    fn emit_statement_node(
        &mut self,
        statement: &StatementNode,
        instructions: &mut Vec<Instruction>,
    ) {
        match statement {
            StatementNode::Block { statements, .. } => statements
                .iter()
                .for_each(|x| self.emit_statement_node(x, instructions)),

            StatementNode::Return { .. } => self.emit_return(statement, instructions),

            StatementNode::Expression(expr) => self.emit_expr(expr, instructions),

            _ => todo!(),
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

        let mut body_instructions = Vec::new();
        if let Some(statements) = body {
            self.emit_statement_node(statements, &mut body_instructions);
        }

        let func_def = Module::FunctionDef {
            return_type: IRType::I32,
            name: name.clone(),
            instructions: body_instructions,
        };

        self.program.0.push(func_def);
    }
}
