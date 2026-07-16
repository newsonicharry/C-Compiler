use std::slice::IterMut;

use crate::{
    lexer::language_features::DataTypes,
    parser::{
        nodes::{AST, GlobalNode, StatementNode},
        type_parser::{SimpleType, TypeNode},
    },
    semantics::semantics::{ScopeId, Semantics, SymbolKind, TypeTableValue},
};

pub struct SemanticAnalysis<'a> {
    ast: IterMut<'a, GlobalNode>,
    semantics: &'a mut Semantics,
}

impl<'a> SemanticAnalysis<'a> {
    pub fn analysis(&mut self) -> Result<(), String> {
        self.semantics.set_scope_id(ScopeId(0));

        while let Some(node) = self.ast.next() {
            match node {
                GlobalNode::Function { .. } => self.update_func_node(node)?,

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

        let info = self.semantics.add_identifier(
            name,
            &TypeTableValue::Identifier(signature.clone()),
            SymbolKind::Function,
        )?;

        *semantic_info = info;

        if let Some(block) = body {
            self.update_block_node(block)?;
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
                StatementNode::General(node) => self.update_general_statement_ndoe(node)?,
                _ => todo!(),
            }
        }

        todo!()
    }

    fn update_general_statement_ndoe(&mut self, node: &mut GlobalNode) -> Result<(), String> {
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

        let info = self.semantics.add_identifier(
            &name,
            &TypeTableValue::Identifier(held_value),
            SymbolKind::Variable,
        )?;

        todo!()
    }
}

const TYPE_INFO: [(DataTypes, u16, bool); 7] = [
    (DataTypes::Int, 32, false),
    (DataTypes::_Bool, 8, false),
    (DataTypes::Char, 8, false),
    (DataTypes::Short, 16, false),
    (DataTypes::Long, 64, false),
    (DataTypes::Float, 32, true),
    (DataTypes::Double, 64, true),
];

fn get_type_info(simple_type: &SimpleType) -> (u16, bool, bool) {
    let mut bits = 0;
    let mut is_floating_point = false;
    let is_unsigned = simple_type.properties.contains(&DataTypes::Unsigned);

    for (data_type, num_bits, is_floating) in TYPE_INFO {
        if simple_type.base_type == data_type || simple_type.properties.contains(&data_type) {
            bits = num_bits;
        }

        if is_floating {
            is_floating_point = true;
        }
    }

    // accounts for a long double (which is technically 80 bits), but padded to 128
    if bits == 64 && is_floating_point {
        bits = 128;
    }

    // pretty sure this is impossible to trigger but just in case
    if bits == 0 {
        panic!()
    }

    (bits, is_floating_point, is_unsigned)
}

fn conversion_rules<'a>(first_type: &'a SimpleType, second_type: &'a SimpleType) -> &'a SimpleType {
    let (first_bits, first_float, first_unsigned) = get_type_info(first_type);
    let (second_bits, second_float, second_unsigned) = get_type_info(second_type);

    // prioritize the type that is a floating point
    if first_float && !second_float {
        return first_type;
    }
    if !first_float && second_float {
        return second_type;
    }

    // prioritize the larger type
    if first_bits > second_bits {
        return first_type;
    }
    if first_bits < second_bits {
        return second_type;
    }

    // prioritize the unsigned type
    if first_unsigned && !second_unsigned {
        return first_type;
    }

    second_type
}
