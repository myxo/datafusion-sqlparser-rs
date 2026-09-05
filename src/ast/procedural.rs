use core::fmt::{self, Display};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "visitor")]
use sqlparser_derive::{Visit, VisitMut};

use super::{DataType, Expr, Ident, ObjectName, Statement, ValueWithSpan};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "visitor", derive(Visit, VisitMut))]
/// A bounded PL/pgSQL block.
pub struct PlPgSqlBlock {
    /// Local variable declarations.
    pub declarations: Vec<PlPgSqlDeclaration>,
    /// Statements in the block body.
    pub statements: Vec<PlPgSqlStatement>,
}

impl Display for PlPgSqlBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.declarations.is_empty() {
            write!(f, "DECLARE ")?;
            for declaration in &self.declarations {
                write!(f, "{declaration}; ")?;
            }
        }
        write!(f, "BEGIN")?;
        for statement in &self.statements {
            write!(f, " {statement};")?;
        }
        write!(f, " END")
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "visitor", derive(Visit, VisitMut))]
/// A PL/pgSQL local variable declaration.
pub struct PlPgSqlDeclaration {
    /// Variable name.
    pub name: Ident,
    /// Declared SQL type.
    pub data_type: DataType,
    /// Optional initializer.
    pub initializer: Option<Expr>,
}

impl Display for PlPgSqlDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.data_type)?;
        if let Some(initializer) = &self.initializer {
            write!(f, " := {initializer}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "visitor", derive(Visit, VisitMut))]
/// A statement in a bounded PL/pgSQL block.
pub enum PlPgSqlStatement {
    /// Assign an expression to a name or field.
    Assignment {
        /// Assignment target.
        target: ObjectName,
        /// Assigned expression.
        value: Expr,
    },
    /// Conditional branches.
    If {
        /// Ordered `IF` and `ELSIF` branches.
        branches: Vec<PlPgSqlIfBranch>,
        /// Optional `ELSE` statements.
        else_statements: Option<Vec<PlPgSqlStatement>>,
    },
    /// Return an expression.
    Return(Expr),
    /// Execute an embedded SQL statement.
    Sql(Box<Statement>),
    /// Assign the affected-row count of the preceding SQL statement.
    GetRowCount {
        /// Destination variable.
        target: Ident,
    },
    /// Raise an exception with formatted arguments.
    RaiseException {
        /// Literal format string.
        format: ValueWithSpan,
        /// Format substitution arguments.
        arguments: Vec<Expr>,
        /// Optional error hint.
        hint: Option<Expr>,
    },
}

impl Display for PlPgSqlStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assignment { target, value } => write!(f, "{target} := {value}"),
            Self::If {
                branches,
                else_statements,
            } => {
                for (index, branch) in branches.iter().enumerate() {
                    if index == 0 {
                        write!(f, "IF {} THEN", branch.condition)?;
                    } else {
                        write!(f, " ELSIF {} THEN", branch.condition)?;
                    }
                    for statement in &branch.statements {
                        write!(f, " {statement};")?;
                    }
                }
                if let Some(statements) = else_statements {
                    write!(f, " ELSE")?;
                    for statement in statements {
                        write!(f, " {statement};")?;
                    }
                }
                write!(f, " END IF")
            }
            Self::Return(expression) => write!(f, "RETURN {expression}"),
            Self::Sql(statement) => statement.fmt(f),
            Self::GetRowCount { target } => {
                write!(f, "GET DIAGNOSTICS {target} = ROW_COUNT")
            }
            Self::RaiseException {
                format,
                arguments,
                hint,
            } => {
                write!(f, "RAISE EXCEPTION {format}")?;
                for argument in arguments {
                    write!(f, ", {argument}")?;
                }
                if let Some(hint) = hint {
                    write!(f, " USING HINT = {hint}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "visitor", derive(Visit, VisitMut))]
/// One conditional PL/pgSQL branch.
pub struct PlPgSqlIfBranch {
    /// Branch condition.
    pub condition: Expr,
    /// Statements run when the condition is true.
    pub statements: Vec<PlPgSqlStatement>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "visitor", derive(Visit, VisitMut))]
/// A PostgreSQL anonymous code block.
pub struct DoStatement {
    /// Raw string body.
    pub body: ValueWithSpan,
    /// Optional procedural language.
    pub language: Option<Ident>,
    /// Whether `LANGUAGE` appeared before the body.
    pub language_before_body: bool,
}

impl Display for DoStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DO")?;
        if self.language_before_body {
            if let Some(language) = &self.language {
                write!(f, " LANGUAGE {language}")?;
            }
        }
        write!(f, " {}", self.body)?;
        if !self.language_before_body {
            if let Some(language) = &self.language {
                write!(f, " LANGUAGE {language}")?;
            }
        }
        Ok(())
    }
}
