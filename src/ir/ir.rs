use std::fmt::{Display, write};

use crate::{
    lexer::{
        language_features::{AssignmentTypes, OperatorTypes},
        lexer::TokenTypes,
    },
    parser::{
        expression_parser::ExprNode,
        nodes::{
            AST,
            GlobalNode::{self, Initalizer},
            StatementNode,
        },
        simple_type::{BaseType, Signedness, SimpleType},
        type_parser::TypeNode,
    },
    semantics::semantics::{SemanticInfo, Semantics, SymbolID, TypeTableValue},
};

struct Program(pub Vec<Module>);

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct VRegisterID(pub u64);

impl Display for VRegisterID {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(display, "{}", self.0)
    }
}

impl VRegisterID {
    pub fn next(&mut self) -> VRegisterID {
        // let temp = *self;
        self.0 += 1;
        *self
    }

    pub fn prev(&self) -> VRegisterID {
        VRegisterID(self.0 - 1)
    }
}

#[derive(Clone, Copy, Default)]
enum IRType {
    I1,
    I8,
    I16,
    #[default]
    I32,
    I64,
    Float,
    Double,
    X86Fp80,
    Ptr,
    Void,
}

impl IRType {
    fn num_bits(&self) -> u8 {
        match self {
            Self::I1 => 1,
            Self::I8 => 8,
            Self::I16 => 16,
            Self::I32 => 32,
            Self::I64 => 64,
            Self::Float => 32,
            Self::Double => 64,
            Self::X86Fp80 => 128,
            Self::Ptr => 64,
            Self::Void => 0,
        }
    }

    fn alignment(&self) -> u8 {
        self.num_bits().div_ceil(8)
    }

    fn is_floating(&self) -> bool {
        match self {
            Self::Float | Self::Double | Self::X86Fp80 => true,
            _ => false,
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

#[derive(Default, Clone, Copy)]
struct SignedIRType {
    ir_type: IRType,
    signedness: Signedness,
}

enum Module {
    FunctionDef {
        return_type: IRType,
        name: String,
        instructions: Vec<Instruction>,
        params: Vec<SignedIRType>,
    },
}

enum RValue {
    VRegister(VRegisterID),
    Literal(i64),
    Void,
}

impl Display for RValue {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::VRegister(id) => format!("%{}", id.0),
            Self::Literal(value) => value.to_string(),
            Self::Void => String::new(),
        };

        write!(display, "{output}")
    }
}

enum BinaryKind {
    Add,
    Sub,
    Mul,
    Udiv,
    Sdiv,
    Fadd,
    Fsub,
    Fmul,
    Fdiv,
    And,
    Or,
    Xor,
    Shl,
    Lshr,
    Ashr,
}

impl Display for BinaryKind {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Udiv => "udiv",
            Self::Sdiv => "sdiv",
            Self::Fadd => "fadd",
            Self::Fsub => "fsub",
            Self::Fmul => "fmul",
            Self::Fdiv => "fdiv",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Shl => "shl",
            Self::Lshr => "lshr",
            Self::Ashr => "ashr",
        };

        write!(display, "{output}")
    }
}

enum CastKind {
    Zext,
    Sext,
    Trunc,
    Sitofp,
    Uitofp,
    Fptosi,
    Fptoui,
    Fpext,
    Fptrunc,
}

impl Display for CastKind {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Zext => "zext",
            Self::Sext => "sext",
            Self::Trunc => "trunc",
            Self::Sitofp => "sitofp",
            Self::Uitofp => "uitofp",
            Self::Fptosi => "fptosi",
            Self::Fptoui => "fptoui",
            Self::Fpext => "fpext",
            Self::Fptrunc => "fptrunc",
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
        value_ptr: VRegisterID,
        v_register: VRegisterID,
    },
    BinaryOp {
        kind: BinaryKind,
        size: IRType,
        left: RValue,
        right: RValue,
        v_register: VRegisterID,
    },
    Cast {
        kind: CastKind,
        from_size: IRType,
        to_size: IRType,
        value: RValue,
        v_register: VRegisterID,
    },
    Call {
        func_name: String,
        args: Vec<(IRType, RValue)>,
        return_value: IRType,
        v_register: VRegisterID,
    },
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
                value_ptr: value,
                v_register,
            } => format!(
                "  %{v_register} = load {size}, ptr %{value}, align {}\n",
                size.alignment()
            ),

            Self::BinaryOp {
                kind,
                size,
                left,
                right,
                v_register,
            } => format!("  %{v_register} = {kind} {size} {left}, {right}\n"),

            Self::Cast {
                kind,
                v_register,
                from_size,
                to_size,
                value,
            } => format!("  %{v_register} = {kind} {from_size} {value} to {to_size} \n"),

            Self::Call {
                func_name,
                args,
                return_value,
                v_register,
            } => {
                let mut output = format!("  %{v_register} = call {return_value} @{func_name}(");

                for (i, (ir_type, r_value)) in args.iter().enumerate() {
                    output.push_str(&format!("{ir_type} {r_value}"));

                    if i != args.len() - 1 {
                        output.push_str(", ");
                    }
                }

                output.push_str(")\n");
                output
            }
        }
    }
}

#[derive(Default, Copy, Clone)]
struct AllocationInfo {
    loaded_id: VRegisterID,
    allocated_id: VRegisterID,
    type_info: SignedIRType,
}

#[derive(Default)]
struct VRegisterIdentifierLookup {
    lookup: Vec<AllocationInfo>,
}

impl VRegisterIdentifierLookup {
    pub fn add_identifier(&mut self, symbol_id: SymbolID, allocation: AllocationInfo) {
        if self.lookup.len() <= symbol_id.0 as usize {
            self.lookup
                .resize(symbol_id.0 as usize + 1, AllocationInfo::default());
        }

        self.lookup[symbol_id.0 as usize] = allocation;
    }

    pub fn update_v_register(&mut self, symbol_id: SymbolID, register_id: VRegisterID) {
        self.lookup[symbol_id.0 as usize].loaded_id = register_id;
    }

    pub fn lookup_v_register(&mut self, symbol_id: SymbolID) -> AllocationInfo {
        self.lookup[symbol_id.0 as usize]
    }
}

#[derive(Default)]
struct VRegisterTypeLookup {
    lookup: Vec<SignedIRType>,
}

impl VRegisterTypeLookup {
    pub fn add_type(&mut self, v_register: VRegisterID, signed_ir_type: SignedIRType) {
        if self.lookup.len() <= v_register.0 as usize {
            self.lookup
                .resize(v_register.0 as usize + 1, SignedIRType::default());
        }

        self.lookup[v_register.0 as usize] = signed_ir_type;
    }

    pub fn lookup(&mut self, v_register: VRegisterID) -> &SignedIRType {
        &self.lookup[v_register.0 as usize]
    }
}

pub struct IRParser<'a> {
    ast: &'a AST,
    semantics: &'a Semantics,
    program: Program,
    curr_id: VRegisterID,
    identifier_lookup: VRegisterIdentifierLookup,
    type_lookup: VRegisterTypeLookup,
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
                    output.push_str(&format!("define dso_local {return_type} @{name}("));

                    for (i, param) in params.iter().enumerate() {
                        output.push_str(&format!("{} %{}", param.ir_type, i + 1));

                        if i != params.len() - 1 {
                            output.push_str(", ");
                        }
                    }

                    output.push_str("){\n");
                    output.push_str("entry:\n");

                    instructions
                        .iter()
                        .for_each(|element| output.push_str(&element.as_llvm_ir()));
                    output.push_str("}\n\n");
                }
            }
        }

        output
    }

    fn emit_assignment_expr(
        &mut self,
        left: &ExprNode,
        right: &ExprNode,
        assignment: &AssignmentTypes,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let ExprNode::Identifier { semantic_info, .. } = left else {
            unreachable!()
        };

        let right_id = self.emit_expr(right, instructions);

        let alloc_info = self
            .identifier_lookup
            .lookup_v_register(semantic_info.symbol_id.unwrap());

        let type_lookup = self.type_lookup.lookup(right_id).clone();

        match assignment {
            AssignmentTypes::SimpleAssignment => {
                instructions.push(Instruction::Store {
                    size: type_lookup.ir_type,
                    value: RValue::VRegister(right_id),
                    store_ptr: alloc_info.allocated_id,
                });
            }

            AssignmentTypes::MultiAssignment => {
                instructions.push(Instruction::BinaryOp {
                    kind: BinaryKind::Mul,
                    size: type_lookup.ir_type,
                    left: RValue::VRegister(alloc_info.loaded_id),
                    right: RValue::VRegister(right_id),
                    v_register: self.curr_id.next(),
                });

                instructions.push(Instruction::Store {
                    size: type_lookup.ir_type,
                    value: RValue::VRegister(self.curr_id),
                    store_ptr: alloc_info.allocated_id,
                });
            }

            _ => todo!(),
        };

        instructions.push(Instruction::Load {
            size: type_lookup.ir_type,
            value_ptr: alloc_info.allocated_id,
            v_register: self.curr_id.next(),
        });

        self.identifier_lookup
            .update_v_register(semantic_info.symbol_id.unwrap(), self.curr_id);
        self.type_lookup.add_type(self.curr_id, type_lookup);

        self.curr_id
    }

    fn emit_binary_expr(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let ExprNode::Binary {
            left,
            operator,
            right,
        } = expr
        else {
            unreachable!()
        };

        if let TokenTypes::Assignment(assignment) = operator {
            return self.emit_assignment_expr(left, right, assignment, instructions);
        }

        let left_id = self.emit_expr(left, instructions);
        let right_id = self.emit_expr(right, instructions);

        let left_type = self.type_lookup.lookup(left_id);

        let binary_kind = match operator {
            TokenTypes::Operator(operator) => match operator {
                OperatorTypes::Plus => match left_type.ir_type.is_floating() {
                    true => BinaryKind::Fadd,
                    false => BinaryKind::Add,
                },
                OperatorTypes::Minus => match left_type.ir_type.is_floating() {
                    true => BinaryKind::Fsub,
                    false => BinaryKind::Sub,
                },
                OperatorTypes::Star => match left_type.ir_type.is_floating() {
                    true => BinaryKind::Fmul,
                    false => BinaryKind::Mul,
                },
                OperatorTypes::Divide => match left_type.signedness {
                    Signedness::Signed => BinaryKind::Sdiv,
                    Signedness::Unsigned => BinaryKind::Udiv,
                    Signedness::NoSign => BinaryKind::Fdiv,
                },
                OperatorTypes::BitLShift => BinaryKind::Shl,
                OperatorTypes::BitRShift => match left_type.signedness {
                    Signedness::Signed => BinaryKind::Ashr,
                    Signedness::Unsigned => BinaryKind::Lshr,
                    Signedness::NoSign => unreachable!(),
                },
                OperatorTypes::Amperstand => BinaryKind::And,
                OperatorTypes::BitXOR => BinaryKind::Xor,
                OperatorTypes::BitOr => BinaryKind::Or,

                _ => todo!(),
            },

            _ => todo!(),
        };

        let binary_type = *self.type_lookup.lookup(left_id);

        let result_instruction = Instruction::BinaryOp {
            kind: binary_kind,
            size: binary_type.ir_type,
            left: RValue::VRegister(left_id),
            right: RValue::VRegister(right_id),
            v_register: self.curr_id.next(),
        };

        instructions.push(result_instruction);

        self.type_lookup.add_type(self.curr_id, binary_type);

        self.curr_id
    }

    fn emit_literal_expr(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let mut size = IRType::I32;
        let signedness;
        let literal;

        match expr {
            ExprNode::Integer { num } => {
                if num.storage.is_long || num.storage.is_long_long {
                    size = IRType::I64;
                } else if num.storage.is_float {
                    size = IRType::Float;
                } else if num.storage.is_double {
                    size = IRType::Double;
                } else if num.storage.is_long_double {
                    size = IRType::X86Fp80;
                }

                signedness = match num.storage.is_unsigned {
                    true => Signedness::Unsigned,
                    false => Signedness::Signed,
                };

                literal = RValue::Literal(num.value as i64)
            }

            _ => todo!(),
        }

        let alloca = Instruction::Alloca {
            size,
            v_register: self.curr_id.next(),
        };

        let store = Instruction::Store {
            size,
            value: literal,
            store_ptr: self.curr_id,
        };

        let load = Instruction::Load {
            size,
            value_ptr: self.curr_id,
            v_register: self.curr_id.next(),
        };

        instructions.push(alloca);
        instructions.push(store);
        instructions.push(load);

        self.type_lookup.add_type(
            self.curr_id,
            SignedIRType {
                ir_type: size,
                signedness,
            },
        );

        self.curr_id
    }

    fn get_identifier_v_register(&mut self, expr: &ExprNode) -> VRegisterID {
        let ExprNode::Identifier {
            identifier: _,
            semantic_info,
        } = expr
        else {
            unreachable!();
        };

        self.identifier_lookup
            .lookup_v_register(semantic_info.symbol_id.unwrap())
            .loaded_id
    }

    fn determine_cast_kind(from: &SignedIRType, to: &SignedIRType) -> Option<CastKind> {
        let from_type = from.ir_type;
        let to_type = to.ir_type;

        if from_type.is_floating() && !to_type.is_floating() {
            return match to.signedness {
                Signedness::Signed => Some(CastKind::Fptosi),
                Signedness::Unsigned => Some(CastKind::Fptoui),
                _ => unreachable!(),
            };
        }

        if !from_type.is_floating() && to_type.is_floating() {
            return match from.signedness {
                Signedness::Signed => Some(CastKind::Sitofp),
                Signedness::Unsigned => Some(CastKind::Uitofp),
                _ => unreachable!(),
            };
        }

        if from_type.is_floating() && to_type.is_floating() {
            return match from_type.num_bits() {
                from_bits if from_bits < to_type.num_bits() => Some(CastKind::Fpext),
                from_bits if from_bits > to_type.num_bits() => Some(CastKind::Fptrunc),
                _ => None,
            };
        }

        match from_type.num_bits() {
            from_bits if from_bits > to_type.num_bits() => return Some(CastKind::Trunc),
            from_bits if from_bits == to_type.num_bits() => return None,
            _ => {}
        }

        match from.signedness {
            Signedness::Signed => Some(CastKind::Sext),
            Signedness::Unsigned => Some(CastKind::Zext),
            Signedness::NoSign => unreachable!(),
        }
    }

    fn emit_cast_expr(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let ExprNode::Cast { cast_type, expr } = expr else {
            unreachable!()
        };

        let expr_id = self.emit_expr(expr, instructions);
        let cast_from_type = self.type_lookup.lookup(expr_id);

        let cast_to_type = lower_type(cast_type);
        let cast_kind = Self::determine_cast_kind(cast_from_type, &cast_to_type);

        if let Some(kind) = cast_kind {
            let cast_instruction = Instruction::Cast {
                kind,
                from_size: cast_from_type.ir_type,
                to_size: cast_to_type.ir_type,
                value: RValue::VRegister(expr_id),
                v_register: self.curr_id.next(),
            };

            instructions.push(cast_instruction);
            self.type_lookup.add_type(self.curr_id, cast_to_type);
        };

        self.curr_id
    }

    fn emit_unary_opeartor(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let ExprNode::Unary { operator, expr } = expr else {
            unreachable!()
        };

        let expr_id = self.emit_expr(expr, instructions);
        let type_info = *self.type_lookup.lookup(expr_id);

        let instruction = match operator {
            // OperatorTypes::Plus => {}
            OperatorTypes::Minus => Instruction::BinaryOp {
                kind: BinaryKind::Sub,
                size: type_info.ir_type,
                left: RValue::Literal(0),
                right: RValue::VRegister(expr_id),
                v_register: self.curr_id.next(),
            },

            OperatorTypes::Inc => Instruction::BinaryOp {
                kind: BinaryKind::Add,
                size: type_info.ir_type,
                left: RValue::VRegister(expr_id),
                right: RValue::Literal(1),
                v_register: self.curr_id.next(),
            },

            OperatorTypes::Dec => Instruction::BinaryOp {
                kind: BinaryKind::Sub,
                size: type_info.ir_type,
                left: RValue::VRegister(expr_id),
                right: RValue::Literal(1),
                v_register: self.curr_id.next(),
            },

            OperatorTypes::BitNot => Instruction::BinaryOp {
                kind: BinaryKind::Xor,
                size: type_info.ir_type,
                left: RValue::VRegister(expr_id),
                right: RValue::Literal(-1),
                v_register: self.curr_id.next(),
            },

            _ => todo!(),
        };

        self.type_lookup.add_type(self.curr_id, type_info);
        instructions.push(instruction);

        self.curr_id
    }

    fn emit_function_call(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let ExprNode::PostFix { left, right } = expr else {
            unreachable!()
        };

        let ExprNode::Identifier {
            identifier,
            semantic_info,
        } = &**left
        else {
            unreachable!()
        };

        let ExprNode::FunctionCall { args, .. } = &**right else {
            unreachable!()
        };

        let func_type = match self.semantics.get_type(semantic_info.type_id.unwrap()) {
            TypeTableValue::Identifier(x) => x,
            _ => unreachable!(),
        };
        let TypeNode::Function { return_type, .. } = &**func_type else {
            unreachable!()
        };

        let mut ir_args = Vec::new();

        for arg in args {
            let v_register = self.emit_expr(arg, instructions);
            let ir_type = self.type_lookup.lookup(v_register);

            ir_args.push((ir_type.ir_type, RValue::VRegister(v_register)));
        }

        let return_type_info = lower_type(return_type);
        let instruction = Instruction::Call {
            func_name: identifier.clone(),
            args: ir_args,
            return_value: return_type_info.ir_type,
            v_register: self.curr_id.next(),
        };

        instructions.push(instruction);
        self.type_lookup.add_type(self.curr_id, return_type_info);
        self.curr_id
    }

    fn emit_postfix_expr(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let ExprNode::PostFix { left, right } = expr else {
            unreachable!()
        };

        // shamelessly copying code
        if matches!(&**right, ExprNode::FunctionCall { .. }) {
            return self.emit_function_call(expr, instructions);
        }

        todo!()
    }

    fn emit_expr(&mut self, expr: &ExprNode, instructions: &mut Vec<Instruction>) -> VRegisterID {
        match expr {
            ExprNode::Binary { .. } => self.emit_binary_expr(expr, instructions),
            ExprNode::Integer { .. } | ExprNode::Float { .. } | ExprNode::Char { .. } => {
                self.emit_literal_expr(expr, instructions)
            }
            ExprNode::Identifier { .. } => self.get_identifier_v_register(expr),
            ExprNode::Cast { .. } => self.emit_cast_expr(expr, instructions),

            ExprNode::Unary { .. } => self.emit_unary_opeartor(expr, instructions),
            ExprNode::PostFix { .. } => self.emit_postfix_expr(expr, instructions),
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

        let v_register = self.emit_expr(expr, instructions);

        instructions.push(Instruction::Return {
            size: self.type_lookup.lookup(v_register).ir_type,
            value: RValue::VRegister(v_register),
        });
    }

    fn emit_initalizer(&mut self, initalizer: &GlobalNode, instructions: &mut Vec<Instruction>) {
        let GlobalNode::Initalizer {
            var_type,
            r_value,
            semantic_info,
        } = initalizer
        else {
            unreachable!()
        };

        let alloc_register;
        let load_register;
        let type_info = lower_type(var_type);

        if let Some(expr) = r_value {
            self.emit_expr(expr, instructions);

            let expression_register = self.curr_id;
            alloc_register = self.curr_id.next();
            load_register = self.curr_id.next();

            instructions.push(Instruction::Alloca {
                size: type_info.ir_type,
                v_register: alloc_register,
            });

            instructions.push(Instruction::Store {
                size: type_info.ir_type,
                value: RValue::VRegister(expression_register),
                store_ptr: alloc_register,
            });

            instructions.push(Instruction::Load {
                size: type_info.ir_type,
                value_ptr: alloc_register,
                v_register: load_register,
            });
        } else {
            alloc_register = self.curr_id.next();
            load_register = self.curr_id.next();

            instructions.push(Instruction::Alloca {
                size: type_info.ir_type,
                v_register: alloc_register,
            });

            instructions.push(Instruction::Load {
                size: type_info.ir_type,
                value_ptr: self.curr_id,
                v_register: load_register,
            });
        }

        let new_identifier = AllocationInfo {
            loaded_id: load_register,
            allocated_id: alloc_register,
            type_info,
        };

        self.type_lookup.add_type(load_register, type_info);
        self.identifier_lookup
            .add_identifier(semantic_info.symbol_id.unwrap(), new_identifier);
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
            StatementNode::General(general_node) => match **general_node {
                GlobalNode::Initalizer { .. } => self.emit_initalizer(general_node, instructions),
                _ => todo!(),
            },
            StatementNode::Expression(expr) => {
                self.emit_expr(expr, instructions);
            }

            _ => todo!(),
        }
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

        self.curr_id = VRegisterID(0);
        let mut params = Vec::new();
        for (parameter, _) in parameters {
            self.curr_id.next();
            let type_info = lower_type(parameter);
            params.push(type_info);
            self.type_lookup.add_type(self.curr_id, type_info);
        }

        let mut body_instructions = Vec::new();

        for (id, (param, (_, semantic_info))) in params.iter().zip(parameters.iter()).enumerate() {
            let param_v_register = VRegisterID(id as u64 + 1);
            let alloc = Instruction::Alloca {
                size: param.ir_type,
                v_register: self.curr_id.next(),
            };

            let store = Instruction::Store {
                size: param.ir_type,
                value: RValue::VRegister(param_v_register),
                store_ptr: self.curr_id,
            };

            let load = Instruction::Load {
                size: param.ir_type,
                value_ptr: self.curr_id,
                v_register: self.curr_id.next(),
            };

            body_instructions.push(alloc);
            body_instructions.push(store);
            body_instructions.push(load);

            self.type_lookup.add_type(self.curr_id, param.clone());

            self.identifier_lookup.add_identifier(
                semantic_info.symbol_id.unwrap(),
                AllocationInfo {
                    loaded_id: self.curr_id,
                    allocated_id: self.curr_id.prev(),
                    type_info: param.clone(),
                },
            );
        }

        if let Some(statements) = body {
            self.emit_statement_node(statements, &mut body_instructions);
        }

        let func_def = Module::FunctionDef {
            return_type: lower_type(return_type).ir_type,
            name: name.clone(),
            instructions: body_instructions,
            params,
        };

        self.program.0.push(func_def);
    }
}

fn lower_type(type_node: &TypeNode) -> SignedIRType {
    let create = |ir_type: IRType, signedness: Signedness| SignedIRType {
        ir_type,
        signedness,
    };

    match type_node {
        TypeNode::Variable { held_value, .. } => lower_type(held_value),
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

        // TypeNode::Array { .. } => IRType::Ptr,
        _ => todo!(),
    }
}
