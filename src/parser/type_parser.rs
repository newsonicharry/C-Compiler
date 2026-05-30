use crate::lexer::language_features::{DataTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::{ExprNode, parse_expression};
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
pub enum DataTypeBlock {
    Empty,

    Variable {
        name: String,
        held_value: Box<DataTypeBlock>,
    },

    Normal {
        held_type: SimpleType,
        held_value: Box<DataTypeBlock>,
    },

    Pointer {
        qualifiers: Vec<DataTypes>,
        function_parameters: Vec<DataTypeBlock>,
        held_value: Box<DataTypeBlock>,
    },

    Array {
        expr: ExprNode,
        held_value: Box<DataTypeBlock>,
    },
}

impl DataTypeBlock {
    // gotta love the rust borrow checker making me write this
    pub fn get_most_nested_layer(&mut self) -> &mut DataTypeBlock {
        let should_recurse = match self {
            DataTypeBlock::Variable { held_value, .. }
            | DataTypeBlock::Normal { held_value, .. }
            | DataTypeBlock::Pointer { held_value, .. }
            | DataTypeBlock::Array { held_value, .. } => {
                !matches!(held_value.as_ref(), DataTypeBlock::Empty)
            }
            _ => false,
        };

        if !should_recurse {
            return self;
        }

        match self {
            DataTypeBlock::Variable { held_value, .. }
            | DataTypeBlock::Normal { held_value, .. }
            | DataTypeBlock::Pointer { held_value, .. }
            | DataTypeBlock::Array { held_value, .. } => held_value.get_most_nested_layer(),
            _ => unreachable!(),
        }
    }

    pub fn set_most_nested_held_value(&mut self, nested_value: &DataTypeBlock) {
        match self {
            Self::Variable { held_value, .. }
            | Self::Normal { held_value, .. }
            | Self::Pointer { held_value, .. }
            | Self::Array { held_value, .. } => {
                held_value.set_most_nested_held_value(nested_value);
            }

            Self::Empty => *self = nested_value.clone(),
        }
    }

    fn write_pointer_data_for_display(
        output: &mut String,
        qualifiers: &Vec<DataTypes>,
        held_value: &Box<DataTypeBlock>,
        function_parameters: &Vec<DataTypeBlock>,
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

    fn display(node: &DataTypeBlock) -> String {
        let mut output = String::new();

        match node {
            DataTypeBlock::Variable { name, held_value } => {
                if !name.is_empty() {
                    output.push_str(&format!("(Var {} {})", name, Self::display(held_value)));
                }
            }

            DataTypeBlock::Array { expr, held_value } => {
                output.push_str(&format!(
                    "(Arr {} {})",
                    expr.to_string()
                        .chars()
                        .filter(|x| *x != '\n')
                        .collect::<String>(),
                    Self::display(&held_value)
                ));
            }

            DataTypeBlock::Pointer {
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

            DataTypeBlock::Normal {
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

            DataTypeBlock::Empty => {}
        }

        output
    }
}

impl Display for DataTypeBlock {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = Self::display(self);

        write!(display, "{final_str}")
    }
}

pub fn parse_type(lexer: &mut Lexer) -> Result<DataTypeBlock, String> {
    // temporary of the inner most type
    let mut final_type = DataTypeBlock::Empty;

    // we hold the modifier, qualifier and type of the left most part of the type
    // for most types this is all there will be but for anything more complex (such as a pointer or anytype of nesting)
    // we will use this as just the outer most part of the type
    let mut original_type = None;

    // if it is a pointer we hold its qualifiers
    let mut found_pointer = false;
    let mut pointer_qualifiers = Vec::new();
    let mut pointer_function_parameters = Vec::new();

    // if theres an array element(s) we hold that as well
    let mut array_expressions = Vec::new();

    while let Some(token) = lexer.peek() {
        match token {
            TokenTypes::Identifier(identifier) => {
                // variable_name = Some(identifier);
                final_type = DataTypeBlock::Variable {
                    name: identifier,
                    held_value: Box::new(DataTypeBlock::Empty),
                };
                lexer.advance();
            }

            TokenTypes::DataType(_) => {
                original_type = Some(parse_normal_type(lexer)?);
            }

            TokenTypes::Operator(OperatorTypes::Star) => {
                found_pointer = true;
                pointer_qualifiers = parse_pointer_qualifiers(lexer);
            }
            TokenTypes::Operator(OperatorTypes::LParen) => {
                lexer.advance();

                if matches!(lexer.peek(), Some(TokenTypes::DataType(_))) {
                    pointer_function_parameters = parse_parameter_list(lexer)?;
                } else {
                    final_type = parse_type(lexer)?;
                };

                lexer.advance();
            }
            TokenTypes::Operator(OperatorTypes::LSquareBracket) => {
                lexer.advance();
                array_expressions.push(parse_expression(lexer, 0));
                lexer.advance();
            }

            _ => break,
        }
    }
    // this is because multidimensional arrays exist and we start at the outside in
    for array_expr in array_expressions.iter().rev() {
        let internal_type = DataTypeBlock::Array {
            expr: array_expr.clone(),
            held_value: Box::new(DataTypeBlock::Empty),
        };

        final_type.set_most_nested_held_value(&internal_type);
    }

    if !pointer_function_parameters.is_empty() {
        let most_nested_type = final_type.get_most_nested_layer();

        let DataTypeBlock::Pointer { qualifiers, .. } = most_nested_type.clone() else {
            return Err(String::from(
                "Expected type to be pointer due to function parameters",
            ));
        };

        *most_nested_type = DataTypeBlock::Pointer {
            qualifiers,
            function_parameters: pointer_function_parameters.clone(),
            held_value: Box::new(DataTypeBlock::Empty),
        };
    }

    if found_pointer {
        let internal_type = DataTypeBlock::Pointer {
            qualifiers: pointer_qualifiers.clone(),
            function_parameters: Vec::new(),
            held_value: Box::new(DataTypeBlock::Empty),
        };

        final_type.set_most_nested_held_value(&internal_type);
    }

    if let Some(normal_type) = original_type {
        final_type.set_most_nested_held_value(&normal_type);
    }

    Ok(final_type)
}

fn parse_parameter_list(lexer: &mut Lexer) -> Result<Vec<DataTypeBlock>, String> {
    let mut param_list = Vec::new();

    while !matches!(
        lexer.peek(),
        Some(TokenTypes::Operator(OperatorTypes::RParen))
    ) {
        if matches!(lexer.peek(), Some(TokenTypes::Comma)) {
            lexer.advance();
        }

        param_list.push(parse_type(lexer)?);

        if let Some(next_token) = lexer.peek()
            && !matches!(next_token, TokenTypes::Operator(OperatorTypes::RParen))
        {
            lexer.expect(|x| matches!(x, TokenTypes::Comma))?;
        }
    }
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

fn parse_normal_type(lexer: &mut Lexer) -> Result<DataTypeBlock, String> {
    let mut base_type = DataTypes::NoType;
    let mut modifiers: Vec<DataTypes> = Vec::new();
    let mut qualifiers: Vec<DataTypes> = Vec::new();

    while let Some(token) = lexer.peek()
        && matches!(token, TokenTypes::DataType(_))
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

    Ok(DataTypeBlock::Normal {
        held_type: final_type,
        held_value: Box::new(DataTypeBlock::Empty),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_tests(test_cases: Vec<(&str, &str)>) {
        for (test_case, correct_result) in test_cases {
            let mut lexer = Lexer::new(&test_case);
            let result = parse_type(&mut lexer).unwrap().to_string();
            println!("{result}");
            assert_eq!(correct_result, result);
        }
    }

    #[test]
    fn test_modifier_qualifier() {
        let test_cases = vec![
            (
                "unsigned long long int a;",
                "(Var a (Type unsigned long long int))",
            ),
            (
                "const volatile double b;",
                "(Var b (Type const volatile double))",
            ),
            ("float * restrict c;", "(Var c (Ptr restrict (Type float)))"),
            ("short d;", "(Var d (Type short int))"),
        ];

        run_tests(test_cases);
    }

    #[test]
    fn test_pointer_and_array() {
        let test_cases = vec![
            ("int *ptr;", "(Var ptr (Ptr (Type int)))"),
            ("const int *c_ptr;", "(Var c_ptr (Ptr (Type const int)))"),
            ("int * const ptr_c;", "(Var ptr_c (Ptr const (Type int)))"),
            (
                "char matrix[10][20];",
                "(Var matrix (Arr 20 (Arr 10 (Type char))))",
            ),
        ];

        run_tests(test_cases);
    }

    #[test]
    fn test_function_pointers() {
        let test_cases = vec![
            (
                "void (*callback)(void);",
                "(Var callback (FuncPtr (Params (Type void)) (Type void)))",
            ),
            (
                "int (*math_op)(int, int);",
                "(Var math_op (FuncPtr (Params (Type int)(Type int)) (Type int)))",
            ),
            (
                "const void * (*lookup_table[5])(const char *);",
                "(Var lookup_table (Arr 5 (FuncPtr (Params (Ptr (Type const char))) (Ptr (Type const void)))))",
            ),
        ];

        run_tests(test_cases);
    }

    #[test]
    fn test_combination() {
        let test_cases = vec![
            (
                "const int *(* volatile multi_layer_ptr)[10];",
                "(Var multi_layer_ptr (Ptr volatile (Arr 10 (Ptr (Type const int)))))",
            ),
            (
                "unsigned int (*(*func_ptr_array[5])(void))[10];",
                "(Var func_ptr_array (Arr 5 (FuncPtr (Params (Type void)) (Ptr (Arr 10 (Type unsigned int))))))",
            ),
            (
                "const char * const (*(*complex_func)(int, void (*)(int)))(double);",
                "(Var complex_func (FuncPtr (Params (Type int)(FuncPtr (Params (Type int)) (Type void))) (FuncPtr (Params (Type double)) (Ptr const (Type const char)))))",
            ),
        ];

        run_tests(test_cases);
    }
}
