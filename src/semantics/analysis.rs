use std::slice::IterMut;

use crate::{
    lexer::{
        language_features::{DataTypes, OperatorTypes},
        lexer::TokenTypes,
    },
    parser::{
        expression_parser::ExprNode,
        nodes::{AST, GlobalNode, StatementNode},
        simple_type::{BaseType, Signedness, SimpleType},
        type_parser::TypeNode,
    },
    semantics::semantics::{Namespace, ScopeId, Semantics, SymbolKind, TypeTableValue},
};

pub struct SemanticAnalysis<'a> {
    pub ast: IterMut<'a, GlobalNode>,
    pub semantics: &'a mut Semantics,
}

impl<'a> SemanticAnalysis<'a> {
    pub fn analyze(&mut self) -> Result<(), String> {
        self.semantics.set_scope_id(ScopeId(0));

        while let Some(node) = self.ast.next() {
            match node {
                GlobalNode::Function { .. } => self.update_func_node(node)?,
                // GlobalNode::Initalizer { .. } => self.update_init_node(node)?,
                _ => todo!(),
            }
        }

        Ok(())
    }

    fn update_func_node(&mut self, func_node: &mut GlobalNode) -> Result<(), String> {
        self.semantics.set_scope_id(ScopeId(0));

        let GlobalNode::Function {
            signature,
            body,
            semantic_info,
        } = func_node
        else {
            unreachable!()
        };

        let TypeNode::Function { name, .. } = &**signature else {
            unreachable!()
        };

        *semantic_info = self.semantics.add_identifier(
            name,
            &TypeTableValue::Identifier(signature.clone()),
            SymbolKind::Function,
        )?;

        self.semantics.set_curr_function(signature);

        let TypeNode::Function { parameters, .. } = &mut **signature else {
            unreachable!()
        };

        if let Some(StatementNode::Block { scope_id, .. }) = body {
            self.semantics.set_scope_id(*scope_id);
            for (parameter, param_semantic_info) in parameters {
                let TypeNode::Variable { name, held_value } = parameter.clone() else {
                    unreachable!();
                };

                *param_semantic_info = self.semantics.add_identifier(
                    &name,
                    &TypeTableValue::Identifier(held_value),
                    SymbolKind::Variable,
                )?;
            }

            self.update_block_node(body.as_mut().unwrap())?;
        }

        Ok(())
    }

    // fn update_binary_expr_cast(&mut self, expr: &mut ExprNode) {
    //     todo!()
    // }

    fn get_expr_type(&self, expr: &ExprNode) -> SimpleType {
        match expr {
            ExprNode::Cast { cast_type, .. } => match &**cast_type {
                TypeNode::Normal { held_type, .. } => held_type.clone(),
                _ => todo!(),
            },
            ExprNode::Integer { num } => {
                let signedness = if num.storage.is_unsigned {
                    Signedness::Unsigned
                } else {
                    Signedness::Signed
                };

                if num.storage.is_long || num.storage.is_long_long {
                    return init_new_simple_type(BaseType::Long, signedness);
                } else if num.storage.is_float {
                    return init_new_simple_type(BaseType::Float, signedness);
                } else if num.storage.is_double {
                    return init_new_simple_type(BaseType::Double, signedness);
                } else if num.storage.is_long_double {
                    return init_new_simple_type(BaseType::LongDouble, signedness);
                } else {
                    return init_new_simple_type(BaseType::Int, signedness);
                }
            }
            // ExprNode::Identifier { identifier, semantic_info }
            ExprNode::Unary { expr, .. } => self.get_expr_type(expr),
            _ => panic!("type not found"),
        }
    }

    fn update_function_call(&mut self, expr: &mut ExprNode) -> Result<(), String> {
        let ExprNode::PostFix { left, right } = expr else {
            unreachable!()
        };

        let ExprNode::Identifier {
            identifier,
            semantic_info,
        } = &mut **left
        else {
            return Err(format!(
                "Expected function call to be attached to an identifier"
            ));
        };

        let ExprNode::FunctionCall { args, .. } = &mut **right else {
            unreachable!()
        };

        let Some(symbol) = self
            .semantics
            .check_symbol(identifier, Namespace::Identifier)
            .cloned()
        else {
            return Err(format!("Expected function call {identifier} to exist"));
        };

        semantic_info.symbol_id = Some(symbol.symbol_id);
        semantic_info.type_id = Some(symbol.type_id);

        let function_type_table_value = self.semantics.get_type(symbol.type_id);

        let function_type = match function_type_table_value {
            TypeTableValue::Identifier(type_node) => type_node,
            _ => return Err(format!("Expected identifier type for function")),
        };

        let TypeNode::Function {
            return_type,
            parameters,
            ..
        } = &**function_type
        else {
            return Err(format!(
                "Expected function call to {identifier} to be a function type"
            ));
        };

        let mut has_void_parameter = false;
        if parameters.len() == 1 {
            if let TypeNode::Normal { held_type, .. } = &parameters[0].0 {
                if held_type.base_type == Some(BaseType::Void) {
                    has_void_parameter = true;
                }
            }
        }

        if has_void_parameter {
            if args.len() != 0 {
                return Err(format!("Unexpected argument"));
            } else {
                return Ok(());
            }
        }

        if args.len() != parameters.len() {
            println!("{}, {}", args.len(), parameters.len());
            return Err(format!("Unexpected number of argument"));
        }

        for ((parameter, _), arg) in parameters.iter().zip(args.iter_mut()) {
            let parameter = match parameter {
                TypeNode::Variable { held_value, .. } => *held_value.clone(),
                _ => parameter.clone(),
            };

            *arg = ExprNode::Cast {
                cast_type: Box::new(parameter.clone()),
                expr: Box::new(arg.clone()),
            };
        }

        Ok(())
        // todo!();
    }

    fn update_postfix_node(&mut self, expr: &mut ExprNode) -> Result<(), String> {
        let ExprNode::PostFix { left, right } = expr else {
            unreachable!()
        };

        if matches!(&**right, ExprNode::FunctionCall { .. }) {
            self.update_function_call(expr)?;
        }

        Ok(())
    }

    fn update_expr_node(&mut self, expr: &mut ExprNode) -> Result<(), String> {
        match expr {
            ExprNode::Binary {
                left,
                operator,
                right,
            } => {
                self.update_expr_node(left)?;
                self.update_expr_node(right)?;

                if matches!(operator, TokenTypes::Assignment(_)) {
                    let ExprNode::Identifier { identifier, .. } = &**left else {
                        return Err(format!(
                            "Assignment operator can only be paired with a variable"
                        ));
                    };

                    let Some(symbol) = self
                        .semantics
                        .check_symbol(identifier, Namespace::Identifier)
                        .cloned()
                    else {
                        return Err(format!("Expected variable {identifier} to be defined"));
                    };

                    let var_type = match self.semantics.get_type(symbol.type_id) {
                        TypeTableValue::Identifier(x) => x,
                        _ => unreachable!(),
                    };

                    *right = Box::new(ExprNode::Cast {
                        cast_type: Box::new(*var_type.clone()),
                        expr: Box::new(*right.clone()),
                    });
                } else {
                    let cast =
                        conversion_rules(&self.get_expr_type(left), &self.get_expr_type(right));
                    left.add_cast(&cast);
                    right.add_cast(&cast);
                }
            }
            ExprNode::Identifier {
                identifier,
                semantic_info,
            } => {
                let Some(symbol) = self
                    .semantics
                    .check_symbol(identifier, Namespace::Identifier)
                else {
                    return Err(format!("Undefined identifier {identifier} in expression"));
                };

                semantic_info.symbol_id = Some(symbol.symbol_id);
            }
            ExprNode::Cast { expr, .. } => self.update_expr_node(expr)?,
            ExprNode::PostFix { .. } => self.update_postfix_node(expr)?,

            _ => {}
        };

        Ok(())
    }

    fn update_return(&mut self, potential_expr: &mut Option<ExprNode>) -> Result<(), String> {
        if let Some(expr) = potential_expr {
            self.update_expr_node(expr)?;

            let TypeNode::Function { return_type, .. } = self.semantics.get_curr_function() else {
                unreachable!()
            };

            *expr = ExprNode::Cast {
                cast_type: Box::new(*return_type.clone()),
                expr: Box::new(expr.clone()),
            };
        }

        Ok(())
    }

    fn update_block_node(&mut self, block: &mut StatementNode) -> Result<(), String> {
        let StatementNode::Block {
            statements,
            scope_id,
        } = block
        else {
            unreachable!()
        };

        self.semantics.set_scope_id(*scope_id);

        for statement in statements {
            match statement {
                StatementNode::General(node) => self.update_general_statement_node(node)?,
                StatementNode::Expression(expr) => self.update_expr_node(expr)?,
                StatementNode::Return(potential_expr) => self.update_return(potential_expr)?,
                _ => todo!(),
            }
        }

        // todo!()

        Ok(())
    }

    fn update_general_statement_node(&mut self, node: &mut GlobalNode) -> Result<(), String> {
        match node {
            GlobalNode::Initalizer { .. } => self.update_initalizer(node),
            GlobalNode::TagType { .. } => todo!(),
            GlobalNode::Function { .. } => unreachable!(),
        }
    }

    fn update_initalizer(&mut self, initalizer: &mut GlobalNode) -> Result<(), String> {
        let GlobalNode::Initalizer {
            var_type,
            r_value,
            semantic_info,
        } = initalizer
        else {
            unreachable!();
        };

        let TypeNode::Variable { name, held_value } = var_type.clone() else {
            unreachable!();
        };

        let TypeNode::Normal { held_type, .. } = &*held_value.clone() else {
            todo!()
        };

        *semantic_info = self.semantics.add_identifier(
            &name,
            &TypeTableValue::Identifier(held_value),
            SymbolKind::Variable,
        )?;

        if let Some(r_value) = r_value {
            self.update_expr_node(r_value)?;
            r_value.add_cast(held_type);
        }

        Ok(())
        // todo!()
    }
}

fn init_new_simple_type(base_type: BaseType, signedness: Signedness) -> SimpleType {
    SimpleType {
        base_type: Some(base_type),
        storage: None,
        signedness,
        qualifiers: Vec::new(),
        is_inline_function: false,
        is_typedef: false,
    }
}

fn conversion_rules(first_type: &SimpleType, second_type: &SimpleType) -> SimpleType {
    let get_type_info = |simple_type: &SimpleType| {
        let num_bits = simple_type.base_type.unwrap().num_bits();
        let is_floating = simple_type.base_type.unwrap().is_floating();
        let is_unsigned = matches!(simple_type.signedness, Signedness::Unsigned);
        return (num_bits, is_floating, is_unsigned);
    };
    let (first_bits, first_float, first_unsigned) = get_type_info(first_type);
    let (second_bits, second_float, second_unsigned) = get_type_info(second_type);

    // prioritize the type that is a floating point
    if first_float && !second_float {
        return init_new_simple_type(first_type.base_type.unwrap(), Signedness::NoSign);
    }
    if !first_float && second_float {
        return init_new_simple_type(second_type.base_type.unwrap(), Signedness::NoSign);
    }

    // automatically upcast to an int
    if first_bits < 32 && second_bits < 32 {
        return init_new_simple_type(BaseType::Int, Signedness::Signed);
    }

    // prioritize the larger type
    if first_bits > second_bits {
        return init_new_simple_type(first_type.base_type.unwrap(), first_type.signedness);
    }
    if first_bits < second_bits {
        return init_new_simple_type(second_type.base_type.unwrap(), second_type.signedness);
    }

    // prioritize the unsigned type
    if first_unsigned && !second_unsigned {
        return init_new_simple_type(first_type.base_type.unwrap(), first_type.signedness);
    }

    init_new_simple_type(second_type.base_type.unwrap(), second_type.signedness)
}
