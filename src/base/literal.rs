use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case, take};
use nom::character::complete::{digit1, multispace0};
use nom::combinator::{map, opt};
use nom::multi::{fold_many0, many0};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::{CommonParser, ItemPlaceholder};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Bool(bool),
    Null,
    Integer(i64),
    UnsignedInteger(u64),
    FixedPoint(Real),
    String(String),
    Blob(Vec<u8>),
    CurrentTime,
    CurrentDate,
    CurrentTimestamp,
    Placeholder(ItemPlaceholder),
}

impl Literal {
    // Integer literal value
    pub fn integer_literal(i: &str) -> IResult<&str, Literal, ParseSQLError<&str>> {
        map(pair(opt(tag("-")), digit1), |tup| {
            let mut intval = i64::from_str(tup.1).unwrap();
            if (tup.0).is_some() {
                intval *= -1;
            }
            Literal::Integer(intval)
        })(i)
    }

    fn unpack(v: &str) -> i32 {
        i32::from_str(v).unwrap()
    }

    // Floating point literal value
    pub fn float_literal(i: &str) -> IResult<&str, Literal, ParseSQLError<&str>> {
        map(tuple((opt(tag("-")), digit1, tag("."), digit1)), |tup| {
            Literal::FixedPoint(Real {
                integral: if (tup.0).is_some() {
                    -Self::unpack(tup.1)
                } else {
                    Self::unpack(tup.1)
                },
                fractional: Self::unpack(tup.3),
            })
        })(i)
    }

    /// String literal value
    fn raw_string_quoted(
        input: &str,
        is_single_quote: bool,
    ) -> IResult<&str, String, ParseSQLError<&str>> {
        // Adjusted to work with &str
        let quote_char = if is_single_quote { '\'' } else { '"' };
        let quote_str = if is_single_quote { "\'" } else { "\"" };
        let double_quote_str = if is_single_quote { "\'\'" } else { "\"\"" };
        let backslash_quote = if is_single_quote { "\\\'" } else { "\\\"" };

        delimited(
            tag(quote_str),
            fold_many0(
                alt((
                    is_not(backslash_quote),
                    map(tag(double_quote_str), |_| {
                        if is_single_quote {
                            "\'"
                        } else {
                            "\""
                        }
                    }),
                    map(tag("\\\\"), |_| "\\"),
                    map(tag("\\b"), |_| "\x7F"), // 注意：\x7f 是 DEL，\x08 是退格
                    map(tag("\\r"), |_| "\r"),
                    map(tag("\\n"), |_| "\n"),
                    map(tag("\\t"), |_| "\t"),
                    map(tag("\\0"), |_| "\0"),
                    map(tag("\\Z"), |_| "\x1A"),
                    preceded(tag("\\"), take(1usize)),
                )),
                String::new,
                |mut acc: String, bytes: &str| {
                    acc.push_str(bytes);
                    acc
                },
            ),
            tag(quote_str),
        )(input)
    }

    fn raw_string_single_quoted(i: &str) -> IResult<&str, String, ParseSQLError<&str>> {
        Self::raw_string_quoted(i, true)
    }

    fn raw_string_double_quoted(i: &str) -> IResult<&str, String, ParseSQLError<&str>> {
        Self::raw_string_quoted(i, false)
    }

    pub fn string_literal(i: &str) -> IResult<&str, Literal, ParseSQLError<&str>> {
        map(
            alt((
                Self::raw_string_single_quoted,
                Self::raw_string_double_quoted,
            )),
            Literal::String,
        )(i)
    }

    // Any literal value.
    pub fn parse(i: &str) -> IResult<&str, Literal, ParseSQLError<&str>> {
        alt((
            Self::float_literal,
            Self::integer_literal,
            Self::string_literal,
            map(tag_no_case("NULL"), |_| Literal::Null),
            map(tag_no_case("CURRENT_TIMESTAMP"), |_| {
                Literal::CurrentTimestamp
            }),
            map(tag_no_case("CURRENT_DATE"), |_| Literal::CurrentDate),
            map(tag_no_case("CURRENT_TIME"), |_| Literal::CurrentTime),
            map(tag("?"), |_| {
                Literal::Placeholder(ItemPlaceholder::QuestionMark)
            }),
            map(preceded(tag(":"), digit1), |num| {
                let value = i32::from_str(num).unwrap();
                Literal::Placeholder(ItemPlaceholder::ColonNumber(value))
            }),
            map(preceded(tag("$"), digit1), |num| {
                let value = i32::from_str(num).unwrap();
                Literal::Placeholder(ItemPlaceholder::DollarNumber(value))
            }),
        ))(i)
    }

    // Parse a list of values (e.g., for INSERT syntax).
    pub fn value_list(i: &str) -> IResult<&str, Vec<Literal>, ParseSQLError<&str>> {
        many0(delimited(
            multispace0,
            Literal::parse,
            opt(CommonParser::ws_sep_comma),
        ))(i)
    }
}

impl From<i64> for Literal {
    fn from(i: i64) -> Self {
        Literal::Integer(i)
    }
}

impl From<u64> for Literal {
    fn from(i: u64) -> Self {
        Literal::UnsignedInteger(i)
    }
}

impl From<i32> for Literal {
    fn from(i: i32) -> Self {
        Literal::Integer(i.into())
    }
}

impl From<u32> for Literal {
    fn from(i: u32) -> Self {
        Literal::UnsignedInteger(i.into())
    }
}

impl From<String> for Literal {
    fn from(s: String) -> Self {
        Literal::String(s)
    }
}

impl<'a> From<&'a str> for Literal {
    fn from(s: &'a str) -> Self {
        Literal::String(String::from(s))
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Literal::Null => write!(f, "NULL"),
            Literal::Bool(ref value) => {
                if *value {
                    write!(f, "TRUE")
                } else {
                    write!(f, "FALSE")
                }
            }
            Literal::Integer(ref i) => write!(f, "{}", i),
            Literal::UnsignedInteger(ref i) => write!(f, "{}", i),
            Literal::FixedPoint(ref fp) => write!(f, "{}.{}", fp.integral, fp.fractional),
            Literal::String(ref s) => write!(f, "'{}'", s.replace('\'', "''")),
            Literal::Blob(ref bv) => {
                let val = bv
                    .iter()
                    .map(|v| format!("{:x}", v))
                    .collect::<Vec<String>>()
                    .join(" ")
                    .to_string();
                write!(f, "{}", val)
            }
            Literal::CurrentTime => write!(f, "CURRENT_TIME"),
            Literal::CurrentDate => write!(f, "CURRENT_DATE"),
            Literal::CurrentTimestamp => write!(f, "CURRENT_TIMESTAMP"),
            Literal::Placeholder(ref item) => write!(f, "{}", item),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LiteralExpression {
    pub value: Literal,
    pub alias: Option<String>,
}

impl LiteralExpression {
    pub fn parse(i: &str) -> IResult<&str, LiteralExpression, ParseSQLError<&str>> {
        map(
            pair(
                CommonParser::opt_delimited(tag("("), Literal::parse, tag(")")),
                opt(CommonParser::as_alias),
            ),
            |p| LiteralExpression {
                value: p.0,
                alias: (p.1).map(|a| a.to_string()),
            },
        )(i)
    }
}

impl From<Literal> for LiteralExpression {
    fn from(l: Literal) -> Self {
        LiteralExpression {
            value: l,
            alias: None,
        }
    }
}

impl fmt::Display for LiteralExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.alias {
            Some(ref alias) => write!(f, "{} AS {}", self.value, alias),
            None => write!(f, "{}", self.value),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Real {
    pub integral: i32,
    pub fractional: i32,
}

impl Display for Real {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.integral, self.fractional)
    }
}

#[cfg(test)]
mod tests {
    use base::Literal;

    #[test]
    #[allow(clippy::redundant_slicing)]
    fn literal_string_single_backslash_escape() {
        let all_escaped = r#"\0\'\"\b\n\r\t\Z\\\%\_"#;
        for quote in ["'", "\""].iter() {
            let quoted = &[quote, &all_escaped[..], quote].concat();
            let res = Literal::string_literal(quoted);
            let expected = Literal::String("\0\'\"\x7F\n\r\t\x1a\\%_".to_string());

            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, expected);
        }
    }

    #[test]
    fn literal_string_single_quote() {
        let res = Literal::string_literal("'a''b'");
        let expected = Literal::String("a'b".to_string());

        assert!(res.is_ok());
        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn literal_string_double_quote() {
        let res = Literal::string_literal(r#""a""b""#);
        let expected = Literal::String(r#"a"b"#.to_string());

        assert!(res.is_ok());
        assert_eq!(res.unwrap().1, expected);
    }
}
