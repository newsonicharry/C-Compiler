use crate::parser::expression_parser::ExprNode;
use crate::parser::helper::pretty_clean_string;
use crate::parser::if_statement::IfStatement;
use crate::parser::jump_label::JumpLabel;
use crate::parser::tag_types::enum_parser::EnumMember;
use crate::parser::tag_types::helper::TagType;
use crate::parser::tag_types::struct_parser::StructMember;
use crate::parser::tag_types::union_parser::UnionMember;
use crate::parser::type_parser::TypeNode;
use std::fmt::Display;

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

    Union(TagType<UnionMember>),
    Struct(TagType<StructMember>),
    Enum(TagType<EnumMember>),
    // Typedef is on an eternal todo list
    // It'll be done, just not right now...
}

impl Display for GlobalNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = self.display(0);

        write!(display, "{final_str}")
    }
}

impl GlobalNode {
    fn display(&self, indentation: usize) -> String {
        let mut output = String::new();

        let str_indent = " ".repeat(indentation);

        match self {
            Self::Function { signature, body } => {
                output.push_str(&signature.to_string());

                if let Some(body) = body {
                    output.pop();
                    output.push_str(&format!("\n{}", body.display(indentation + 2)));
                    output.push_str(")");
                }
            }

            Self::Initalizer { var_type, r_value } => {
                output.push_str(&format!("{str_indent}(Variable {var_type}"));

                if let Some(expression) = r_value.clone() {
                    output.push_str(&format!("\n{}", &expression.display(indentation + 2)));
                }

                output.push_str(")");
            }

            Self::Struct(data) => {
                output.push_str(&data.display(indentation));
            }

            Self::Enum(data) => {
                output.push_str(&data.display(indentation));
            }

            Self::Union(data) => {
                output.push_str(&data.display(indentation));
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
        let final_str = Self::display(self, 0);

        write!(display, "{final_str}")
    }
}

impl StatementNode {
    pub fn display(&self, indentation: usize) -> String {
        let mut output = String::new();
        let str_indent = " ".repeat(indentation);
        let next_str_indent = " ".repeat(indentation + 2);

        match self {
            Self::Block { statements } => {
                for (i, statement) in statements.iter().enumerate() {
                    output.push_str(&format!("{}", statement.display(indentation)));
                    if i != statements.len() - 1 {
                        output.push('\n');
                    }
                }
            }

            Self::Expression(expr) => {
                output.push_str(&format!(
                    "{str_indent}(Expr\n{})",
                    expr.clone().display(indentation + 2)
                ));
            }

            Self::General(global_node) => {
                output.push_str(&global_node.display(indentation));
            }

            Self::Return(expr) => {
                output.push_str(&format!("{str_indent}(Return"));

                if let Some(expr) = expr {
                    output.push_str(&format!(" {}", &pretty_clean_string(&expr.to_string())));
                }

                output.push(')');
            }

            Self::If(if_statement) => {
                output.push_str(&if_statement.display(indentation));
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
                    conditional.clone().display(indentation + 4)
                ));

                output.push_str(&format!(
                    "{next_str_indent}(Body\n{})",
                    body.display(indentation + 4)
                ));
            }

            Self::DoWhile { conditional, body } => {
                output.push_str(&format!("{str_indent}(DoWhile\n"));
                output.push_str(&format!(
                    "{next_str_indent}(Condition\n{})\n",
                    conditional.clone().display(indentation + 4)
                ));

                output.push_str(&format!(
                    "{next_str_indent}(Body\n{})",
                    body.display(indentation + 4)
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
                    output.push_str(&format!("\n{}", init.display(indentation + 4)));
                }
                output.push(')');

                output.push_str(&format!("\n{next_str_indent}(Body"));
                if let Some(condition) = condition {
                    output.push_str(&format!("\n{}", condition.clone().display(indentation + 4)));
                }

                output.push(')');

                output.push_str(&format!("\n{next_str_indent}(Iterate"));
                if let Some(iteration) = iteration {
                    output.push_str(&format!("\n{}", iteration.clone().display(indentation + 4)));
                }

                output.push(')');

                output.push_str(&format!(
                    "\n{next_str_indent}(Body\n{})",
                    body.display(indentation + 4)
                ));
            }

            Self::Switch { case_label, body } => {
                output.push_str(&format!(
                    "{str_indent}(Switch (CaseLabel {})\n{})",
                    pretty_clean_string(&case_label.to_string()),
                    body.display(indentation + 2)
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
