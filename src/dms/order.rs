use std::fmt;
use std::str;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::opt;
use nom::error::VerboseError;
use nom::IResult;
use nom::multi::many0;
use nom::sequence::{preceded, tuple};

use base::column::Column;
use common::keywords::escape_if_keyword;
use common::ws_sep_comma;
use common::OrderType;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct OrderClause {
    pub columns: Vec<(Column, OrderType)>, // TODO(malte): can this be an arbitrary expr?
}

impl fmt::Display for OrderClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ORDER BY ")?;
        write!(
            f,
            "{}",
            self.columns
                .iter()
                .map(|&(ref c, ref o)| format!("{} {}", escape_if_keyword(&c.name), o))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn order_expr(i: &str) -> IResult<&str, (Column, OrderType), VerboseError<&str>> {
    let (remaining_input, (field_name, ordering, _)) = tuple((
        Column::without_alias,
        opt(preceded(multispace0, OrderType::parse)),
        opt(ws_sep_comma),
    ))(i)?;

    Ok((
        remaining_input,
        (field_name, ordering.unwrap_or(OrderType::Asc)),
    ))
}

// Parse ORDER BY clause
pub fn order_clause(i: &str) -> IResult<&str, OrderClause, VerboseError<&str>> {
    let (remaining_input, (_, _, _, columns)) = tuple((
        multispace0,
        tag_no_case("order by"),
        multispace1,
        many0(order_expr),
    ))(i)?;

    Ok((remaining_input, OrderClause { columns }))
}

#[cfg(test)]
mod tests {
    use common::OrderType;
    use dms::select::selection;

    use super::*;

    #[test]
    fn order_clause() {
        let str1 = "select * from users order by name desc\n";
        let str2 = "select * from users order by name asc, age desc\n";
        let str3 = "select * from users order by name\n";

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

        let res1 = selection(str1);
        let res2 = selection(str2);
        let res3 = selection(str3);
        assert_eq!(res1.unwrap().1.order, Some(expected_ord1));
        assert_eq!(res2.unwrap().1.order, Some(expected_ord2));
        assert_eq!(res3.unwrap().1.order, Some(expected_ord3));
    }
}
