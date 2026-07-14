use crate::parser::nodes::GlobalNode;
use crate::parser::parser::Parser;
use crate::parser::tag_types::helper::TagTypeData;
use crate::parser::tag_types::helper::TagTypeKind;
use crate::parser::type_parser::TypeNode;
use crate::semantics::semantics::Namespace;
use crate::semantics::semantics::SymbolKind;
use crate::semantics::semantics::TypeTableValue;

impl Parser {
    pub fn update_tag_type_typedef(
        &mut self,
        defined_typedefs: &Vec<GlobalNode>,
        defined_tag_type: &TagTypeData,
    ) {
        let tag_type_name_wrapped = defined_tag_type.name.as_ref();
        let tag_type_name;

        // generate a new name if the tag type is anonymous
        if let Some(name) = tag_type_name_wrapped {
            tag_type_name = name.to_owned();
        } else {
            tag_type_name = self.semantics.generate_new_name();
        }

        let symbol_kind = match defined_tag_type.kind {
            TagTypeKind::Struct => SymbolKind::Struct,
            TagTypeKind::Union => SymbolKind::Union,
            TagTypeKind::Enum => SymbolKind::Enum,
        };

        self.semantics.add_identifier(
            &tag_type_name,
            &TypeTableValue::TagType(defined_tag_type.clone()),
            symbol_kind,
        );

        let Some(tag_type_symbol) = self
            .semantics
            .check_symbol(&tag_type_name, Namespace::TagType)
            .cloned()
        else {
            unreachable!()
        };

        for typedef in defined_typedefs {
            let GlobalNode::Initalizer { var_type, .. } = typedef else {
                panic!("Expected typedef")
            };

            let TypeNode::Variable {
                name: typedef_name,
                mut held_value,
            } = var_type.clone()
            else {
                unreachable!()
            };

            held_value.remove_typedef_property();

            let TypeNode::TagType { type_id, .. } = held_value.get_most_nested_layer() else {
                unreachable!();
            };

            *type_id = Some(tag_type_symbol.type_id);

            self.semantics.add_identifier(
                &typedef_name,
                &TypeTableValue::Identifier(held_value),
                SymbolKind::Typedef,
            );
        }
    }

    /// returns a boolean on whether a given global node is a typedef
    /// though typedefs can be invalid according to the grammar and so is wrapped within a result
    /// if it is a typedef the typedef is added to the semantics
    pub fn is_typedef_analysis(&mut self, node: &GlobalNode) -> Result<bool, String> {
        match node {
            GlobalNode::Initalizer { .. } => self.initalizer_typedef_analysis(node),
            GlobalNode::Function { .. } => self.function_typedef_analysis(node),
            _ => panic!(),
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

        let TypeNode::Function { name, .. } = *signature.clone() else {
            unreachable!()
        };

        self.semantics.add_identifier(
            &name,
            &TypeTableValue::Identifier(signature),
            SymbolKind::Typedef,
        );

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

    // function typedefs will be further parsed in semantics, and thus is not tested here
    // as it is a function typedef will run and give an ast node, but like with function declarations
    // the result will be technically correct but honestly won't make a whole lot of sense

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
            typedef struct {
                int x;
            } StructAnon;

            StructAnon v1 = {1};

            typedef struct Point {
                int x;
                int y;
            } Point;

            Point v2 = {1,2};

            typedef struct Node Node;

            struct Node {
                int value;
                Node *next;
            };

            Node v3 = {10, 0};
        ";

        let result = "
            (Variable (Name v1 (Struct Anon-TagType-0))
                (AggInit (InitElement (Expr (Num 1)))))

            (Variable (Name v2 (Struct Point))
                (AggInit
                    (InitElement (Expr (Num 1)))
                    (InitElement (Expr (Num 2)))))

            (Struct Node (Members
                (Member (Name value (Type int)))
                (Member (Name next (Ptr (Struct Node))))
            ))

            (Variable (Name v3 (Struct Node))
                (AggInit
                    (InitElement (Expr (Num 10)))
                    (InitElement (Expr (Num 0)))))
        ";

        run_tests(Parser::parse_program, vec![(code, result)]);
    }

    #[test]
    fn typedef_unions() {
        let code = "
            typedef union {
                int i;
                float f;
            } Number;

            Number v1 = {.i = 42};

            typedef union TaggedUnion {
                int x;
                long y;
            } TaggedUnion;

            TaggedUnion v2 = {.y = 100};
        ";

        let result = "
            (Variable (Name v1 (Union Anon-TagType-0))
                (AggInit (Member (Var i (Expr (Num 42))))))

            (Variable (Name v2 (Union TaggedUnion))
                (AggInit (Member (Var y (Expr (Num 100))))))
        ";

        run_tests(Parser::parse_program, vec![(code, result)]);
    }

    #[test]
    fn typedef_enums() {
        let code = "
            typedef enum { RED, GREEN, BLUE } Color;
            Color v1 = GREEN;

            typedef enum Direction { NORTH,SOUTH } Direction;
            Direction v2 = SOUTH;
        ";

        let result = "
            (Variable (Name v1 (Enum Anon-TagType-0)) (Var GREEN))
            (Variable (Name v2 (Enum Direction)) (Var SOUTH))
        ";

        run_tests(Parser::parse_program, vec![(code, result)]);
    }
}
