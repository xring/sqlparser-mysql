use std::{fmt, str};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{multispace0, multispace1},
    combinator::{map, opt},
    lib::std::fmt::Formatter,
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Err::Error,
    IResult,
};

use base::Column;
use base::ParseSQLErrorKind;
use base::{CommonParser, DataType, Literal, ParseSQLError};

#[derive(Debug, Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl ArithmeticOperator {
    fn add_sub_operator(i: &str) -> IResult<&str, ArithmeticOperator, ParseSQLError<&str>> {
        alt((
            map(tag("+"), |_| ArithmeticOperator::Add),
            map(tag("-"), |_| ArithmeticOperator::Subtract),
        ))(i)
    }

    fn mul_div_operator(i: &str) -> IResult<&str, ArithmeticOperator, ParseSQLError<&str>> {
        alt((
            map(tag("*"), |_| ArithmeticOperator::Multiply),
            map(tag("/"), |_| ArithmeticOperator::Divide),
        ))(i)
    }
}

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArithmeticOperator::Add => write!(f, "+"),
            ArithmeticOperator::Subtract => write!(f, "-"),
            ArithmeticOperator::Multiply => write!(f, "*"),
            ArithmeticOperator::Divide => write!(f, "/"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ArithmeticBase {
    Column(Column),
    Scalar(Literal),
    Bracketed(Box<Arithmetic>),
}

impl ArithmeticBase {
    // Base case for nested arithmetic expressions: column name or literal.
    fn parse(i: &str) -> IResult<&str, ArithmeticBase, ParseSQLError<&str>> {
        alt((
            map(Literal::integer_literal, ArithmeticBase::Scalar),
            map(Column::without_alias, ArithmeticBase::Column),
            map(
                delimited(
                    terminated(tag("("), multispace0),
                    Arithmetic::parse,
                    preceded(multispace0, tag(")")),
                ),
                |ari| ArithmeticBase::Bracketed(Box::new(ari)),
            ),
        ))(i)
    }
}

impl fmt::Display for ArithmeticBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArithmeticBase::Column(ref col) => write!(f, "{}", col),
            ArithmeticBase::Scalar(ref lit) => write!(f, "{}", lit.to_string()),
            ArithmeticBase::Bracketed(ref ari) => write!(f, "({})", ari),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ArithmeticItem {
    Base(ArithmeticBase),
    Expr(Box<Arithmetic>),
}

impl ArithmeticItem {
    fn term(i: &str) -> IResult<&str, ArithmeticItem, ParseSQLError<&str>> {
        map(
            pair(Self::arithmetic_cast, many0(Self::term_rest)),
            |(b, rs)| {
                rs.into_iter()
                    .fold(ArithmeticItem::Base(b.0), |acc, (o, r)| {
                        ArithmeticItem::Expr(Box::new(Arithmetic {
                            op: o,
                            left: acc,
                            right: r,
                        }))
                    })
            },
        )(i)
    }

    fn term_rest(
        i: &str,
    ) -> IResult<&str, (ArithmeticOperator, ArithmeticItem), ParseSQLError<&str>> {
        separated_pair(
            preceded(multispace0, ArithmeticOperator::mul_div_operator),
            multispace0,
            map(Self::arithmetic_cast, |b| ArithmeticItem::Base(b.0)),
        )(i)
    }

    fn expr(i: &str) -> IResult<&str, ArithmeticItem, ParseSQLError<&str>> {
        map(
            pair(ArithmeticItem::term, many0(Self::expr_rest)),
            |(item, rs)| {
                rs.into_iter().fold(item, |acc, (o, r)| {
                    ArithmeticItem::Expr(Box::new(Arithmetic {
                        op: o,
                        left: acc,
                        right: r,
                    }))
                })
            },
        )(i)
    }

    fn expr_rest(
        i: &str,
    ) -> IResult<&str, (ArithmeticOperator, ArithmeticItem), ParseSQLError<&str>> {
        separated_pair(
            preceded(multispace0, ArithmeticOperator::add_sub_operator),
            multispace0,
            ArithmeticItem::term,
        )(i)
    }

    fn arithmetic_cast(
        i: &str,
    ) -> IResult<&str, (ArithmeticBase, Option<DataType>), ParseSQLError<&str>> {
        alt((
            Self::arithmetic_cast_helper,
            map(ArithmeticBase::parse, |v| (v, None)),
        ))(i)
    }

    fn arithmetic_cast_helper(
        i: &str,
    ) -> IResult<&str, (ArithmeticBase, Option<DataType>), ParseSQLError<&str>> {
        let (remaining_input, (_, _, _, _, a_base, _, _, _, _sign, sql_type, _, _)) = tuple((
            tag_no_case("CAST"),
            multispace0,
            tag("("),
            multispace0,
            // TODO(malte): should be arbitrary expr
            ArithmeticBase::parse,
            multispace1,
            tag_no_case("AS"),
            multispace1,
            opt(terminated(tag_no_case("SIGNED"), multispace1)),
            DataType::type_identifier,
            multispace0,
            tag(")"),
        ))(i)?;

        Ok((remaining_input, (a_base, Some(sql_type))))
    }
}

impl fmt::Display for ArithmeticItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArithmeticItem::Base(ref b) => write!(f, "{}", b),
            ArithmeticItem::Expr(ref expr) => write!(f, "{}", expr),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Arithmetic {
    pub op: ArithmeticOperator,
    pub left: ArithmeticItem,
    pub right: ArithmeticItem,
}

impl Arithmetic {
    fn parse(i: &str) -> IResult<&str, Arithmetic, ParseSQLError<&str>> {
        let res = ArithmeticItem::expr(i)?;
        match res.1 {
            ArithmeticItem::Base(ArithmeticBase::Column(_))
            | ArithmeticItem::Base(ArithmeticBase::Scalar(_)) => {
                let mut error: ParseSQLError<&str> = ParseSQLError { errors: vec![] };
                error.errors.push((i, ParseSQLErrorKind::Context("Tag")));
                Err(Error(error))
            } // no operator
            ArithmeticItem::Base(ArithmeticBase::Bracketed(expr)) => Ok((res.0, *expr)),
            ArithmeticItem::Expr(expr) => Ok((res.0, *expr)),
        }
    }
    pub fn new(op: ArithmeticOperator, left: ArithmeticBase, right: ArithmeticBase) -> Self {
        Self {
            op,
            left: ArithmeticItem::Base(left),
            right: ArithmeticItem::Base(right),
        }
    }
}

impl fmt::Display for Arithmetic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.op, self.right)
    }
}

#[derive(Debug, Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ArithmeticExpression {
    pub ari: Arithmetic,
    pub alias: Option<String>,
}

impl ArithmeticExpression {
    pub fn parse(i: &str) -> IResult<&str, ArithmeticExpression, ParseSQLError<&str>> {
        map(
            pair(Arithmetic::parse, opt(CommonParser::as_alias)),
            |(ari, opt_alias)| ArithmeticExpression {
                ari,
                alias: opt_alias.map(String::from),
            },
        )(i)
    }
}

impl ArithmeticExpression {
    pub fn new(
        op: ArithmeticOperator,
        left: ArithmeticBase,
        right: ArithmeticBase,
        alias: Option<String>,
    ) -> Self {
        Self {
            ari: Arithmetic {
                op,
                left: ArithmeticItem::Base(left),
                right: ArithmeticItem::Base(right),
            },
            alias,
        }
    }
}

impl fmt::Display for ArithmeticExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.alias {
            Some(ref alias) => write!(f, "{} AS {}", self.ari, alias),
            None => write!(f, "{}", self.ari),
        }
    }
}

#[cfg(test)]
mod tests {
    use base::arithmetic::ArithmeticBase::Scalar;
    use base::arithmetic::ArithmeticOperator::{Add, Divide, Multiply, Subtract};
    use base::column::{Column, FunctionArgument, FunctionExpression};

    use super::*;

    #[test]
    fn parses_arithmetic_expressions() {
        use super::{
            ArithmeticBase::{Column as ArithmeticBaseColumn, Scalar},
            ArithmeticOperator::*,
        };

        let lit_ae = [
            "5 + 42",
            "5+42",
            "5 * 42",
            "5 - 42",
            "5 / 42",
            "2 * 10 AS twenty ",
        ];

        // N.B. trailing space in "5 + foo " is required because `sql_identifier`'s keyword
        // detection requires a follow-up character (in practice, there always is one because we
        // use semicolon-terminated queries).
        let col_lit_ae = [
            "foo+5",
            "foo + 5",
            "5 + foo ",
            "foo * bar AS foobar",
            "MAX(foo)-3333",
        ];

        let expected_lit_ae = [
            ArithmeticExpression::new(Add, Scalar(5.into()), Scalar(42.into()), None),
            ArithmeticExpression::new(Add, Scalar(5.into()), Scalar(42.into()), None),
            ArithmeticExpression::new(Multiply, Scalar(5.into()), Scalar(42.into()), None),
            ArithmeticExpression::new(Subtract, Scalar(5.into()), Scalar(42.into()), None),
            ArithmeticExpression::new(Divide, Scalar(5.into()), Scalar(42.into()), None),
            ArithmeticExpression::new(
                Multiply,
                Scalar(2.into()),
                Scalar(10.into()),
                Some(String::from("twenty")),
            ),
        ];
        let expected_col_lit_ae = [
            ArithmeticExpression::new(
                Add,
                ArithmeticBaseColumn("foo".into()),
                Scalar(5.into()),
                None,
            ),
            ArithmeticExpression::new(
                Add,
                ArithmeticBaseColumn("foo".into()),
                Scalar(5.into()),
                None,
            ),
            ArithmeticExpression::new(
                Add,
                Scalar(5.into()),
                ArithmeticBaseColumn("foo".into()),
                None,
            ),
            ArithmeticExpression::new(
                Multiply,
                ArithmeticBaseColumn("foo".into()),
                ArithmeticBaseColumn("bar".into()),
                Some(String::from("foobar")),
            ),
            ArithmeticExpression::new(
                Subtract,
                ArithmeticBaseColumn(Column {
                    name: String::from("max(foo)"),
                    alias: None,
                    table: None,
                    function: Some(Box::new(FunctionExpression::Max(FunctionArgument::Column(
                        "foo".into(),
                    )))),
                }),
                Scalar(3333.into()),
                None,
            ),
        ];

        for (i, e) in lit_ae.iter().enumerate() {
            let res = ArithmeticExpression::parse(e);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, expected_lit_ae[i]);
        }

        for (i, e) in col_lit_ae.iter().enumerate() {
            let res = ArithmeticExpression::parse(e);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, expected_col_lit_ae[i]);
        }
    }

    #[test]
    fn displays_arithmetic_expressions() {
        use super::{
            ArithmeticBase::{Column as ArithmeticBaseColumn, Scalar},
            ArithmeticOperator::*,
        };

        let expressions = [
            ArithmeticExpression::new(
                Add,
                ArithmeticBaseColumn("foo".into()),
                Scalar(5.into()),
                None,
            ),
            ArithmeticExpression::new(
                Subtract,
                Scalar(5.into()),
                ArithmeticBaseColumn("foo".into()),
                None,
            ),
            ArithmeticExpression::new(
                Multiply,
                ArithmeticBaseColumn("foo".into()),
                ArithmeticBaseColumn("bar".into()),
                None,
            ),
            ArithmeticExpression::new(Divide, Scalar(10.into()), Scalar(2.into()), None),
            ArithmeticExpression::new(
                Add,
                Scalar(10.into()),
                Scalar(2.into()),
                Some(String::from("bob")),
            ),
        ];

        let expected_strings = ["foo + 5", "5 - foo", "foo * bar", "10 / 2", "10 + 2 AS bob"];
        for (i, e) in expressions.iter().enumerate() {
            assert_eq!(expected_strings[i], format!("{}", e));
        }
    }

    #[test]
    fn parses_arithmetic_casts() {
        use super::{
            ArithmeticBase::{Column as ArithmeticBaseColumn, Scalar},
            ArithmeticOperator::*,
        };

        let exprs = [
            "CAST(`t`.`foo` AS signed int) + CAST(`t`.`bar` AS signed int) ",
            "CAST(5 AS bigint) - foo ",
            "CAST(5 AS bigint) - foo AS `5_minus_foo`",
        ];

        // XXX(malte): currently discards the cast and type information!
        let expected = [
            ArithmeticExpression::new(
                Add,
                ArithmeticBaseColumn(Column::from("t.foo")),
                ArithmeticBaseColumn(Column::from("t.bar")),
                None,
            ),
            ArithmeticExpression::new(
                Subtract,
                Scalar(5.into()),
                ArithmeticBaseColumn("foo".into()),
                None,
            ),
            ArithmeticExpression::new(
                Subtract,
                Scalar(5.into()),
                ArithmeticBaseColumn("foo".into()),
                Some("5_minus_foo".into()),
            ),
        ];

        for (i, e) in exprs.iter().enumerate() {
            let res = ArithmeticExpression::parse(e);
            assert!(res.is_ok(), "{} failed to parse", e);
            assert_eq!(res.unwrap().1, expected[i]);
        }
    }

    #[test]
    fn parse_nested_arithmetic() {
        let qs = [
            "1 + 1",
            "1 + 2 - 3",
            "1 + 2 * 3",
            "2 * 3 - 1 / 3",
            "3 * (1 + 2)",
        ];

        let expects =
            [
                Arithmetic::new(Add, Scalar(1.into()), Scalar(1.into())),
                Arithmetic {
                    op: Subtract,
                    left: ArithmeticItem::Expr(Box::new(Arithmetic::new(
                        Add,
                        Scalar(1.into()),
                        Scalar(2.into()),
                    ))),
                    right: ArithmeticItem::Base(Scalar(3.into())),
                },
                Arithmetic {
                    op: Add,
                    left: ArithmeticItem::Base(Scalar(1.into())),
                    right: ArithmeticItem::Expr(Box::new(Arithmetic::new(
                        Multiply,
                        Scalar(2.into()),
                        Scalar(3.into()),
                    ))),
                },
                Arithmetic {
                    op: Subtract,
                    left: ArithmeticItem::Expr(Box::new(Arithmetic::new(
                        Multiply,
                        Scalar(2.into()),
                        Scalar(3.into()),
                    ))),
                    right: ArithmeticItem::Expr(Box::new(Arithmetic::new(
                        Divide,
                        Scalar(1.into()),
                        Scalar(3.into()),
                    ))),
                },
                Arithmetic {
                    op: Multiply,
                    left: ArithmeticItem::Base(Scalar(3.into())),
                    right: ArithmeticItem::Base(ArithmeticBase::Bracketed(Box::new(
                        Arithmetic::new(Add, Scalar(1.into()), Scalar(2.into())),
                    ))),
                },
            ];

        for (i, e) in qs.iter().enumerate() {
            let res = Arithmetic::parse(e);
            let ari = res.unwrap().1;
            assert_eq!(ari, expects[i]);
            assert_eq!(format!("{}", ari), qs[i]);
        }
    }

    #[test]
    fn parse_arithmetic_scalar() {
        let qs = "56";
        let res = Arithmetic::parse(qs);
        assert!(res.is_err());
    }
}
