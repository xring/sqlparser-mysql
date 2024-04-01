use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, digit1, multispace0, multispace1};
use nom::combinator::{map, map_res, opt, recognize};
use nom::error::VerboseError;
use nom::multi::many1;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

use common::OrderType;
use common::{sql_identifier, ws_sep_comma};

/// key_part: {col_name [(length)] | (expr)} [ASC | DESC]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct KeyPart {
    r#type: KeyPartType,
    order: Option<OrderType>,
}

impl KeyPart {
    /// key_part: {col_name [(length)] | (expr)} [ASC | DESC]
    fn parse(i: &str) -> IResult<&str, KeyPart, VerboseError<&str>> {
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

    /// (key_part,...)
    /// key_part: {col_name [(length)] | (expr)} [ASC | DESC]
    pub fn key_part_list(i: &str) -> IResult<&str, Vec<KeyPart>, VerboseError<&str>> {
        map(
            tuple((
                multispace0,
                delimited(
                    tag("("),
                    delimited(
                        multispace0,
                        many1(map(terminated(Self::parse, opt(ws_sep_comma)), |e| e)),
                        multispace0,
                    ),
                    tag(")"),
                ),
            )),
            |(_, val)| val,
        )(i)
    }
}

/// {col_name [(length)] | (expr)}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum KeyPartType {
    ColumnNameWithLength(String, Option<usize>),
    Expr(String),
}

impl KeyPartType {
    /// {col_name [(length)] | (expr)}
    fn parse(i: &str) -> IResult<&str, KeyPartType, VerboseError<&str>> {
        // {col_name [(length)]
        let col_name_with_length = tuple((
            multispace0,
            sql_identifier,
            multispace0,
            opt(delimited(
                tag("("),
                map_res(digit1, |digit_str: &str| digit_str.parse::<usize>()),
                tag(")"),
            )),
        ));

        let expr = preceded(
            multispace1,
            delimited(tag("("), recognize(many1(anychar)), tag(")")),
        );

        alt((
            map(col_name_with_length, |(_, col_name, _, length)| {
                KeyPartType::ColumnNameWithLength(String::from(col_name), length)
            }),
            map(expr, |expr| KeyPartType::Expr(String::from(expr))),
        ))(i)
    }
}
