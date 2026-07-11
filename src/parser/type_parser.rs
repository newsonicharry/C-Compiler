use crate::lexer::language_features::{DataTypes, KeywordTypes, OperatorTypes};
use crate::lexer::lexer::TokenTypes;
use crate::parser::expression_parser::ExprNode;
use crate::parser::helper::verify_next_in_comma_list;
use crate::parser::parser::Parser;
use crate::parser::tag_types::helper::TagTypeKind;
use crate::semantics::semantics::IdentifierType;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleType {
    pub base_type: DataTypes,
    pub properties: Vec<DataTypes>,
}

impl Display for SimpleType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        let mut push_data_types = |data_type_list: &Vec<DataTypes>| {
            for data_type in data_type_list {
                output.push_str(&format!("{data_type} "));
            }
        };
        push_data_types(&self.properties);

        output.push_str(&self.base_type.to_string());

        write!(display, "{output}")
    }
}

#[derive(Clone, Debug, PartialEq)]
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

    TagType {
        kind: TagTypeKind,
        name: String,
        qualifiers: Vec<DataTypes>,
    },

    Function {
        name: String,
        return_type: Box<TypeNode>,
        parameters: Vec<TypeNode>,
    },
}

impl TypeNode {
    pub fn contains_tag_type(&self) -> Option<TagTypeKind> {
        if let TypeNode::TagType { kind, .. } = self {
            return Some(kind.clone());
        }

        match self {
            TypeNode::Variable { held_value, .. }
            | TypeNode::Normal { held_value, .. }
            | TypeNode::Pointer { held_value, .. }
            | TypeNode::Array { held_value, .. } => held_value.contains_tag_type(),

            _ => None,
        }
    }

    // gotta love the rust borrow checker making me write this
    pub fn get_most_nested_layer(&mut self) -> &mut TypeNode {
        let should_recurse = match self {
            TypeNode::Variable { held_value, .. }
            | TypeNode::Normal { held_value, .. }
            | TypeNode::Pointer { held_value, .. }
            | TypeNode::Array { held_value, .. } => !matches!(held_value.as_ref(), TypeNode::Empty),

            TypeNode::Function { return_type, .. } => {
                !matches!(return_type.as_ref(), TypeNode::Empty)
            }
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

            TypeNode::Function { return_type, .. } => return_type.get_most_nested_layer(),
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

            Self::Function { return_type, .. } => {
                return_type.set_most_nested_held_value(nested_value);
            }

            Self::Empty => *self = nested_value.clone(),

            Self::TagType { .. } => {
                panic!("Tag type type is a unique type with no internal values")
            }
        }
    }

    /// refers to a member of a tag type in which its member has a specifier thats not allowed
    /// only qualifiers and base types are allowed, storage class specifiers are not
    pub fn has_invalid_tag_type_specifier(&self) -> bool {
        let mut cloned_value = self.clone();
        let nested_value = cloned_value.get_most_nested_layer();

        let Self::Normal { held_type, .. } = nested_value.get_most_nested_layer() else {
            return false;
        };

        if held_type
            .properties
            .iter()
            .any(|x| x.is_storage_specifier() || x.is_function_specifier())
        {
            return true;
        }

        false
    }

    pub fn is_typedef(&self) -> bool {
        let mut nested_value = match self {
            Self::Variable { held_value, .. } => held_value.clone(),
            Self::Function { return_type, .. } => return_type.clone(),
            _ => return false,
        };

        let Self::Normal {
            held_type: type_properties,
            ..
        } = nested_value.get_most_nested_layer()
        else {
            return false;
        };

        if type_properties.properties.contains(&DataTypes::Typedef) {
            return true;
        }

        false
    }

    pub fn remove_typedef_property(&mut self) {
        // at this point its a normal because we've already stripped out the outer part (which is either a variable or a function)
        let final_layer = self.get_most_nested_layer();

        let Self::Normal {
            held_type: type_properties,
            ..
        } = final_layer
        else {
            return;
        };

        if let Some(position) = type_properties
            .properties
            .iter()
            .position(|x| *x == DataTypes::Typedef)
        {
            type_properties.properties.remove(position);
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

    fn display_tag_type(output: &mut String, name: &str, qualifiers: &Vec<DataTypes>) {
        output.push_str(&format!(" {name}"));

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

            TypeNode::TagType {
                kind,
                name,
                qualifiers,
            } => {
                match kind {
                    TagTypeKind::Struct => output.push_str(&format!("(Struct")),
                    TagTypeKind::Union => output.push_str(&format!("(Union")),
                    TagTypeKind::Enum => output.push_str(&format!("(Enum")),
                }

                Self::display_tag_type(&mut output, name, qualifiers);
            }

            TypeNode::Function {
                name,
                return_type,
                parameters,
            } => {
                output.push_str(&format!(
                    "(Function {name} (Return {})",
                    Self::display(return_type)
                ));

                if !parameters.is_empty() {
                    output.push_str(" (Params");
                    for parameter in parameters {
                        output.push_str(&format!(" {}", parameter));
                    }
                    output.push(')');
                }

                output.push(')');
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

impl Parser {
    pub fn parse_type(&mut self) -> Result<TypeNode, String> {
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

        while let Some(token) = self.lexer.peek() {
            match token {
                TokenTypes::Identifier(identifier) => {
                    // this is true if its a function's return type
                    // if thats the case then we dont want to pick up its name and we break
                    if let Some(TokenTypes::Operator(OperatorTypes::LParen)) =
                        self.lexer.forward_peek()
                    {
                        self.lexer.advance(); // moves past the identifier
                        self.lexer.advance(); // moves past the left parenthesis

                        let params = self.parse_parameter_list()?;

                        final_type = TypeNode::Function {
                            name: identifier,
                            return_type: Box::new(TypeNode::Empty),
                            parameters: params,
                        };

                        break;
                    }

                    // if the identifier is a typedef we want to use it as if it is a normal type
                    if let Some(identifier_type) = self.semantics.check_identifier(&identifier)
                        && original_type.is_none()
                    {
                        if let IdentifierType::Typedef(type_node) = identifier_type {
                            original_type = Some(type_node);
                            self.lexer.advance();
                            continue;
                        };
                    }

                    if !Self::is_valid_var_name(&identifier) {
                        return Err(String::from("Variable does not have a valid variable name"));
                    }

                    final_type = TypeNode::Variable {
                        name: identifier,
                        held_value: Box::new(TypeNode::Empty),
                    };
                    self.lexer.advance();
                }

                TokenTypes::DataType(_)
                | TokenTypes::Keyword(KeywordTypes::Struct)
                | TokenTypes::Keyword(KeywordTypes::Enum)
                | TokenTypes::Keyword(KeywordTypes::Union) => {
                    if original_type.is_some() {
                        return Err(String::from("Unexpected data type in type parser"));
                    }
                    original_type = Some(self.parse_normal_type()?);
                }

                TokenTypes::Operator(OperatorTypes::Star) => {
                    all_pointer_data.push(self.parse_pointer_qualifiers()?);
                }
                TokenTypes::Operator(OperatorTypes::LParen) => {
                    self.lexer.advance();

                    if matches!(self.lexer.peek(), Some(TokenTypes::DataType(_))) {
                        pointer_function_parameters = self.parse_parameter_list()?;
                    } else {
                        final_type = self.parse_type()?;
                        self.lexer.advance();
                    };
                }
                TokenTypes::Operator(OperatorTypes::LSquareBracket) => {
                    self.lexer.advance();
                    array_expressions.push(self.parse_expression(0)?);
                    self.lexer.advance();
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

    pub fn parse_parameter_list(&mut self) -> Result<Vec<TypeNode>, String> {
        let mut param_list = Vec::new();

        while !matches!(
            self.lexer.peek(),
            Some(TokenTypes::Operator(OperatorTypes::RParen))
        ) {
            if matches!(
                self.lexer.peek(),
                Some(TokenTypes::Operator(OperatorTypes::Comma))
            ) {
                return Err(String::from("Unexpected comma in parameter list"));
            }

            param_list.push(self.parse_type()?);

            verify_next_in_comma_list(
                &mut self.lexer,
                TokenTypes::Operator(OperatorTypes::RParen),
                "Unexpected end to parameter list",
            )?;

            if matches!(
                self.lexer.peek(),
                Some(TokenTypes::Operator(OperatorTypes::Comma))
            ) {
                self.lexer.advance();
            }
        }
        self.lexer.advance();
        Ok(param_list)
    }

    fn parse_pointer_qualifiers(&mut self) -> Result<Vec<DataTypes>, String> {
        self.lexer.advance(); // move past the *
        let mut qualifiers: Vec<DataTypes> = Vec::new();

        while let Some(TokenTypes::DataType(data_type)) = self.lexer.peek() {
            if data_type.is_qualifier() {
                qualifiers.push(data_type);
            } else {
                return Err(format!("Unexpected non pointer qualifer {data_type}"));
            }
            self.lexer.advance();
        }

        Ok(qualifiers)
    }

    fn parse_normal_tag_type(&mut self, qualifiers: &Vec<DataTypes>) -> Result<TypeNode, String> {
        let Some(TokenTypes::Keyword(keyword_type)) = self.lexer.peek() else {
            return Ok(TypeNode::Empty);
        };

        self.lexer.advance();

        let tag_type_name = self.lexer.expect_extract(|x| match x {
            TokenTypes::Identifier(name) => Some(name),
            _ => None,
        })?;

        let tag_type_kind: TagTypeKind = (&keyword_type).into();

        Ok(TypeNode::TagType {
            kind: tag_type_kind,
            name: tag_type_name,
            qualifiers: qualifiers.clone(),
        })
    }

    fn parse_normal_type(&mut self) -> Result<TypeNode, String> {
        let mut base_type = DataTypes::NoType;
        let mut properties: Vec<DataTypes> = Vec::new();

        while let Some(TokenTypes::DataType(data_type)) = self.lexer.peek() {
            if data_type.is_modifier()
                || data_type.is_qualifier()
                || data_type.is_storage_specifier()
                || data_type.is_function_specifier()
            {
                properties.push(data_type);
            } else {
                if base_type != DataTypes::NoType {
                    return Err(format!(
                        "Unexpected data type of {data_type}, already found type {base_type}"
                    ));
                }

                base_type = data_type;
            }

            self.lexer.advance();
        }

        // check for a typedef if the current type is not a completed type
        // (aka it has no base type, such as an int or a float)
        if let Some(TokenTypes::Identifier(identifier)) = self.lexer.peek()
            && base_type == DataTypes::NoType
        {
            if let Some(IdentifierType::Typedef(typedef_type)) =
                self.semantics.check_identifier(&identifier)
            {
                let TypeNode::Normal {
                    held_type: mut typedef_simple_type,
                    held_value: typedef_held_value,
                } = typedef_type
                else {
                    return Err(String::from("Unexpected typedef type in new type"));
                };

                typedef_simple_type.properties.extend(properties.clone());

                return Ok(TypeNode::Normal {
                    held_type: typedef_simple_type,
                    held_value: typedef_held_value,
                });
            }
        }

        // error if there are multiple typedefs
        if properties
            .iter()
            .filter(|x| **x == DataTypes::Typedef)
            .count()
            > 1
        {
            return Err(String::from(
                "Expected only a single typedef, found multiple",
            ));
        }

        let tag_type = self.parse_normal_tag_type(&properties)?;
        if !matches!(tag_type, TypeNode::Empty) {
            return Ok(tag_type);
        }

        let is_modifier = properties.iter().any(|x| x.is_modifier());

        // a long or a short modifier is still a long or a short without an implicit int base type
        // (e.g., short x; is just as valid as short int x;)
        if is_modifier && base_type == DataTypes::NoType {
            base_type = DataTypes::Int;
        }
        // occurs when there is no base type (e.g., const x; is not a valid type)
        else if !is_modifier && base_type == DataTypes::NoType {
            return Err(String::from(
                "Not given a valid base type, only a modifer or qualifier.",
            ));
        }

        let final_type = SimpleType {
            base_type,
            properties,
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
}
#[cfg(test)]
mod tests {
    use crate::parser::helper::run_tests;

    use super::*;

    #[test]
    fn type_modifier_qualifier() {
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

        run_tests(Parser::parse_type, test_cases);
    }

    #[test]
    fn type_pointer_and_array() {
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

        run_tests(Parser::parse_type, test_cases);
    }

    #[test]
    fn type_function_pointers() {
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

        run_tests(Parser::parse_type, test_cases);
    }

    #[test]
    fn type_combination() {
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

        run_tests(Parser::parse_type, test_cases);
    }

    #[test]
    fn type_struct() {
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

        run_tests(Parser::parse_type, test_cases);
    }
}
