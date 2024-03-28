use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::str;

use common_parsers::{Literal, SqlDataType};
use keywords::escape_if_keyword;
use zz_case::CaseWhenExpression;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FunctionExpression {
    Avg(FunctionArgument, bool),
    Count(FunctionArgument, bool),
    CountStar,
    Sum(FunctionArgument, bool),
    Max(FunctionArgument),
    Min(FunctionArgument),
    GroupConcat(FunctionArgument, String),
    Generic(String, FunctionArguments),
}

impl Display for FunctionExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FunctionExpression::Avg(ref col, d) if d => write!(f, "avg(distinct {})", col),
            FunctionExpression::Count(ref col, d) if d => write!(f, "count(distinct {})", col),
            FunctionExpression::Sum(ref col, d) if d => write!(f, "sum(distinct {})", col),
            FunctionExpression::Avg(ref col, _) => write!(f, "avg({})", col),
            FunctionExpression::Count(ref col, _) => write!(f, "count({})", col),
            FunctionExpression::CountStar => write!(f, "count(*)"),
            FunctionExpression::Sum(ref col, _) => write!(f, "sum({})", col),
            FunctionExpression::Max(ref col) => write!(f, "max({})", col),
            FunctionExpression::Min(ref col) => write!(f, "min({})", col),
            FunctionExpression::GroupConcat(ref col, ref s) => {
                write!(f, "group_concat({}, {})", col, s)
            }
            FunctionExpression::Generic(ref name, ref args) => write!(f, "{}({})", name, args),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FunctionArguments {
    pub arguments: Vec<FunctionArgument>,
}

impl Display for FunctionArguments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.arguments
                .iter()
                .map(|arg| format!("{}", arg))
                .collect::<Vec<String>>()
                .join(",")
        )?;
        Ok(())
    }
}

impl<'a> From<Vec<FunctionArgument>> for FunctionArguments {
    fn from(args: Vec<FunctionArgument>) -> FunctionArguments {
        FunctionArguments { arguments: args }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FunctionArgument {
    Column(Column),
    Conditional(CaseWhenExpression),
}

impl Display for FunctionArgument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FunctionArgument::Column(ref col) => write!(f, "{}", col)?,
            FunctionArgument::Conditional(ref e) => {
                write!(f, "{}", e)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub alias: Option<String>,
    pub table: Option<String>,
    pub function: Option<Box<FunctionExpression>>,
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref table) = self.table {
            write!(
                f,
                "{}.{}",
                escape_if_keyword(table),
                escape_if_keyword(&self.name)
            )?;
        } else if let Some(ref function) = self.function {
            write!(f, "{}", *function)?;
        } else {
            write!(f, "{}", escape_if_keyword(&self.name))?;
        }
        if let Some(ref alias) = self.alias {
            write!(f, " AS {}", escape_if_keyword(alias))?;
        }
        Ok(())
    }
}

impl From<String> for Column {
    fn from(value: String) -> Self {
        match value.find(".") {
            None => Column {
                name: value,
                alias: None,
                table: None,
                function: None,
            },
            Some(i) => Column {
                name: String::from(&value[i + 1..]),
                alias: None,
                table: Some(String::from(&value[0..i])),
                function: None,
            },
        }
    }
}

impl<'a> From<&'a str> for Column {
    fn from(c: &str) -> Column {
        match c.find(".") {
            None => Column {
                name: String::from(c),
                alias: None,
                table: None,
                function: None,
            },
            Some(i) => Column {
                name: String::from(&c[i + 1..]),
                alias: None,
                table: Some(String::from(&c[0..i])),
                function: None,
            },
        }
    }
}

impl Ord for Column {
    fn cmp(&self, other: &Column) -> Ordering {
        if self.table.is_some() && other.table.is_some() {
            match self.table.cmp(&other.table) {
                Ordering::Equal => self.name.cmp(&other.name),
                x => x,
            }
        } else {
            self.name.cmp(&other.name)
        }
    }
}

impl PartialOrd for Column {
    fn partial_cmp(&self, other: &Column) -> Option<Ordering> {
        if self.table.is_some() && other.table.is_some() {
            match self.table.cmp(&other.table) {
                Ordering::Equal => Some(self.name.cmp(&other.name)),
                x => Some(x),
            }
        } else if self.table.is_none() && other.table.is_none() {
            Some(self.name.cmp(&other.name))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ColumnConstraint {
    NotNull,
    CharacterSet(String),
    Collation(String),
    DefaultValue(Literal),
    AutoIncrement,
    PrimaryKey,
    Unique,
}

impl fmt::Display for ColumnConstraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ColumnConstraint::NotNull => write!(f, "NOT NULL"),
            ColumnConstraint::CharacterSet(ref charset) => write!(f, "CHARACTER SET {}", charset),
            ColumnConstraint::Collation(ref collation) => write!(f, "COLLATE {}", collation),
            ColumnConstraint::DefaultValue(ref literal) => {
                write!(f, "DEFAULT {}", literal.to_string())
            }
            ColumnConstraint::AutoIncrement => write!(f, "AutoIncrement"),
            ColumnConstraint::PrimaryKey => write!(f, "PRIMARY KEY"),
            ColumnConstraint::Unique => write!(f, "UNIQUE"),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MySQLColumnPosition {
    First,
    After(Column),
}

impl Display for MySQLColumnPosition {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            MySQLColumnPosition::First => Ok(write!(f, "FIRST")?),
            MySQLColumnPosition::After(column) => {
                let column_name = match &column.table {
                    Some(table) => format!("{}.{}", table, &column.name),
                    None => format!("{}", &column.name),
                };
                Ok(write!(f, "AFTER {column_name}")?)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ColumnSpecification {
    pub column: Column,
    pub sql_type: SqlDataType,
    pub constraints: Vec<ColumnConstraint>,
    pub comment: Option<String>,
    pub position: Option<MySQLColumnPosition>,
}

impl fmt::Display for ColumnSpecification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            escape_if_keyword(&self.column.name),
            self.sql_type
        )?;
        for constraint in self.constraints.iter() {
            write!(f, " {}", constraint)?;
        }
        if let Some(ref comment) = self.comment {
            write!(f, " COMMENT '{}'", comment)?;
        }
        Ok(())
    }
}

impl ColumnSpecification {
    pub fn new(column: Column, sql_type: SqlDataType) -> ColumnSpecification {
        ColumnSpecification {
            column,
            sql_type,
            constraints: vec![],
            comment: None,
            position: None,
        }
    }

    pub fn with_constraints(
        column: Column,
        sql_type: SqlDataType,
        constraints: Vec<ColumnConstraint>,
    ) -> ColumnSpecification {
        ColumnSpecification {
            column,
            sql_type,
            constraints,
            comment: None,
            position: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_from_str() {
        let s = "table.col";
        let c = Column::from(s);

        assert_eq!(
            c,
            Column {
                name: String::from("col"),
                alias: None,
                table: Some(String::from("table")),
                function: None,
            }
        );
    }

    #[test]
    fn print_function_column() {
        let c1 = Column {
            name: "".into(), // must be present, but will be ignored
            alias: Some("foo".into()),
            table: None,
            function: Some(Box::new(FunctionExpression::CountStar)),
        };
        let c2 = Column {
            name: "".into(), // must be present, but will be ignored
            alias: None,
            table: None,
            function: Some(Box::new(FunctionExpression::CountStar)),
        };
        let c3 = Column {
            name: "".into(), // must be present, but will be ignored
            alias: None,
            table: None,
            function: Some(Box::new(FunctionExpression::Sum(
                FunctionArgument::Column(Column::from("mytab.foo")),
                false,
            ))),
        };

        assert_eq!(format!("{}", c1), "count(*) AS foo");
        assert_eq!(format!("{}", c2), "count(*)");
        assert_eq!(format!("{}", c3), "sum(mytab.foo)");
    }
}
