use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::TokenTypes;
use crate::parser::expression_parser::ExprNode;
use crate::parser::nodes::IndentDisplay;
use crate::parser::nodes::StatementNode;
use crate::parser::parser::Parser;

#[derive(Clone)]
pub enum IfStatement {
    If {
        conditional: ExprNode,
        body: Box<StatementNode>, // either an expression or block
        chain: Option<Box<IfStatement>>,
    },
    ElseIf {
        conditional: ExprNode,
        body: Box<StatementNode>,
        chain: Option<Box<IfStatement>>,
    },
    Else(StatementNode),
}

impl IndentDisplay for IfStatement {
    fn indent_display(&self, indent: usize) -> String {
        let mut output = String::new();
        let indent_str = " ".repeat(indent);
        let next_indent_str = " ".repeat(indent + 2);

        match self {
            Self::If { .. } => output.push_str(&format!("{indent_str}(If")),
            Self::ElseIf { .. } => output.push_str(&format!("{indent_str}(ElseIf")),
            Self::Else { .. } => output.push_str(&format!("{indent_str}(Else")),
        }

        match self {
            Self::If {
                conditional,
                body,
                chain,
            }
            | Self::ElseIf {
                conditional,
                body,
                chain,
            } => {
                output.push_str(&format!(
                    "\n{next_indent_str}(Condition \n{})\n{next_indent_str}(Body\n{}\n{next_indent_str})",
                    &conditional.clone().display(indent + 4),
                    &body.clone().indent_display(indent + 4)
                ));

                if let Some(chain) = chain {
                    output.push_str(&format!(
                        "\n{next_indent_str}(Chain\n{}\n{next_indent_str})",
                        chain.indent_display(indent + 4)
                    ));
                }

                output.push(')');
            }
            Self::Else(body) => {
                output.push_str(&format!(
                    "\n{next_indent_str}(Body\n{}\n{next_indent_str})",
                    &body.clone().indent_display(indent + 4)
                ));
            }
        }

        output
    }
}

impl Parser {
    pub fn parse_if_statement(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::If)))?;

        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;
        let conditional = self.parse_expression(0)?;
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;

        let next_token = self
            .lexer
            .force_peek("Expected if statement to have a body, got nothing")?;

        let body;
        if matches!(next_token, TokenTypes::LCurlyBrace) {
            self.lexer.advance();
            body = Box::new(self.parse_block()?);
        } else {
            body = Box::new(self.parse_single_statement()?)
        }

        let next_token = self.lexer.force_peek("Unexpected end to body")?;

        let mut chain = None;
        if matches!(next_token, TokenTypes::Keyword(KeywordTypes::Else)) {
            chain = Some(Box::new(self.parse_else_if_statement()?));
        }

        let final_if = IfStatement::If {
            conditional,
            body,
            chain,
        };

        Ok(StatementNode::If(Box::new(final_if)))
    }

    fn parse_else_if_statement(&mut self) -> Result<IfStatement, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Else)))?;

        let next_token = self
            .lexer
            .force_peek("Expected else statement to have a body, got nothing")?;

        // else statement
        if !matches!(next_token, TokenTypes::Keyword(KeywordTypes::If)) {
            let next_token = self
                .lexer
                .force_peek("Expected else statement to have a body, got nothing")?;

            if matches!(next_token, TokenTypes::LCurlyBrace) {
                self.lexer.advance();
                return Ok(IfStatement::Else(self.parse_block()?));
            }

            return Ok(IfStatement::Else(self.parse_single_statement()?));
        }

        // else if
        let StatementNode::If(chained) = self.parse_if_statement()? else {
            unreachable!();
        };

        let IfStatement::If {
            conditional,
            body,
            chain,
        } = *chained
        else {
            unreachable!();
        };

        Ok(IfStatement::ElseIf {
            conditional,
            body,
            chain,
        })
    }
}
#[cfg(test)]
mod tests {
    use crate::parser::{helper::run_tests, parser::Parser};

    #[test]
    fn if_statement() {
        let test_cases = vec![
            (
                "int main(){ if (1) ; } ",
                "
                (Function main (Return (Type int))
                    (If (Condition (Num 1))
                        (Body (Op ;))))
                ",
            ),
            (
                "int main(){ if (x > 5) y++; } ",
                "
                (Function main (Return (Type int))
                    (If
                        (Condition  (Binary (Var x) (Op >) (Num 5)))
                        (Body (Expr (Postfix (Var y) (PostInc)))
                )))
                ",
            ),
            (
                "
                int main(){
                    if (x) y++;
                    else y--;
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var x))
                    (Body
                        (Expr (Postfix (Var y) (PostInc))))
                    (Chain (Else
                        (Body (Expr (Postfix (Var y) (PostDec))))))
                )
                ",
            ),
            (
                "
                int main(){
                    if (a)
                        if (b)
                            c++;
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var a))
                    (Body
                        (If
                            (Condition (Var b))
                            (Body (Expr (Postfix (Var c) (PostInc))))
                        )
                    )
                ))
                ",
            ),
            (
                "
                int main(){
                    if (a)
                        if (b)
                            c++;
                        else
                            d++;
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var a))
                    (Body
                        (If
                            (Condition (Var b))
                            (Body (Expr (Postfix (Var c) (PostInc))))
                            (Chain
                                (Else (Body (Expr (Postfix (Var d) (PostInc))))
                    ))
                )))
                ",
            ),
            (
                "
                int main(){
                    if (a) {
                        if (b)
                            c++;
                    }
                    else {
                        d++;
                    }
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var a))
                    (Body
                        (If (Condition (Var b))
                        (Body (Expr (Postfix (Var c) (PostInc))))))
                        (Chain (Else
                            (Body (Expr (Postfix (Var d) (PostInc))))))
                )
                ",
            ),
            (
                "
                int main(){
                    if (x) ;
                    else ;
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var x))
                    (Body (Op ;))
                    (Chain (Else
                        (Body (Op ;))))
                )
                ",
            ),
            (
                "
                int main(){
                    if (x == 0)
                        a();
                    else if (x == 1)
                        b();
                    else if (x == 2)
                        c();
                    else
                        d();
                }
                ",
                "
                (Function main (Return (Type int))
                  (If
                    (Condition (Binary  (Var x)  (Op ==)  (Num 0)))
                    (Body (Expr (Postfix (Var a) (FuncCall))))
                    (Chain
                        (ElseIf
                            (Condition (Binary (Var x) (Op ==) (Num 1)))
                            (Body (Expr (Postfix (Var b) (FuncCall))))
                            (Chain
                                (ElseIf
                                    (Condition (Binary (Var x) (Op ==) (Num 2)))
                                    (Body (Expr (Postfix (Var c) (FuncCall))))
                                    (Chain
                                        (Else
                                            (Body (Expr (Postfix (Var d) (FuncCall)))))))))))
                ",
            ),
        ];

        run_tests(Parser::parse_program, test_cases);
    }
}
