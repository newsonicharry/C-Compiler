use crate::{
    ir::{
        instructions::{
            AllocationInfo, BinaryKind, CastKind, IRType, Instruction, RValue, SignedIRType,
            VRegisterID,
        },
        ir::IRParser,
    },
    lexer::{
        language_features::{AssignmentTypes, OperatorTypes},
        lexer::TokenTypes,
    },
    parser::{
        expression_parser::ExprNode,
        nodes::{
            GlobalNode::{self},
            StatementNode,
        },
        simple_type::Signedness,
        type_parser::TypeNode,
    },
    semantics::semantics::TypeTableValue,
};

impl<'a> IRParser<'a> {
    fn emit_assignment_expr(
        &mut self,
        left: &ExprNode,
        right: &ExprNode,
        assignment: &AssignmentTypes,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let right_id = self.emit_expr(right, instructions);
        let type_lookup = self.type_lookup.lookup_type(right_id).clone();

        let store_ptr = match left {
            ExprNode::Identifier { semantic_info, .. } => {
                self.identifier_lookup
                    .lookup_v_register(semantic_info.symbol_id.unwrap())
                    .allocated_id
            }
            ExprNode::Unary {
                operator: OperatorTypes::Star,
                expr: ptr_expr,
                ..
            } => self.emit_expr(ptr_expr, instructions),

            _ => unreachable!(),
        };

        match assignment {
            AssignmentTypes::SimpleAssignment => {
                instructions.push(Instruction::Store {
                    size: type_lookup.ir_type,
                    value: RValue::VRegister(right_id),
                    store_ptr,
                });
            }

            AssignmentTypes::MultiAssignment => {
                instructions.push(Instruction::Load {
                    size: type_lookup.ir_type.clone(),
                    value_ptr: store_ptr,
                    v_register: self.curr_id.next(),
                });

                instructions.push(Instruction::BinaryOp {
                    kind: BinaryKind::Mul,
                    size: type_lookup.ir_type.clone(),
                    left: RValue::VRegister(self.curr_id),
                    right: RValue::VRegister(right_id),
                    v_register: self.curr_id.next(),
                });

                instructions.push(Instruction::Store {
                    size: type_lookup.ir_type,
                    value: RValue::VRegister(self.curr_id),
                    store_ptr,
                });
            }
            _ => todo!(),
        };

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

        let left_type = self.type_lookup.lookup_type(left_id);

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

        let binary_type = self.type_lookup.lookup_type(left_id).clone();

        let result_instruction = Instruction::BinaryOp {
            kind: binary_kind,
            size: binary_type.ir_type.clone(),
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
            size: size.clone(),
            v_register: self.curr_id.next(),
        };

        let store = Instruction::Store {
            size: size.clone(),
            value: literal,
            store_ptr: self.curr_id,
        };

        let load = Instruction::Load {
            size: size.clone(),
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

    fn get_identifier_v_register(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let ExprNode::Identifier { semantic_info, .. } = expr else {
            unreachable!();
        };

        let allocated = self
            .identifier_lookup
            .lookup_v_register(semantic_info.symbol_id.unwrap())
            .allocated_id;

        let type_table_value = self.semantics.get_type(semantic_info.type_id.unwrap());

        let type_lookup = match type_table_value {
            TypeTableValue::Identifier(x) => self.lower_type(x, instructions),
            _ => todo!(),
        };

        instructions.push(Instruction::Load {
            size: type_lookup.ir_type.clone(),
            value_ptr: allocated,
            v_register: self.curr_id.next(),
        });

        self.type_lookup.add_type(self.curr_id, type_lookup);

        return self.curr_id;
    }

    fn determine_cast_kind(from: &SignedIRType, to: &SignedIRType) -> Option<CastKind> {
        let from_type = from.ir_type.clone();
        let to_type = to.ir_type.clone();

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
        let cast_from_type = self.type_lookup.lookup_type(expr_id).clone();

        let cast_to_type = self.lower_type(cast_type, instructions);

        if matches!(cast_to_type.ir_type, IRType::Ptr)
            || matches!(cast_from_type.ir_type, IRType::Ptr)
        {
            return expr_id;
        }

        let cast_kind = Self::determine_cast_kind(&cast_from_type, &cast_to_type);

        if let Some(kind) = cast_kind {
            let cast_instruction = Instruction::Cast {
                kind,
                from_size: cast_from_type.ir_type.clone(),
                to_size: cast_to_type.ir_type.clone(),
                value: RValue::VRegister(expr_id),
                v_register: self.curr_id.next(),
            };

            instructions.push(cast_instruction);
            self.type_lookup
                .add_type(self.curr_id, cast_to_type.clone());
            return self.curr_id;
        };

        expr_id
    }

    fn emit_unary_opeartor(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let ExprNode::Unary {
            operator,
            expr,
            converted_to_type,
        } = expr
        else {
            unreachable!()
        };

        let expr_id = self.emit_expr(expr, instructions);
        let type_info = self.type_lookup.lookup_type(expr_id).clone();

        let potential_var_address = match &**expr {
            ExprNode::Identifier { semantic_info, .. } => Some(
                self.identifier_lookup
                    .lookup_v_register(semantic_info.symbol_id.unwrap())
                    .allocated_id,
            ),
            _ => None,
        };

        let mut added_instructions = Vec::new();

        match operator {
            // as far as im aware a unary plus literally doesnt have a purpose
            OperatorTypes::Plus => {}
            OperatorTypes::Amperstand => {
                let ExprNode::Identifier { semantic_info, .. } = &**expr else {
                    unreachable!()
                };

                return self
                    .identifier_lookup
                    .lookup_v_register(semantic_info.symbol_id.unwrap())
                    .allocated_id;
            }

            OperatorTypes::Star => {
                let Some(converted_to_type) = converted_to_type else {
                    unreachable!();
                };
                let new_type = self.lower_type(converted_to_type, instructions);

                let load = Instruction::Load {
                    size: new_type.ir_type.clone(),
                    value_ptr: expr_id,
                    v_register: self.curr_id.next(),
                };

                added_instructions.push(load);
                self.type_lookup.add_type(self.curr_id, new_type);
            }

            OperatorTypes::Minus => {
                added_instructions.push(Instruction::BinaryOp {
                    kind: BinaryKind::Sub,
                    size: type_info.ir_type.clone(),
                    left: RValue::Literal(0),
                    right: RValue::VRegister(expr_id),
                    v_register: self.curr_id.next(),
                });

                self.type_lookup.add_type(self.curr_id, type_info);
            }

            OperatorTypes::Inc | OperatorTypes::Dec => {
                let add = Instruction::BinaryOp {
                    kind: match operator {
                        OperatorTypes::Inc => BinaryKind::Add,
                        OperatorTypes::Dec => BinaryKind::Sub,
                        _ => unreachable!(),
                    },
                    size: type_info.ir_type.clone(),
                    left: RValue::VRegister(expr_id),
                    right: RValue::Literal(1),
                    v_register: self.curr_id.next(),
                };
                let store = Instruction::Store {
                    size: type_info.ir_type.clone(),
                    value: RValue::VRegister(self.curr_id),
                    store_ptr: potential_var_address.unwrap(),
                };

                added_instructions.push(add);
                added_instructions.push(store);

                self.type_lookup.add_type(self.curr_id, type_info);
            }

            OperatorTypes::BitNot => {
                added_instructions.push(Instruction::BinaryOp {
                    kind: BinaryKind::Xor,
                    size: type_info.ir_type.clone(),
                    left: RValue::VRegister(expr_id),
                    right: RValue::Literal(-1),
                    v_register: self.curr_id.next(),
                });

                self.type_lookup.add_type(self.curr_id, type_info);
            }
            _ => todo!(),
        };

        instructions.extend(added_instructions);

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
            let ir_type = self.type_lookup.lookup_type(v_register);

            ir_args.push((ir_type.ir_type.clone(), RValue::VRegister(v_register)));
        }

        let return_type_info = self.lower_type(return_type, instructions);

        let instruction = Instruction::Call {
            func_name: identifier.clone(),
            args: ir_args,
            return_value: return_type_info.ir_type.clone(),
            v_register: if matches!(return_type_info.ir_type, IRType::Void) {
                None
            } else {
                Some(self.curr_id.next())
            },
        };

        instructions.push(instruction);

        if !matches!(return_type_info.ir_type, IRType::Void) {
            self.type_lookup.add_type(self.curr_id, return_type_info);
        }

        self.curr_id
    }

    fn emit_postfix_expr(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        let ExprNode::PostFix { right, .. } = expr else {
            unreachable!()
        };

        // shamelessly copying code
        if matches!(&**right, ExprNode::FunctionCall { .. }) {
            return self.emit_function_call(expr, instructions);
        }

        todo!()
    }

    pub(super) fn emit_expr(
        &mut self,
        expr: &ExprNode,
        instructions: &mut Vec<Instruction>,
    ) -> VRegisterID {
        match expr {
            ExprNode::Binary { .. } => self.emit_binary_expr(expr, instructions),
            ExprNode::Integer { .. } | ExprNode::Float { .. } | ExprNode::Char { .. } => {
                self.emit_literal_expr(expr, instructions)
            }
            ExprNode::Identifier { .. } => self.get_identifier_v_register(expr, instructions),
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
            size: self.type_lookup.lookup_type(v_register).ir_type.clone(),
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
        let type_info = self.lower_type(var_type, instructions);

        if let Some(expr) = r_value {
            let expression_register = self.emit_expr(expr, instructions);

            alloc_register = self.curr_id.next();

            instructions.push(Instruction::Alloca {
                size: type_info.ir_type.clone(),
                v_register: alloc_register,
            });

            instructions.push(Instruction::Store {
                size: type_info.ir_type,
                value: RValue::VRegister(expression_register),
                store_ptr: alloc_register,
            });
        } else {
            alloc_register = self.curr_id.next();

            instructions.push(Instruction::Alloca {
                size: type_info.ir_type,
                v_register: alloc_register,
            });
        }

        let new_identifier = AllocationInfo {
            allocated_id: alloc_register,
        };

        self.identifier_lookup
            .add_identifier(semantic_info.symbol_id.unwrap(), new_identifier);
    }

    pub(super) fn emit_statement_node(
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
}
