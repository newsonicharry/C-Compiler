use crate::lexer::escape_sequences::CharList;
use crate::lexer::escape_sequences::CharType;
use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::LiteralTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::lexer::number_parser::FloatType;
use crate::lexer::number_parser::IntType;
use crate::parser::aggregate_init::AggregateInit;
use crate::parser::aggregate_init::parse_aggregate_init;
use crate::parser::helper::pretty_clean_string;
use crate::parser::helper::verify_next_in_comma_list;
use crate::parser::type_parser::TypeNode;
use crate::parser::type_parser::parse_type;

use std::fmt::Display;
use std::u8;

#[derive(Clone, Debug)]
pub enum SizeOf {
    Type(TypeNode),
    Expr(ExprNode),
}

impl SizeOf {
    pub fn holds_value(&self) -> bool {
        match self {
            Self::Type(TypeNode::Empty) => false,
            Self::Expr(ExprNode::Empty) => false,
            _ => true,
        }
    }
}

impl Display for SizeOf {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_str = match self {
            Self::Type(x) => x.to_string(),
            Self::Expr(x) => x.to_string(),
        };

        write!(display, "{as_str}")
    }
}

#[derive(Clone, Debug)]
pub enum ExprNode {
    Empty,
    Binary {
        left: Box<ExprNode>,
        operator: TokenTypes,
        right: Box<ExprNode>,
    },
    Ternary {
        if_expr: Box<ExprNode>,
        then_expr: Box<ExprNode>,
        else_expr: Box<ExprNode>,
    },
    Integer {
        num: IntType,
    },
    Float {
        num: FloatType,
    },
    Char {
        character: CharType,
    },
    StrType {
        string: CharList,
    },

    Identifier {
        identifier: String,
    },
    Unary {
        operator: OperatorTypes,
        expr: Box<ExprNode>,
    },
    PostFix {
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },

    PostInc,
    PostDec,

    MemberAccess {
        member: String,
        operator: OperatorTypes,
        next_member: Box<ExprNode>,
    },

    FunctionCall {
        args: Vec<ExprNode>,
        nested_call: Box<ExprNode>,
    },

    Accessor {
        expr: Box<ExprNode>,
        nested_accessor: Box<ExprNode>,
    },

    Sizeof {
        held: Box<SizeOf>,
    },
    Cast {
        all_casts: Vec<TypeNode>,
        expr: Box<ExprNode>,
    },
    Aggregate {
        aggregate: Box<AggregateInit>,
    },
}

impl ExprNode {
    pub fn display(self, indentation: usize) -> String {
        let mut output = String::new();
        let indent_str = " ".repeat(indentation);
        let next_indent_str = " ".repeat(indentation + 2);

        match self {
            Self::Binary {
                left,
                operator,
                right,
            } => {
                let op_as_str = match operator {
                    TokenTypes::Operator(op) => op.to_string(),
                    TokenTypes::Assignment(assign) => assign.to_string(),
                    _ => unreachable!(),
                };

                output.push_str(&format!(
                    "{indent_str}(Binary\n{}\n{next_indent_str}(Op {op_as_str})\n{})",
                    left.display(indentation + 2),
                    right.display(indentation + 2)
                ));
            }

            Self::Ternary {
                if_expr,
                then_expr,
                else_expr,
            } => {
                output.push_str(&format!(
                    "{indent_str}(Ternary\n{}\n{}\n{})",
                    if_expr.display(indentation + 2),
                    then_expr.display(indentation + 2),
                    else_expr.display(indentation + 2)
                ));
            }

            Self::PostFix { left, right } => {
                output.push_str(&format!(
                    "{indent_str}(Postfix\n{}\n{})",
                    left.display(indentation + 2),
                    right.display(indentation + 2)
                ));
            }

            Self::Unary { operator, expr } => {
                output.push_str(&format!(
                    "{indent_str}(Unary\n{next_indent_str}(Op {operator})\n{})",
                    expr.display(indentation + 2)
                ));
            }

            Self::Integer { num } => {
                output.push_str(&format!("{indent_str}(Num {num})"));
            }

            Self::Float { num } => {
                output.push_str(&format!("{indent_str}(Num {num})"));
            }

            Self::Char { character } => {
                output.push_str(&format!("{indent_str}(Char {character})"));
            }

            Self::StrType { string } => {
                output.push_str(&format!("{indent_str}(Str {string})"));
            }

            Self::Identifier { identifier } => {
                output.push_str(&format!("{indent_str}(Var {identifier})"));
            }

            Self::MemberAccess {
                member,
                next_member,
                operator,
            } => {
                let name = match operator {
                    OperatorTypes::DotOperator => "MemberAccess",
                    OperatorTypes::ArrowOperator => "PointerAccess",
                    _ => unreachable!(),
                };
                output.push_str(&format!("{indent_str}({name} (Var {member})"));

                if !matches!(*next_member, ExprNode::Empty) {
                    output.push_str(&format!("\n{}", next_member.display(indentation + 2)));
                }

                output.push(')');
            }

            Self::Sizeof { held: size_of_info } => {
                output.push_str(&format!("{indent_str}(Sizeof"));

                if size_of_info.holds_value() {
                    output.push_str(&format!(
                        " {}",
                        pretty_clean_string(&size_of_info.to_string())
                    ));
                }

                output.push(')');
            }

            Self::Accessor {
                expr,
                nested_accessor,
            } => {
                output.push_str(&format!(
                    "{indent_str}(Accessor\n{}",
                    expr.display(indentation + 2),
                ));

                if !matches!(*nested_accessor, ExprNode::Empty) {
                    output.push_str(&format!("\n{}", nested_accessor.display(indentation + 2)));
                }

                output.push(')');
            }

            Self::PostInc => {
                output.push_str(&format!("{indent_str}(PostInc)\n"));
            }

            Self::PostDec => {
                output.push_str(&format!("{indent_str}(PostDec)\n"));
            }

            Self::FunctionCall { args, nested_call } => {
                output.push_str(&format!("{indent_str}(FuncCall"));

                for arg in args {
                    output.push_str(&format!(" {}", arg.display(0)));
                }

                if !matches!(*nested_call, ExprNode::Empty) {
                    output.push_str(&format!("\n{}", nested_call.display(indentation + 2)));
                }

                output.push(')');
            }

            Self::Cast { all_casts, expr } => {
                output.push_str(&format!("{indent_str}(Cast"));

                for cast in all_casts {
                    output.push_str(&format!(" {cast}"));
                }

                output.push_str(&format!("\n{})", expr.display(indentation + 2)));
            }

            Self::Aggregate { aggregate } => {
                output.push_str(&format!("{indent_str}{aggregate}"));
            }

            Self::Empty => {
                output.push_str(&format!("{indent_str}(Empty)"));
            }
        }

        output
    }
}

impl Display for ExprNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = self.clone().display(0);

        write!(display, "{final_str}")
    }
}

fn parse_cast(lexer: &mut Lexer) -> Result<ExprNode, String> {
    let mut all_casts = Vec::new();

    while let Some(TokenTypes::Operator(OperatorTypes::LParen)) = lexer.peek() {
        let Some(future_token) = lexer.forward_peek() else {
            return Err(format!("Expected next token in expression, got nothing"));
        };

        if !matches!(future_token, TokenTypes::DataType(_))
            && !matches!(future_token, TokenTypes::Keyword(KeywordTypes::Struct))
        {
            break;
        }
        lexer.advance();

        all_casts.push(parse_type(lexer)?);

        lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;
    }

    // because we cast use the rightward one first
    all_casts.reverse();

    let parsed_expr = parse_expression(lexer, u8::MAX)?;

    Ok(ExprNode::Cast {
        all_casts,
        expr: Box::new(parsed_expr),
    })
}
fn parse_size_of(lexer: &mut Lexer) -> Result<ExprNode, String> {
    lexer.advance();
    let token = lexer.force_peek("Expected expression after size of keyword, got nothing")?;

    let is_paren = matches!(token, TokenTypes::Operator(OperatorTypes::LParen));
    if is_paren {
        lexer.advance();
    }

    let token = lexer.force_peek("Expected expression after size of keyword, got nothing")?;

    // this is broken, ill fix later trust
    // let is_valid_expr = is_mutable_l_value(&token);
    let is_valid_expr = true;

    let size_of_data = match token {
        TokenTypes::DataType(_) => {
            if !is_paren {
                return Err(format!("Data type must be surrounded by parenthesis"));
            }

            SizeOf::Type(parse_type(lexer)?)
        }

        _ => {
            if !is_valid_expr {
                return Err(format!("Given sizeof does not contain a valid expression"));
            }

            if is_paren {
                SizeOf::Expr(parse_expression(lexer, 0)?)
            } else {
                // max the precedence so it cant grab anything else
                // this makes it treat it as a unary operator, but can still grab
                // postfix values
                SizeOf::Expr(parse_expression(lexer, u8::MAX)?)
            }
        }
    };

    if is_paren {
        lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;
    }

    return Ok(ExprNode::Sizeof {
        held: Box::new(size_of_data),
    });
}

pub fn parse_unary(lexer: &mut Lexer) -> Result<ExprNode, String> {
    if let Some(TokenTypes::Keyword(KeywordTypes::Sizeof)) = lexer.peek() {
        return parse_size_of(lexer);
    }

    if let Some(TokenTypes::Operator(operator_type)) = lexer.peek()
        && (operator_type.potential_unary())
    {
        lexer.advance();
        let operand = parse_unary(lexer);
        return Ok(ExprNode::Unary {
            operator: operator_type,
            expr: Box::new(operand?),
        });
    }

    return parse_primary(lexer);
}

pub fn parse_accessor_operator(lexer: &mut Lexer) -> Result<ExprNode, String> {
    let mut expr_nodes = Vec::new();

    while let Some(TokenTypes::Operator(OperatorTypes::LSquareBracket)) = lexer.peek() {
        lexer.advance();
        expr_nodes.push(parse_expression(lexer, 0)?);
        lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RSquareBracket)))?;
    }

    let mut final_accessor = ExprNode::Empty;

    // we reverse it because when building the tree
    // we have to start at the most inner most nested part
    for node in expr_nodes.iter().rev() {
        final_accessor = ExprNode::Accessor {
            expr: Box::new(node.clone()),
            nested_accessor: Box::new(final_accessor.clone()),
        };
    }

    Ok(final_accessor)
}

pub fn parse_func_call_args(lexer: &mut Lexer) -> Result<ExprNode, String> {
    let mut args = Vec::new();

    lexer.advance(); // move past the left parenthesis

    while let Some(token) = lexer.peek()
        && !matches!(token, TokenTypes::Operator(OperatorTypes::RParen))
    {
        if matches!(token, TokenTypes::Operator(OperatorTypes::Comma)) {
            return Err(format!("Unexpected comma in function call"));
        }

        // since we don't want to collect commas we start as a higher precedence level
        // commas have a precedence of 2 so we start at 3 instead
        args.push(parse_expression(lexer, 3)?);

        verify_next_in_comma_list(lexer, TokenTypes::Operator(OperatorTypes::RParen), "abc")?;

        if matches!(
            lexer.peek(),
            Some(TokenTypes::Operator(OperatorTypes::Comma))
        ) {
            lexer.advance();
        }
    }
    lexer.advance();

    Ok(ExprNode::FunctionCall {
        args,
        nested_call: Box::new(ExprNode::Empty),
    })
}

fn parse_func_calls(lexer: &mut Lexer) -> Result<ExprNode, String> {
    let mut func_calls = Vec::new();

    while let Some(TokenTypes::Operator(OperatorTypes::LParen)) = lexer.peek() {
        func_calls.push(parse_func_call_args(lexer)?);
    }

    // similar logic to what was done in the accessors
    let mut final_func_call = ExprNode::Empty;

    for func_call in func_calls.iter().rev() {
        let args = match func_call {
            ExprNode::FunctionCall { args, .. } => args.clone(),
            _ => unreachable!(),
        };

        final_func_call = ExprNode::FunctionCall {
            args,
            nested_call: Box::new(final_func_call.clone()),
        };
    }

    Ok(final_func_call)
}

fn parse_member_access(lexer: &mut Lexer) -> Result<ExprNode, String> {
    let mut all_members = Vec::new();

    while let Some(TokenTypes::Operator(op_type)) = lexer.peek() {
        lexer.advance();
        let member = lexer.expect_extract(|x| match x {
            TokenTypes::Identifier(member) => Some(member),
            _ => None,
        })?;

        all_members.push(ExprNode::MemberAccess {
            member,
            operator: op_type,
            next_member: Box::new(ExprNode::Empty),
        });
    }

    // similar logic to what was done in the accessors
    let mut final_member = ExprNode::Empty;

    for member in all_members.iter().rev() {
        let (member_name, operator) = match member {
            ExprNode::MemberAccess {
                member, operator, ..
            } => (member.clone(), operator),
            _ => unreachable!(),
        };

        final_member = ExprNode::MemberAccess {
            member: member_name,
            operator: *operator,
            next_member: Box::new(final_member.clone()),
        };
    }

    Ok(final_member)
}

fn parse_postfix(lexer: &mut Lexer) -> Result<ExprNode, String> {
    let mut node = ExprNode::Empty;

    if let Some(TokenTypes::Identifier(identifier)) = lexer.peek() {
        node = ExprNode::Identifier { identifier };
        lexer.advance();
    }

    while let Some(TokenTypes::Operator(operator_type)) = lexer.peek() {
        match operator_type {
            OperatorTypes::Inc => {
                node = ExprNode::PostFix {
                    left: Box::new(node),
                    right: Box::new(ExprNode::PostInc),
                };
                lexer.advance();
            }
            OperatorTypes::Dec => {
                node = ExprNode::PostFix {
                    left: Box::new(node),
                    right: Box::new(ExprNode::PostDec),
                };
                lexer.advance();
            }

            OperatorTypes::LSquareBracket => {
                node = ExprNode::PostFix {
                    left: Box::new(node),
                    right: Box::new(parse_accessor_operator(lexer)?),
                };
            }

            OperatorTypes::DotOperator | OperatorTypes::ArrowOperator => {
                let accessor_node = parse_member_access(lexer)?;

                node = ExprNode::PostFix {
                    left: Box::new(node),
                    right: Box::new(accessor_node),
                };
            }

            OperatorTypes::LParen => {
                node = ExprNode::PostFix {
                    left: Box::new(node),
                    right: Box::new(parse_func_calls(lexer)?),
                };
            }

            _ => break,
        }
    }

    return Ok(node);
}

fn get_precedence(token: &TokenTypes) -> Option<u8> {
    let precedence = match token {
        TokenTypes::Operator(op) => match op {
            OperatorTypes::Comma => 2,
            OperatorTypes::Colon | OperatorTypes::QuestionMark => 3,
            OperatorTypes::Or => 4,
            OperatorTypes::And => 5,
            OperatorTypes::BitOr => 6,
            OperatorTypes::BitXOR => 7,
            OperatorTypes::Amperstand => 8, // BitAnd
            OperatorTypes::Equal | OperatorTypes::NotEqual => 9,
            OperatorTypes::Greater
            | OperatorTypes::GreaterOrEq
            | OperatorTypes::Less
            | OperatorTypes::LessOrEq => 10,
            OperatorTypes::BitLShift | OperatorTypes::BitRShift => 11,
            OperatorTypes::Plus | OperatorTypes::Minus => 12,
            OperatorTypes::Star | OperatorTypes::Divide | OperatorTypes::Modulus => 13,
            _ => 0,
        },

        TokenTypes::Assignment(_) => 1,

        _ => 0,
    };

    if precedence == 0 {
        return None;
    }

    Some(precedence)
}

pub fn parse_expression(lexer: &mut Lexer, min_precedence: u8) -> Result<ExprNode, String> {
    let mut left = parse_primary(lexer)?;

    while let Some(token) = lexer.peek() {
        let Some(precedence) = get_precedence(&token) else {
            break;
        };

        if precedence < min_precedence {
            break;
        }

        lexer.advance();

        let next_min_precedence = precedence + 1;

        if matches!(token, TokenTypes::Operator(OperatorTypes::QuestionMark)) {
            let middle = parse_expression(lexer, next_min_precedence)?;
            lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::Colon)))?;
            let end = parse_expression(lexer, 0)?;

            left = ExprNode::Ternary {
                if_expr: Box::new(left),
                then_expr: Box::new(middle),
                else_expr: Box::new(end),
            };

            continue;
        }

        let right = parse_expression(lexer, next_min_precedence)?;
        left = ExprNode::Binary {
            left: Box::new(left),
            operator: token,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_primary(lexer: &mut Lexer) -> Result<ExprNode, String> {
    if let Some(token_type) = lexer.peek() {
        match token_type {
            TokenTypes::Literal(literal_type) => match literal_type {
                LiteralTypes::Integer(x) => {
                    lexer.advance();
                    return Ok(ExprNode::Integer { num: x });
                }
                LiteralTypes::Character(x) => {
                    lexer.advance();
                    return Ok(ExprNode::Char { character: (x) });
                }

                LiteralTypes::Float(x) => {
                    lexer.advance();
                    return Ok(ExprNode::Float { num: x });
                }

                LiteralTypes::String(x) => {
                    lexer.advance();
                    return Ok(ExprNode::StrType { string: x });
                }
            },

            TokenTypes::Operator(OperatorTypes::LParen) => {
                let Some(future_token) = lexer.forward_peek() else {
                    return Err(format!("Expected next token in expression, got nothing"));
                };

                let node;
                if matches!(future_token, TokenTypes::DataType(_))
                    | matches!(future_token, TokenTypes::Keyword(KeywordTypes::Struct))
                {
                    node = parse_cast(lexer)?;
                } else {
                    lexer.advance();
                    node = parse_expression(lexer, 0)?;
                    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;
                }

                return Ok(node);
            }

            TokenTypes::Identifier(_) => return parse_postfix(lexer),
            TokenTypes::Operator(op) if op.potential_unary() => return parse_unary(lexer),

            TokenTypes::Keyword(KeywordTypes::Sizeof) => return parse_unary(lexer),

            TokenTypes::LCurlyBrace => {
                return Ok(ExprNode::Aggregate {
                    aggregate: Box::new(parse_aggregate_init(lexer)?),
                });
            }

            _ => {
                return Err(format!(
                    "Expected primary token in expression parser, got token of type {token_type}"
                ));
            }
        }
    }

    if lexer.peek().is_none() {
        return Err(String::from(
            "Expected another token in expression, got nothing",
        ));
    }

    Err(String::from(
        "Next token in expression must be a literal, operator or identifier",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::expression_parser::{ExprNode, parse_expression};
    use crate::parser::helper::run_tests;

    fn parse_expression_generic(lexer: &mut Lexer) -> Result<ExprNode, String> {
        parse_expression(lexer, 0)
    }

    #[test]
    fn expression_primary() {
        let test_cases = vec![
            (r#"x"#, "(Var x)"),
            (r#"a123"#, "(Var a123)"),
            (r#"123"#, "(Num 123)"),
            (r#"0"#, "(Num 0)"),
            (r#"077"#, "(Num 63)"), // this is octal
            (r#"0xFF"#, "(Num 255)"),
            (r#"3.14"#, "(Num 3.14)"),
            (r#"1e10"#, "(Num 10000000000)"),
            (r#"'a'"#, "(Char a)"),
            (r#"'\n'"#, "(Char \\n)"),
            (r#""hello""#, "(Str hello)"),
            (r#"(a)"#, "(Var a)"),
            (r#"((a))"#, "(Var a)"),
            (r#"(((a)))"#, "(Var a)"),
        ];

        run_tests(parse_expression_generic, test_cases);
    }

    #[test]
    fn expression_accessor() {
        let test_cases = vec![
            (r#"a[0]"#, "(Postfix (Var a) (Accessor (Num 0)))"),
            (r#"a[i]"#, "(Postfix (Var a) (Accessor (Var i)))"),
            (
                r#"a[i + 1]"#,
                "(Postfix (Var a) (Accessor (Binary (Var i) (Op +) (Num 1))))",
            ),
            (
                r#"a[x * y]"#,
                "(Postfix (Var a) (Accessor (Binary (Var x) (Op *) (Var y))))",
            ),
            (
                r#"a[b[c]]"#,
                "(Postfix (Var a) (Accessor (Postfix (Var b) (Accessor (Var c)))))",
            ),
            // (r#""#, ""),
            // (r#""#, ""),
            // (r#""#, ""),
        ];

        run_tests(parse_expression_generic, test_cases);
    }

    #[test]
    fn expression_function_call() {
        let test_cases = vec![
            ("f()", "(Postfix (Var f) (FuncCall))"),
            ("f(x)", "(Postfix (Var f) (FuncCall (Var x)))"),
            ("f(x, y)", "(Postfix (Var f) (FuncCall (Var x) (Var y)))"),
            (
                "f(a + b, c * d)",
                "
                (Postfix (Var f) (FuncCall
                    (Binary (Var a) (Op +) (Var b))
                    (Binary (Var c) (Op *) (Var d))
                ))
                ",
            ),
            (
                "f(g(x))",
                "
                (Postfix (Var f) (FuncCall
                    (Postfix (Var g) (FuncCall (Var x)))
                ))

                ",
            ),
            ("f()(x)", "(Postfix (Var f) (FuncCall (FuncCall (Var x))))"),
        ];

        run_tests(parse_expression_generic, test_cases);
    }

    #[test]
    fn expression_members() {
        let test_cases = vec![
            ("s.x", "(Postfix (Var s) (MemberAccess (Var x)))"),
            ("s.y", "(Postfix (Var s) (MemberAccess (Var y)))"),
            ("p->x", "(Postfix (Var p) (PointerAccess (Var x)))"),
            (
                "p->next->value",
                "(Postfix (Var p) (PointerAccess (Var next) (PointerAccess (Var value))))",
            ),
            (
                "a.b.c.d",
                "
                (Postfix (Var a)
                    (MemberAccess (Var b)
                        (MemberAccess (Var c)
                            (MemberAccess (Var d)))))
                ",
            ),
        ];

        run_tests(parse_expression_generic, test_cases);
    }

    #[test]
    fn expression_unary() {
        let test_cases = vec![
            ("+10", "(Unary (Op +) (Num 10))"),
            ("+10", "(Unary (Op +) (Num 10))"),
            ("-10", "(Unary (Op -) (Num 10))"),
            ("!10", "(Unary (Op !) (Num 10))"),
            ("+x", "(Unary (Op +) (Var x))"),
            ("+x", "(Unary (Op +) (Var x))"),
            ("-x", "(Unary (Op -) (Var x))"),
            ("!x", "(Unary (Op !) (Var x))"),
            ("~x", "(Unary (Op ~) (Var x))"),
            ("&x", "(Unary (Op &) (Var x))"),
            ("*x", "(Unary (Op *) (Var x))"),
            ("++x", "(Unary (Op ++) (Var x))"),
            ("--x", "(Unary (Op --) (Var x))"),
            ("*&x", "(Unary (Op *) (Unary (Op &) (Var x)))"),
            ("**x", "(Unary (Op *) (Unary (Op *) (Var x)))"),
            ("++*p", "(Unary (Op ++) (Unary (Op *) (Var p)))"),
            (
                "--a[i]",
                "(Unary (Op --) (Postfix (Var a) (Accessor (Var i))))",
            ),
        ];

        run_tests(parse_expression_generic, test_cases);
    }

    #[test]
    fn expression_sizeof() {
        let test_cases = vec![
            ("sizeof(int)", "(Sizeof (Type int))"),
            ("sizeof x", "(Sizeof (Var x))"),
            ("sizeof(x)", "(Sizeof (Var x))"),
            ("sizeof(a + b)", "(Sizeof (Binary (Var a) (Op +) (Var b)))"),
            ("sizeof(int *)", "(Sizeof (Ptr (Type int)))"),
            ("sizeof(int[10])", "(Sizeof (Arr (Num 10) (Type int)))"),
        ];

        run_tests(parse_expression_generic, test_cases);
    }

    #[test]
    fn expression_casts() {
        let test_cases = vec![
            ("(int)x", "(Cast (Type int) (Var x))"),
            ("(double)i", "(Cast (Type double) (Var i))"),
            ("(int)(double)x", "(Cast (Type double) (Type int) (Var x))"),
            (
                "(char *)(void *)p",
                "(Cast (Ptr (Type void)) (Ptr (Type char)) (Var p))",
            ),
            (
                "(int)(a+b)",
                "(Cast (Type int) (Binary (Var a) (Op +) (Var b)))",
            ),
            ("(char)10", "(Cast (Type char) (Num 10))"),
        ];

        run_tests(parse_expression_generic, test_cases);
    }

    #[test]
    fn expression_ternary() {
        let test_cases = vec![
            (
                "(a > b) ? a : b;",
                "
                (Ternary
                    (Binary (Var a) (Op >) (Var b))
                    (Var a)
                    (Var b)
                )
                ",
            ),
            (
                r#" (num % 2 == 0) ? "Even" : "Odd" "#,
                "
                (Ternary
                    (Binary (Binary (Var num) (Op %) (Num 2)) (Op ==) (Num 0))
                    (Str Even)
                    (Str Odd)
                )
                    
                ",
            ),
            (
                r#" (num > 0) ? "Positive" :
                            (num < 0) ? "Negative" : "Zero";
                        "#,
                "
                (Ternary
                    (Binary (Var num) (Op >) (Num 0))
                    (Str Positive)
                    (Ternary
                        (Binary (Var num) (Op <) (Num 0))
                        (Str Negative)
                        (Str Zero)
                    )
                )
                    
                ",
            ),
            (
                "(a > b)
                        ? ((a > c) ? a : c)
                        : ((b > c) ? b : c);

                        ",
                "
                (Ternary
                    (Binary (Var a) (Op >) (Var b))
                    (Ternary
                        (Binary (Var a) (Op >) (Var c))
                        (Var a)
                        (Var c)
                    )
                    (Ternary
                        (Binary (Var b) (Op >) (Var c))
                        (Var b)
                        (Var c)
                    )
                )
                ",
            ),
            (
                "(x < 0) ? -x : x",
                "
                (Ternary
                    (Binary (Var x) (Op <) (Num 0))
                    (Unary (Op -) (Var x))
                    (Var x)
                )
                ",
            ),
            (
                "flag ? 100.5 : 0.0",
                "
                (Ternary
                    (Var flag)
                    (Num 100.5)
                    (Num 0)
                )                
                ",
            ),
            (
                "(n % 2 == 0) ? square(n) : cube(n);",
                "
                (Ternary
                    (Binary (Binary (Var n) (Op %) (Num 2)) (Op ==) (Num 0))
                    (Postfix (Var square) (FuncCall (Var n)))
                    (Postfix (Var cube) (FuncCall (Var n)))
                )
                ",
            ),
        ];

        run_tests(parse_expression_generic, test_cases);
    }
}
