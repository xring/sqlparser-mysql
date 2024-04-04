use std::fmt;
use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, separated_pair, terminated};
use nom::IResult;

use base::column::Column;
use base::error::ParseSQLError;
use base::literal::LiteralExpression;
use base::table::Table;
use base::Literal;
use common::arithmetic::ArithmeticExpression;
use common::keywords::escape_if_keyword;
use common::ws_sep_comma;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FieldDefinitionExpression {
    All,
    AllInTable(String),
    Col(Column),
    Value(FieldValueExpression),
}

impl FieldDefinitionExpression {
    // Parse list of column/field definitions.
    pub fn parse(i: &str) -> IResult<&str, Vec<FieldDefinitionExpression>, ParseSQLError<&str>> {
        many0(terminated(
            alt((
                map(tag("*"), |_| FieldDefinitionExpression::All),
                map(terminated(Table::table_reference, tag(".*")), |t| {
                    FieldDefinitionExpression::AllInTable(t.name.clone())
                }),
                map(ArithmeticExpression::parse, |expr| {
                    FieldDefinitionExpression::Value(FieldValueExpression::Arithmetic(expr))
                }),
                map(LiteralExpression::parse, |lit| {
                    FieldDefinitionExpression::Value(FieldValueExpression::Literal(lit))
                }),
                map(Column::parse, |col| FieldDefinitionExpression::Col(col)),
            )),
            opt(ws_sep_comma),
        ))(i)
    }
}

impl Display for FieldDefinitionExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldDefinitionExpression::All => write!(f, "*"),
            FieldDefinitionExpression::AllInTable(ref table) => {
                write!(f, "{}.*", escape_if_keyword(table))
            }
            FieldDefinitionExpression::Col(ref col) => write!(f, "{}", col),
            FieldDefinitionExpression::Value(ref val) => write!(f, "{}", val),
        }
    }
}

impl Default for FieldDefinitionExpression {
    fn default() -> FieldDefinitionExpression {
        FieldDefinitionExpression::All
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FieldValueExpression {
    Arithmetic(ArithmeticExpression),
    Literal(LiteralExpression),
}

impl FieldValueExpression {
    fn parse(i: &str) -> IResult<&str, FieldValueExpression, ParseSQLError<&str>> {
        alt((
            map(Literal::parse, |l| {
                FieldValueExpression::Literal(LiteralExpression {
                    value: l.into(),
                    alias: None,
                })
            }),
            map(ArithmeticExpression::parse, |ae| {
                FieldValueExpression::Arithmetic(ae)
            }),
        ))(i)
    }

    fn assignment_expr(
        i: &str,
    ) -> IResult<&str, (Column, FieldValueExpression), ParseSQLError<&str>> {
        separated_pair(
            Column::without_alias,
            delimited(multispace0, tag("="), multispace0),
            Self::parse,
        )(i)
    }

    pub fn assignment_expr_list(
        i: &str,
    ) -> IResult<&str, Vec<(Column, FieldValueExpression)>, ParseSQLError<&str>> {
        many1(terminated(Self::assignment_expr, opt(ws_sep_comma)))(i)
    }
}

impl Display for FieldValueExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldValueExpression::Arithmetic(ref expr) => write!(f, "{}", expr),
            FieldValueExpression::Literal(ref lit) => write!(f, "{}", lit),
        }
    }
}
