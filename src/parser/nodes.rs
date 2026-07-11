use crate::parser::expression_parser::ExprNode;
use crate::parser::helper::pretty_clean_string;
use crate::parser::if_statement::IfStatement;
use crate::parser::jump_label::JumpLabel;
use crate::parser::tag_types::helper::TagTypeData;
use crate::parser::type_parser::TypeNode;
use std::fmt::Display;

pub trait IndentDisplay {
    fn indent_display(&self, indent: usize) -> String;
}

pub struct Root(pub Vec<GlobalNode>);

impl Display for Root {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        for node in self.0.iter() {
            output.push_str(&format!("{node}\n"));
        }

        if output.chars().last() == Some('\n') {
            output.pop();
        }

        write!(display, "{output}")
    }
}

#[derive(Clone)]
pub enum GlobalNode {
    Function {
        signature: Box<TypeNode>,
        body: Option<StatementNode>,
    },

    Initalizer {
        var_type: TypeNode,
        r_value: Option<ExprNode>,
    },

    TagType(TagTypeData),
}

impl Display for GlobalNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = self.indent_display(0);

        write!(display, "{final_str}")
    }
}

impl IndentDisplay for GlobalNode {
    fn indent_display(&self, indent: usize) -> String {
        let mut output = String::new();

        let str_indent = " ".repeat(indent);

        match self {
            Self::Function { signature, body } => {
                output.push_str(&signature.to_string());

                if let Some(body) = body {
                    output.pop();
                    output.push_str(&format!("\n{}", body.indent_display(indent + 2)));
                    output.push_str(")");
                }
            }

            Self::Initalizer { var_type, r_value } => {
                output.push_str(&format!("{str_indent}(Variable {var_type}"));

                if let Some(expression) = r_value.clone() {
                    output.push_str(&format!("\n{}", &expression.display(indent + 2)));
                }

                output.push_str(")");
            }

            Self::TagType(data) => {
                output.push_str(&data.indent_display(indent));
            }
        }

        output
    }
}

#[derive(Clone)]
pub enum StatementNode {
    // block, expression, if, switch, while, do, for, return, break, continue, goto, label, case, default
    Block {
        statements: Vec<StatementNode>,
    },

    General(Box<GlobalNode>),
    Expression(ExprNode),

    Return(Option<ExprNode>),
    Break,
    Continue,
    If(Box<IfStatement>),

    While {
        conditional: ExprNode,
        body: Box<StatementNode>,
    },

    DoWhile {
        conditional: ExprNode,
        body: Box<StatementNode>,
    },

    For {
        init: Option<Box<StatementNode>>,
        condition: Option<ExprNode>,
        iteration: Option<ExprNode>,
        body: Box<StatementNode>,
    },

    Semicolon,

    Switch {
        case_label: ExprNode,
        body: Box<StatementNode>,
    },

    JumpLabel(JumpLabel),
    GotoStatement(String),
}

impl Display for StatementNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = self.indent_display(0);

        write!(display, "{final_str}")
    }
}

impl IndentDisplay for StatementNode {
    fn indent_display(&self, indent: usize) -> String {
        let mut output = String::new();
        let str_indent = " ".repeat(indent);
        let next_str_indent = " ".repeat(indent + 2);

        match self {
            Self::Block { statements } => {
                for (i, statement) in statements.iter().enumerate() {
                    output.push_str(&format!("{}", statement.indent_display(indent)));
                    if i != statements.len() - 1 {
                        output.push('\n');
                    }
                }
            }

            Self::Expression(expr) => {
                output.push_str(&format!(
                    "{str_indent}(Expr\n{})",
                    expr.clone().display(indent + 2)
                ));
            }

            Self::General(global_node) => {
                output.push_str(&global_node.indent_display(indent));
            }

            Self::Return(expr) => {
                output.push_str(&format!("{str_indent}(Return"));

                if let Some(expr) = expr {
                    output.push_str(&format!(" {}", &pretty_clean_string(&expr.to_string())));
                }

                output.push(')');
            }

            Self::If(if_statement) => {
                output.push_str(&if_statement.indent_display(indent));
            }

            Self::Semicolon => {
                output.push_str(&format!("{str_indent}(Op ;)"));
            }

            Self::Break => {
                output.push_str(&format!("{str_indent}(Break)"));
            }

            Self::Continue => {
                output.push_str(&format!("{str_indent}(Continue)"));
            }

            Self::While { conditional, body } => {
                output.push_str(&format!("{str_indent}(While\n"));
                output.push_str(&format!(
                    "{next_str_indent}(Condition\n{})\n",
                    conditional.clone().display(indent + 4)
                ));

                output.push_str(&format!(
                    "{next_str_indent}(Body\n{})",
                    body.indent_display(indent + 4)
                ));
            }

            Self::DoWhile { conditional, body } => {
                output.push_str(&format!("{str_indent}(DoWhile\n"));
                output.push_str(&format!(
                    "{next_str_indent}(Condition\n{})\n",
                    conditional.clone().display(indent + 4)
                ));

                output.push_str(&format!(
                    "{next_str_indent}(Body\n{})",
                    body.indent_display(indent + 4)
                ));
            }

            Self::For {
                init,
                condition,
                iteration,
                body,
            } => {
                output.push_str(&format!("{str_indent}(For\n"));

                output.push_str(&format!("{next_str_indent}(Init"));
                if let Some(init) = init {
                    output.push_str(&format!("\n{}", init.indent_display(indent + 4)));
                }
                output.push(')');

                output.push_str(&format!("\n{next_str_indent}(Body"));
                if let Some(condition) = condition {
                    output.push_str(&format!("\n{}", condition.clone().display(indent + 4)));
                }

                output.push(')');

                output.push_str(&format!("\n{next_str_indent}(Iterate"));
                if let Some(iteration) = iteration {
                    output.push_str(&format!("\n{}", iteration.clone().display(indent + 4)));
                }

                output.push(')');

                output.push_str(&format!(
                    "\n{next_str_indent}(Body\n{})",
                    body.indent_display(indent + 4)
                ));
            }

            Self::Switch { case_label, body } => {
                output.push_str(&format!(
                    "{str_indent}(Switch (CaseLabel {})\n{})",
                    pretty_clean_string(&case_label.to_string()),
                    body.indent_display(indent + 2)
                ));
            }

            Self::JumpLabel(jump_label) => {
                output.push_str(&format!("{str_indent}{jump_label}"));
            }

            Self::GotoStatement(label) => {
                output.push_str(&format!("{str_indent}(Goto {label})"));
            }
        }

        output
    }
}
