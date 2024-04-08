use std::fmt;
use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, separated_pair, terminated};
use nom::IResult;

use base::arithmetic::ArithmeticExpression;
use base::column::Column;
use base::error::ParseSQLError;
use base::literal::LiteralExpression;
use base::table::Table;
use base::{CommonParser, DisplayUtil, Literal};

#[derive(Default, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FieldDefinitionExpression {
    #[default]
    All,
    AllInTable(String),
    Col(Column),
    Value(FieldValueExpression),
}

impl FieldDefinitionExpression {
    /// Parse list of column/field definitions.
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
                map(Column::parse, FieldDefinitionExpression::Col),
            )),
            opt(CommonParser::ws_sep_comma),
        ))(i)
    }
}

impl Display for FieldDefinitionExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldDefinitionExpression::All => write!(f, "*"),
            FieldDefinitionExpression::AllInTable(ref table) => {
                write!(f, "{}.*", DisplayUtil::escape_if_keyword(table))
            }
            FieldDefinitionExpression::Col(ref col) => write!(f, "{}", col),
            FieldDefinitionExpression::Value(ref val) => write!(f, "{}", val),
        }
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
                    value: l,
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
        many1(terminated(
            Self::assignment_expr,
            opt(CommonParser::ws_sep_comma),
        ))(i)
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

#[cfg(test)]
mod tests {
    use base::algorithm_type::AlgorithmType;
    use base::arithmetic::ArithmeticBase;
    use base::arithmetic::ArithmeticExpression;
    use base::arithmetic::ArithmeticOperator::{Add, Multiply};
    use base::{FieldDefinitionExpression, FieldValueExpression, Literal};
    use std::vec;

    #[test]
    fn parse_field_definition_expression() {
        let str1 = "*";
        let res1 = FieldDefinitionExpression::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, vec![FieldDefinitionExpression::All]);

        let str2 = "tbl_name.*";
        let res2 = FieldDefinitionExpression::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(
            res2.unwrap().1,
            vec![FieldDefinitionExpression::AllInTable(
                "tbl_name".to_string()
            )]
        );

        let str3 = "age, name, score";
        let res3 = FieldDefinitionExpression::parse(str3);
        let exp = vec![
            FieldDefinitionExpression::Col("age".into()),
            FieldDefinitionExpression::Col("name".into()),
            FieldDefinitionExpression::Col("score".into()),
        ];
        assert!(res3.is_ok());
        assert_eq!(res3.unwrap().1, exp);

        let str4 = "1+2, price * count as total_count";
        let res4 = FieldDefinitionExpression::parse(str4);
        let exp = vec![
            FieldDefinitionExpression::Value(FieldValueExpression::Arithmetic(
                ArithmeticExpression::new(
                    Add,
                    ArithmeticBase::Scalar(Literal::Integer(1)),
                    ArithmeticBase::Scalar(Literal::Integer(2)),
                    None,
                ),
            )),
            FieldDefinitionExpression::Value(FieldValueExpression::Arithmetic(
                ArithmeticExpression::new(
                    Multiply,
                    ArithmeticBase::Column("price".into()),
                    ArithmeticBase::Column("count".into()),
                    Some(String::from("total_count")),
                ),
            )),
        ];
        assert!(res4.is_ok());
        assert_eq!(res4.unwrap().1, exp);
    }
}
