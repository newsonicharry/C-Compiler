use crate::lexer::language_features::{DataTypes, KeywordTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::{ExprNode, parse_expression};
use crate::parser::helper::verify_next_in_comma_list;
use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct SimpleType {
    pub base_type: DataTypes,
    pub modifiers: Vec<DataTypes>,
    pub qualifiers: Vec<DataTypes>,
}

impl Display for SimpleType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        let mut push_data_types = |data_type_list: &Vec<DataTypes>| {
            for data_type in data_type_list {
                output.push_str(&format!("{data_type} "));
            }
        };
        push_data_types(&self.modifiers);
        push_data_types(&self.qualifiers);

        output.push_str(&self.base_type.to_string());

        write!(display, "{output}")
    }
}

#[derive(Clone, Debug)]
pub enum TypeNode {
    Empty,

    Variable {
        name: String,
        held_value: Box<TypeNode>,
    },

    Normal {
        held_type: SimpleType,
        held_value: Box<TypeNode>,
    },

    Pointer {
        qualifiers: Vec<DataTypes>,
        function_parameters: Vec<TypeNode>,
        held_value: Box<TypeNode>,
    },

    Array {
        expr: ExprNode,
        held_value: Box<TypeNode>,
    },

    Struct {
        name: Option<String>, // if its declared from an anonymous struct
        qualifiers: Vec<DataTypes>,
    },

    Enum {
        name: Option<String>,
        qualifiers: Vec<DataTypes>,
    },
}

impl TypeNode {
    pub fn contains_struct_type(&self) -> bool {
        match self {
            TypeNode::Variable { held_value, .. }
            | TypeNode::Normal { held_value, .. }
            | TypeNode::Pointer { held_value, .. }
            | TypeNode::Array { held_value, .. } => held_value.contains_struct_type(),

            TypeNode::Struct { .. } => true,

            TypeNode::Empty | TypeNode::Enum { .. } => false,
        }
    }

    // gotta love the rust borrow checker making me write this
    pub fn get_most_nested_layer(&mut self) -> &mut TypeNode {
        let should_recurse = match self {
            TypeNode::Variable { held_value, .. }
            | TypeNode::Normal { held_value, .. }
            | TypeNode::Pointer { held_value, .. }
            | TypeNode::Array { held_value, .. } => !matches!(held_value.as_ref(), TypeNode::Empty),
            _ => false,
        };

        if !should_recurse {
            return self;
        }

        match self {
            TypeNode::Variable { held_value, .. }
            | TypeNode::Normal { held_value, .. }
            | TypeNode::Pointer { held_value, .. }
            | TypeNode::Array { held_value, .. } => held_value.get_most_nested_layer(),
            _ => unreachable!(),
        }
    }

    pub fn set_most_nested_held_value(&mut self, nested_value: &TypeNode) {
        match self {
            Self::Variable { held_value, .. }
            | Self::Normal { held_value, .. }
            | Self::Pointer { held_value, .. }
            | Self::Array { held_value, .. } => {
                held_value.set_most_nested_held_value(nested_value);
            }

            Self::Empty => *self = nested_value.clone(),

            Self::Struct { .. } | Self::Enum { .. } => {
                panic!("Tag type type is a unique type with no internal values")
            }
        }
    }

    fn write_pointer_data_for_display(
        output: &mut String,
        qualifiers: &Vec<DataTypes>,
        held_value: &Box<TypeNode>,
        function_parameters: &Vec<TypeNode>,
    ) {
        match function_parameters.is_empty() {
            true => {
                output.push_str("(Ptr ");
            }
            false => {
                output.push_str("(FuncPtr ");
            }
        };

        for qualifier in qualifiers {
            output.push_str(&format!("{qualifier} "));
        }

        if !function_parameters.is_empty() {
            output.push_str("(Params ");
            for parameter in function_parameters {
                output.push_str(&Self::display(parameter));
            }
            output.push_str(") ");
        }

        output.push_str(&format!("{})", Self::display(held_value)));
    }

    fn display_tag_type(output: &mut String, name: &Option<String>, qualifiers: &Vec<DataTypes>) {
        if let Some(name) = name {
            output.push_str(&format!(" {name}"));
        }

        if !qualifiers.is_empty() {
            output.push_str(" (Qualifiers");
            for qualifier in qualifiers {
                output.push_str(&format!(" {qualifier}"));
            }

            output.push_str(")");
        }

        output.push_str(")");
    }

    fn display(node: &TypeNode) -> String {
        let mut output = String::new();

        match node {
            TypeNode::Variable { name, held_value } => {
                if !name.is_empty() {
                    output.push_str(&format!("(Name {} {})", name, Self::display(held_value)));
                }
            }

            TypeNode::Array { expr, held_value } => {
                output.push_str(&format!(
                    "(Arr {} {})",
                    expr.to_string()
                        .chars()
                        .filter(|x| *x != '\n')
                        .collect::<String>(),
                    Self::display(&held_value)
                ));
            }

            TypeNode::Pointer {
                qualifiers,
                held_value,
                function_parameters,
            } => {
                Self::write_pointer_data_for_display(
                    &mut output,
                    qualifiers,
                    held_value,
                    function_parameters,
                );
            }

            TypeNode::Normal {
                held_value,
                held_type,
            } => {
                output.push_str(&format!(
                    "(Type {} {}",
                    held_type,
                    Self::display(held_value)
                ));

                if output.ends_with(' ') {
                    output.pop();
                }

                output.push_str(")");
            }

            TypeNode::Struct { name, qualifiers } => {
                output.push_str(&format!("(Struct"));
                Self::display_tag_type(&mut output, name, qualifiers);
            }

            TypeNode::Enum { name, qualifiers } => {
                output.push_str(&format!("(Enum"));
                Self::display_tag_type(&mut output, name, qualifiers);
            }

            TypeNode::Empty => {}
        }

        output
    }
}

impl Display for TypeNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = Self::display(self);

        write!(display, "{final_str}")
    }
}

pub fn parse_type(lexer: &mut Lexer) -> Result<TypeNode, String> {
    // temporary of the inner most type
    let mut final_type = TypeNode::Empty;

    // we hold the modifier, qualifier and type of the left most part of the type
    // for most types this is all there will be but for anything more complex (such as a pointer or anytype of nesting)
    // we will use this as just the outer most part of the type
    let mut original_type = None;

    // if it is a pointer we hold its qualifiers
    let mut all_pointer_data: Vec<Vec<DataTypes>> = Vec::new();
    let mut pointer_function_parameters = Vec::new();

    // if theres an array element(s) we hold that as well
    let mut array_expressions = Vec::new();

    while let Some(token) = lexer.peek() {
        match token {
            TokenTypes::Identifier(identifier) => {
                // this is true if its a function's return type
                // if thats the case then we dont want to pick up its name and we break
                if let Some(TokenTypes::Operator(OperatorTypes::LParen)) = lexer.forward_peek() {
                    break;
                }

                if !is_valid_var_name(&identifier) {
                    return Err(String::from("Variable does not have a valid variable name"));
                }

                final_type = TypeNode::Variable {
                    name: identifier,
                    held_value: Box::new(TypeNode::Empty),
                };
                lexer.advance();
            }

            TokenTypes::DataType(_) | TokenTypes::Keyword(KeywordTypes::Struct) => {
                original_type = Some(parse_normal_type(lexer)?);
            }

            TokenTypes::Operator(OperatorTypes::Star) => {
                all_pointer_data.push(parse_pointer_qualifiers(lexer));
            }
            TokenTypes::Operator(OperatorTypes::LParen) => {
                lexer.advance();

                if matches!(lexer.peek(), Some(TokenTypes::DataType(_))) {
                    pointer_function_parameters = parse_parameter_list(lexer)?;
                } else {
                    final_type = parse_type(lexer)?;
                    lexer.advance();
                };
            }
            TokenTypes::Operator(OperatorTypes::LSquareBracket) => {
                lexer.advance();
                array_expressions.push(parse_expression(lexer, 0)?);
                lexer.advance();
            }

            _ => break,
        }
    }
    // this is because multidimensional arrays exist and we start at the outside in
    for array_expr in array_expressions.iter().rev() {
        let internal_type = TypeNode::Array {
            expr: array_expr.clone(),
            held_value: Box::new(TypeNode::Empty),
        };

        final_type.set_most_nested_held_value(&internal_type);
    }

    if !pointer_function_parameters.is_empty() {
        let most_nested_type = final_type.get_most_nested_layer();

        let TypeNode::Pointer { qualifiers, .. } = most_nested_type.clone() else {
            return Err(String::from(
                "Expected type to be pointer due to function parameters",
            ));
        };

        *most_nested_type = TypeNode::Pointer {
            qualifiers,
            function_parameters: pointer_function_parameters.clone(),
            held_value: Box::new(TypeNode::Empty),
        };
    }

    for pointer_qualifiers in all_pointer_data.iter().rev() {
        let internal_type = TypeNode::Pointer {
            qualifiers: pointer_qualifiers.clone(),
            function_parameters: Vec::new(),
            held_value: Box::new(TypeNode::Empty),
        };

        final_type.set_most_nested_held_value(&internal_type);
    }

    if let Some(normal_type) = original_type {
        final_type.set_most_nested_held_value(&normal_type);
    }

    Ok(final_type)
}

pub fn parse_parameter_list(lexer: &mut Lexer) -> Result<Vec<TypeNode>, String> {
    let mut param_list = Vec::new();

    while !matches!(
        lexer.peek(),
        Some(TokenTypes::Operator(OperatorTypes::RParen))
    ) {
        if matches!(
            lexer.peek(),
            Some(TokenTypes::Operator(OperatorTypes::Comma))
        ) {
            return Err(String::from("Unexpected comma in parameter list"));
        }

        param_list.push(parse_type(lexer)?);

        verify_next_in_comma_list(
            lexer,
            TokenTypes::Operator(OperatorTypes::RParen),
            "Unexpected end to parameter list",
        )?;

        if matches!(
            lexer.peek(),
            Some(TokenTypes::Operator(OperatorTypes::Comma))
        ) {
            lexer.advance();
        }
    }
    lexer.advance();
    Ok(param_list)
}

fn parse_pointer_qualifiers(lexer: &mut Lexer) -> Vec<DataTypes> {
    lexer.advance(); // move past the *
    let mut qualifiers: Vec<DataTypes> = Vec::new();

    while let Some(TokenTypes::DataType(data_type)) = lexer.peek() {
        if data_type.is_qualifier() {
            qualifiers.push(data_type);
        }
        lexer.advance();
    }

    qualifiers
}

fn parse_normal_type(lexer: &mut Lexer) -> Result<TypeNode, String> {
    let mut base_type = DataTypes::NoType;
    let mut modifiers: Vec<DataTypes> = Vec::new();
    let mut qualifiers: Vec<DataTypes> = Vec::new();

    while let Some(token) = lexer.peek()
        && (matches!(token, TokenTypes::DataType(_)))
    {
        let TokenTypes::DataType(data_type) = token else {
            unreachable!()
        };

        if data_type.is_modifier() {
            modifiers.push(data_type);
        } else if data_type.is_qualifier() {
            qualifiers.push(data_type);
        } else {
            base_type = data_type;
        }

        lexer.advance();
    }

    if matches!(
        lexer.peek(),
        Some(TokenTypes::Keyword(KeywordTypes::Struct))
    ) {
        lexer.advance();

        let struct_name = lexer.expect_extract(|x| match x {
            TokenTypes::Identifier(struct_name) => Some(struct_name),
            _ => None,
        })?;

        return Ok(TypeNode::Struct {
            name: Some(struct_name),
            qualifiers,
        });
    }

    let is_long_or_short = modifiers
        .iter()
        .any(|x| *x == DataTypes::Long || *x == DataTypes::Short);

    // a long or a short modifier is still a long or a short without an implicit int base type
    // (e.g., short x; is just as valid as short int x;)
    if is_long_or_short && base_type == DataTypes::NoType {
        base_type = DataTypes::Int;
    }
    // occurs when there is no base type (e.g., const x; is not a valid type)
    else if !is_long_or_short && base_type == DataTypes::NoType {
        return Err(String::from(
            "Not given a valid base type, only a modifer or qualifier.",
        ));
    }

    let final_type = SimpleType {
        base_type,
        modifiers,
        qualifiers,
    };

    Ok(TypeNode::Normal {
        held_type: final_type,
        held_value: Box::new(TypeNode::Empty),
    })
}

pub fn is_valid_var_name(name: &str) -> bool {
    // we don't have to worry about whitespace as the lexer can't parse that
    if name.is_empty() {
        return false;
    }

    if name.chars().nth(0).unwrap().is_ascii_digit() {
        return false;
    }

    name.chars().all(|x| x.is_ascii_alphanumeric() || x == '_')
}

#[cfg(test)]
mod tests {
    use crate::parser::helper::run_tests;

    use super::*;

    #[test]
    fn test_modifier_qualifier() {
        let test_cases = vec![
            (
                "unsigned long long int a;",
                "(Name a (Type unsigned long long int))",
            ),
            (
                "const volatile double b;",
                "(Name b (Type const volatile double))",
            ),
            (
                "float * restrict c;",
                "(Name c (Ptr restrict (Type float)))",
            ),
            ("short d;", "(Name d (Type short int))"),
        ];

        run_tests(parse_type, test_cases);
    }

    #[test]
    fn test_pointer_and_array() {
        let test_cases = vec![
            ("int *ptr;", "(Name ptr (Ptr (Type int)))"),
            ("const int *c_ptr;", "(Name c_ptr (Ptr (Type const int)))"),
            ("int** p", "(Name p (Ptr (Ptr (Type int))))"),
            ("int** const p", "(Name p (Ptr const (Ptr (Type int))))"),
            ("int* const * p", "(Name p (Ptr (Ptr const (Type int))))"),
            ("int * const ptr_c;", "(Name ptr_c (Ptr const (Type int)))"),
            (
                "char matrix[10][20];",
                "(Name matrix (Arr (Num 20) (Arr (Num 10) (Type char))))",
            ),
        ];

        run_tests(parse_type, test_cases);
    }

    #[test]
    fn test_function_pointers() {
        let test_cases = vec![
            (
                "void (*callback)(void);",
                "(Name callback (FuncPtr (Params (Type void)) (Type void)))",
            ),
            (
                "int (*math_op)(int, int);",
                "(Name math_op (FuncPtr (Params (Type int)(Type int)) (Type int)))",
            ),
            (
                "const void * (*lookup_table[5])(const char *);",
                "(Name lookup_table (Arr (Num 5) (FuncPtr (Params (Ptr (Type const char))) (Ptr (Type const void)))))",
            ),
        ];

        run_tests(parse_type, test_cases);
    }

    #[test]
    fn test_combination() {
        let test_cases = vec![
            (
                "const int *(* volatile multi_layer_ptr)[10];",
                "(Name multi_layer_ptr (Ptr volatile (Arr (Num 10) (Ptr (Type const int)))))",
            ),
            (
                "unsigned int (*(*func_ptr_array[5])(void))[10];",
                "(Name func_ptr_array (Arr (Num 5) (FuncPtr (Params (Type void)) (Ptr (Arr (Num 10) (Type unsigned int))))))",
            ),
            (
                "const char * const (*(*complex_func)(int, void (*)(int)))(double);",
                "(Name complex_func (FuncPtr (Params (Type int)(FuncPtr (Params (Type int)) (Type void))) (FuncPtr (Params (Type double)) (Ptr const (Type const char)))))",
            ),
        ];

        run_tests(parse_type, test_cases);
    }

    #[test]
    fn test_struct_types() {
        let test_cases = vec![
            ("struct Point *p;", "(Name p (Ptr (Struct Point)))"),
            ("struct Point **p;", "(Name p (Ptr (Ptr (Struct Point))))"),
            (
                "struct Point p[10];",
                "(Name p (Arr (Num 10) (Struct Point)))",
            ),
            (
                "struct Point p[n];",
                "(Name p (Arr (Var n) (Struct Point)))",
            ),
            (
                "struct Point (*p)(void);",
                "(Name p (FuncPtr (Params (Type void)) (Struct Point)))",
            ),
            (
                "struct Point (*p)(int, int);",
                "(Name p (FuncPtr (Params (Type int)(Type int)) (Struct Point)))",
            ),
            (
                "const struct Point p;",
                "(Name p (Struct Point (Qualifiers const)))",
            ),
            (
                "volatile struct Point p;",
                "(Name p (Struct Point (Qualifiers volatile)))",
            ),
        ];

        run_tests(parse_type, test_cases);
    }
}
