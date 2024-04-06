use std::fmt;
use std::str;

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{preceded, tuple};
use nom::IResult;

use base::column::Column;
use base::error::ParseSQLError;
use base::keywords::escape_if_keyword;
use base::CommonParser;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct OrderClause {
    pub columns: Vec<(Column, OrderType)>, // TODO(malte): can this be an arbitrary expr?
}

impl OrderClause {
    // Parse ORDER BY clause
    pub fn parse(i: &str) -> IResult<&str, OrderClause, ParseSQLError<&str>> {
        let (remaining_input, (_, _, _, _, _, columns)) = tuple((
            multispace0,
            tag_no_case("ORDER"),
            multispace1,
            tag_no_case("BY"),
            multispace1,
            many0(Self::order_expr),
        ))(i)?;

        Ok((remaining_input, OrderClause { columns }))
    }

    fn order_expr(i: &str) -> IResult<&str, (Column, OrderType), ParseSQLError<&str>> {
        let (remaining_input, (field_name, ordering, _)) = tuple((
            Column::without_alias,
            opt(preceded(multispace0, OrderType::parse)),
            opt(CommonParser::ws_sep_comma),
        ))(i)?;

        Ok((
            remaining_input,
            (field_name, ordering.unwrap_or(OrderType::Asc)),
        ))
    }
}

impl fmt::Display for OrderClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ORDER BY ")?;
        write!(
            f,
            "{}",
            self.columns
                .iter()
                .map(|(c, o)| format!("{} {}", escape_if_keyword(&c.name), o))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// [ASC | DESC]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Asc,
    Desc,
}

impl OrderType {
    pub fn parse(i: &str) -> IResult<&str, OrderType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("DESC"), |_| OrderType::Desc),
            map(tag_no_case("ASC"), |_| OrderType::Asc),
        ))(i)
    }
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            OrderType::Asc => write!(f, "ASC"),
            OrderType::Desc => write!(f, "DESC"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_clause() {
        let str1 = "order by name desc";
        let str2 = "order by name asc, age desc";
        let str3 = "order by name";

        let expected_ord1 = OrderClause {
            columns: vec![("name".into(), OrderType::Desc)],
        };
        let expected_ord2 = OrderClause {
            columns: vec![
                ("name".into(), OrderType::Asc),
                ("age".into(), OrderType::Desc),
            ],
        };
        let expected_ord3 = OrderClause {
            columns: vec![("name".into(), OrderType::Asc)],
        };

        let res1 = OrderClause::parse(str1);
        let res2 = OrderClause::parse(str2);
        let res3 = OrderClause::parse(str3);
        assert_eq!(res1.unwrap().1, expected_ord1);
        assert_eq!(res2.unwrap().1, expected_ord2);
        assert_eq!(res3.unwrap().1, expected_ord3);
    }
}
