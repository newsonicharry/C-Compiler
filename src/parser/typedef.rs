use crate::parser::nodes::GlobalNode;
use crate::parser::parser::Parser;
use crate::parser::type_parser::TypeNode;
use crate::semantics::semantics::SymbolKind;
use crate::semantics::semantics::TypeTableValue;

impl Parser {
    /// returns a boolean on whether a given global node is a typedef
    /// though typedefs can be invalid according to the grammar and so is wrapped within a result
    /// if it is a typedef the typedef is added to the semantics
    pub fn is_typedef_analysis(&mut self, node: &GlobalNode) -> Result<bool, String> {
        match node {
            GlobalNode::Initalizer { .. } => self.initalizer_typedef_analysis(node),
            GlobalNode::Function { .. } => self.function_typedef_analysis(node),
            _ => todo!(),
        }
    }

    fn function_typedef_analysis(&mut self, node: &GlobalNode) -> Result<bool, String> {
        let GlobalNode::Function {
            mut signature,
            body,
            ..
        } = node.clone()
        else {
            unreachable!()
        };

        if !signature.is_typedef() {
            return Ok(false);
        }

        if body.is_some() {
            return Err(String::from(
                "Typedefed function is not allowed to have a body",
            ));
        }

        signature.remove_typedef_property();

        todo!();
        // self.semantics
        //     .add_identifier(&TypeTableValue::Identifier(signature), SymbolKind::Typedef);

        Ok(true)
    }

    fn initalizer_typedef_analysis(&mut self, node: &GlobalNode) -> Result<bool, String> {
        let GlobalNode::Initalizer {
            var_type, r_value, ..
        } = node
        else {
            unreachable!()
        };
        let is_typedef = var_type.is_typedef();
        if !is_typedef {
            return Ok(false);
        }

        if is_typedef && r_value.is_some() {
            return Err(String::from(
                "Typedef is not expected to have an initalizer",
            ));
        }

        let TypeNode::Variable {
            name,
            mut held_value,
        } = var_type.clone()
        else {
            unreachable!()
        };

        // the value the semantics uses is only the raw type, without the original typedef property
        // or the orginial variable name
        held_value.remove_typedef_property();
        self.semantics.add_identifier(
            &name,
            &TypeTableValue::Identifier(held_value),
            SymbolKind::Typedef,
        );

        // let semantics_type = IdentifierType::Typedef(*held_value.clone());

        // // if theres already a typedef in the same scope with the same name and same value its not an error
        // // otherwise is a redefinition
        // if let Err(_) = self.semantics.add_identifier_symbol(&name, &semantics_type) {
        //     let Some(found_type) = self.semantics.check_identifier(&name) else {
        //         unreachable!()
        //     };

        //     if found_type != semantics_type {
        //         return Err(String::from("Unexpected reduplication of typedef"));
        //     }
        // }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::helper::run_tests;

    use super::*;

    #[test]
    fn typedef_simple() {
        let code = "
                typedef int TInt;
                TInt v1 = 1;

                typedef unsigned UInt;
                UInt v2 = 2u;

                typedef unsigned long ULong;
                ULong v3 = 4UL;

                typedef unsigned char UChar;
                UChar v4 = 255;

               typedef long double LDouble;
                LDouble v5 = 3.5L;
            ";

        let result = "
            (Variable (Name v1 (Type int)) (Num 1))
            (Variable (Name v2 (Type unsigned int)) (Num 2u))
            (Variable (Name v3 (Type unsigned long int)) (Num 4ul))
            (Variable (Name v4 (Type unsigned char)) (Num 255))
            (Variable (Name v5 (Type long double)) (Num 3.5l))
        ";

        run_tests(Parser::parse_program, vec![(code, result)]);
    }

    #[test]
    fn typedef_qualifiers() {
        let code = "
                typedef const int ConstInt;
                ConstInt v1 = 1;

                typedef volatile int VolatileInt;
                VolatileInt v2 = 2;

                typedef const volatile int CVInt;
                CVInt v3 = 3;
        ";

        let result = "
            (Variable (Name v1 (Type const int)) (Num 1))
            (Variable (Name v2 (Type volatile int)) (Num 2))
            (Variable (Name v3 (Type const volatile int)) (Num 3))
        ";

        run_tests(Parser::parse_program, vec![(code, result)]);
    }

    #[test]
    fn typedef_pointer() {
        let code = "
                typedef int *IntPtr;
                IntPtr v1 = 0;

                typedef const int *ConstIntPtr;
                ConstIntPtr v2 = 0;

                typedef int **IntPtrPtr;
                IntPtrPtr v3 = 0;

                typedef void *VoidPtr;
                VoidPtr v4 = 0;
        ";

        let result = "
                (Variable (Name v1 (Ptr (Type int))) (Num 0))
                (Variable (Name v2 (Ptr (Type const int))) (Num 0))
                (Variable (Name v3 (Ptr (Ptr (Type int)))) (Num 0))
                (Variable (Name v4 (Ptr (Type void))) (Num 0))
        ";

        run_tests(Parser::parse_program, vec![(code, result)]);
    }

    #[test]
    fn typedef_array() {
        let code = "
                typedef int IntArray5[5];
                IntArray5 v1 = {};

                typedef char CharArray6[6];
                CharArray6 v2 = \"hello\";
        ";

        let result = "
                (Variable (Name v1 (Arr (Num 5) (Type int))) (AggInit))
                (Variable (Name v2 (Arr (Num 6) (Type char))) (Str hello))
        ";

        run_tests(Parser::parse_program, vec![(code, result)]);
    }

    #[test]
    fn typedef_struct() {
        let code = "
                
        ";

        let result = "

        ";

        run_tests(Parser::parse_program, vec![(code, result)]);
    }
}
