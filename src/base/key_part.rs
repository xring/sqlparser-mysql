use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, digit1, multispace0, multispace1};
use nom::combinator::{map, opt, recognize};
use nom::multi::many1;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::{CommonParser, OrderType};

/// parse `key_part: {col_name [(length)] | (expr)} [ASC | DESC]`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct KeyPart {
    pub r#type: KeyPartType,
    pub order: Option<OrderType>,
}

impl KeyPart {
    ///parse list of key_part `(key_part,...)`
    pub fn parse(i: &str) -> IResult<&str, Vec<KeyPart>, ParseSQLError<&str>> {
        map(
            tuple((
                multispace0,
                delimited(
                    tag("("),
                    delimited(
                        multispace0,
                        many1(map(
                            terminated(Self::parse_item, opt(CommonParser::ws_sep_comma)),
                            |e| e,
                        )),
                        multispace0,
                    ),
                    tag(")"),
                ),
            )),
            |(_, val)| val,
        )(i)
    }

    /// parse `key_part: {col_name [(length)] | (expr)} [ASC | DESC]`
    fn parse_item(i: &str) -> IResult<&str, KeyPart, ParseSQLError<&str>> {
        map(
            tuple((
                KeyPartType::parse,
                opt(map(
                    tuple((multispace1, OrderType::parse, multispace0)),
                    |(_, order, _)| order,
                )),
            )),
            |(r#type, order)| KeyPart { r#type, order },
        )(i)
    }
}

/// parse `{col_name [(length)] | (expr)}`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum KeyPartType {
    ColumnNameWithLength {
        col_name: String,
        length: Option<usize>,
    },
    Expr {
        expr: String,
    },
}

impl KeyPartType {
    fn parse(i: &str) -> IResult<&str, KeyPartType, ParseSQLError<&str>> {
        // {col_name [(length)]
        let col_name_with_length = tuple((
            CommonParser::sql_identifier,
            multispace0,
            opt(delimited(
                tag("("),
                map(digit1, |digit_str: &str| {
                    digit_str.parse::<usize>().unwrap()
                }),
                tag(")"),
            )),
        ));

        let expr = preceded(
            multispace0,
            delimited(tag("("), recognize(many1(anychar)), tag(")")),
        );

        alt((
            map(col_name_with_length, |(col_name, _, length)| {
                KeyPartType::ColumnNameWithLength {
                    col_name: String::from(col_name),
                    length,
                }
            }),
            map(expr, |expr| KeyPartType::Expr {
                expr: String::from(expr),
            }),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use base::{KeyPart, KeyPartType};

    #[test]
    fn parse_key_part_type() {
        let str1 = "column_name(10)";
        let res1 = KeyPartType::parse(str1);
        let exp = KeyPartType::ColumnNameWithLength {
            col_name: "column_name".to_string(),
            length: Some(10),
        };
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, exp);
    }

    #[test]
    fn parse_key_part() {
        let str1 = "(column_name(10))";
        let res1 = KeyPart::parse(str1);

        let key_part = KeyPartType::ColumnNameWithLength {
            col_name: "column_name".to_string(),
            length: Some(10),
        };
        let exp = vec![KeyPart {
            r#type: key_part,
            order: None,
        }];
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, exp);
    }
}
