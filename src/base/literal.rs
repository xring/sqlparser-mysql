use std::fmt;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case, take};
use nom::character::complete::{digit1, multispace0};
use nom::combinator::{map, opt};
use nom::multi::{fold_many0, many0};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::ItemPlaceholder;
use common::{as_alias, opt_delimited, ws_sep_comma};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Literal {
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
                    -1 * Self::unpack(tup.1)
                } else {
                    Self::unpack(tup.1)
                },
                fractional: Self::unpack(tup.3) as i32,
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
                    map(tag("\\b"), |_| "\x08"), // 注意：\x7f 是 DEL，\x08 是退格
                    map(tag("\\r"), |_| "\r"),
                    map(tag("\\n"), |_| "\n"),
                    map(tag("\\t"), |_| "\t"),
                    map(tag("\\0"), |_| "\0"),
                    map(tag("\\Z"), |_| "\x1A"),
                    preceded(tag("\\"), take(1usize)),
                )),
                || String::new(),
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
            |str| Literal::String(str),
        )(i)
    }

    // Any literal value.
    pub fn parse(i: &str) -> IResult<&str, Literal, ParseSQLError<&str>> {
        alt((
            Self::float_literal,
            Self::integer_literal,
            Self::string_literal,
            map(tag_no_case("null"), |_| Literal::Null),
            map(tag_no_case("current_timestamp"), |_| {
                Literal::CurrentTimestamp
            }),
            map(tag_no_case("current_date"), |_| Literal::CurrentDate),
            map(tag_no_case("current_time"), |_| Literal::CurrentTime),
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
        many0(delimited(multispace0, Literal::parse, opt(ws_sep_comma)))(i)
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

impl ToString for Literal {
    fn to_string(&self) -> String {
        match *self {
            Literal::Null => "NULL".to_string(),
            Literal::Integer(ref i) => format!("{}", i),
            Literal::UnsignedInteger(ref i) => format!("{}", i),
            Literal::FixedPoint(ref f) => format!("{}.{}", f.integral, f.fractional),
            Literal::String(ref s) => format!("'{}'", s.replace('\'', "''")),
            Literal::Blob(ref bv) => format!(
                "{}",
                bv.iter()
                    .map(|v| format!("{:x}", v))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Literal::CurrentTime => "CURRENT_TIME".to_string(),
            Literal::CurrentDate => "CURRENT_DATE".to_string(),
            Literal::CurrentTimestamp => "CURRENT_TIMESTAMP".to_string(),
            Literal::Placeholder(ref item) => item.to_string(),
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
                opt_delimited(tag("("), Literal::parse, tag(")")),
                opt(as_alias),
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
            Some(ref alias) => write!(f, "{} AS {}", self.value.to_string(), alias),
            None => write!(f, "{}", self.value.to_string()),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Real {
    pub integral: i32,
    pub fractional: i32,
}

#[cfg(test)]
mod tests {
    use base::Literal;

    #[test]
    fn literal_string_single_backslash_escape() {
        let all_escaped = r#"\0\'\"\b\n\r\t\Z\\\%\_"#;
        for quote in [&"'"[..], &"\""[..]].iter() {
            let quoted = &[quote, &all_escaped[..], quote].concat();
            let res = Literal::string_literal(quoted);
            let expected = Literal::String("\0\'\"\x7F\n\r\t\x1a\\%_".to_string());
            assert_eq!(res, Ok(("", expected)));
        }
    }

    #[test]
    fn literal_string_single_quote() {
        let res = Literal::string_literal("'a''b'");
        let expected = Literal::String("a'b".to_string());
        assert_eq!(res, Ok(("", expected)));
    }

    #[test]
    fn literal_string_double_quote() {
        let res = Literal::string_literal(r#""a""b""#);
        let expected = Literal::String(r#"a"b"#.to_string());
        assert_eq!(res, Ok(("", expected)));
    }
}
