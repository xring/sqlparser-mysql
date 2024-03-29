use nom::character::complete::multispace1;
use std::{fmt, str};

use common::table::Table;
use common_parsers::{schema_table_reference, statement_terminator};
use zz_condition::ConditionExpression;
use keywords::escape_if_keyword;
use nom::bytes::complete::tag_no_case;
use nom::combinator::opt;
use nom::sequence::{delimited, tuple};
use nom::IResult;
use zz_select::where_clause;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DeleteStatement {
    pub table: Table,
    pub where_clause: Option<ConditionExpression>,
}

impl fmt::Display for DeleteStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DELETE FROM ")?;
        write!(f, "{}", escape_if_keyword(&self.table.name))?;
        if let Some(ref where_clause) = self.where_clause {
            write!(f, " WHERE ")?;
            write!(f, "{}", where_clause)?;
        }
        Ok(())
    }
}

pub fn deletion(i: &[u8]) -> IResult<&[u8], DeleteStatement> {
    let (remaining_input, (_, _, table, where_clause, _)) = tuple((
        tag_no_case("delete"),
        delimited(multispace1, tag_no_case("from"), multispace1),
        schema_table_reference,
        opt(where_clause),
        statement_terminator,
    ))(i)?;

    Ok((
        remaining_input,
        DeleteStatement {
            table,
            where_clause,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::column::Column;
    use common::Operator;
    use common::Literal;
    use zz_condition::ConditionBase::*;
    use zz_condition::ConditionExpression::*;
    use zz_condition::ConditionTree;

    #[test]
    fn simple_delete() {
        let qstring = "DELETE FROM users;";
        let res = deletion(qstring.as_bytes());
        assert_eq!(
            res.unwrap().1,
            DeleteStatement {
                table: Table::from("users"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn simple_delete_schema() {
        let qstring = "DELETE FROM db1.users;";
        let res = deletion(qstring.as_bytes());
        assert_eq!(
            res.unwrap().1,
            DeleteStatement {
                table: Table::from(("db1", "users")),
                ..Default::default()
            }
        );
    }

    #[test]
    fn delete_with_where_clause() {
        let str = "DELETE FROM users WHERE id = 1;";
        let res = deletion(str.as_bytes());

        let expected_left = Base(Field(Column::from("id")));
        let expected_where_cond = Some(ComparisonOp(ConditionTree {
            left: Box::new(expected_left),
            right: Box::new(Base(Literal(Literal::Integer(1)))),
            operator: Operator::Equal,
        }));
        assert_eq!(
            res.unwrap().1,
            DeleteStatement {
                table: Table::from("users"),
                where_clause: expected_where_cond,
                ..Default::default()
            }
        );
    }

    #[test]
    fn format_delete() {
        let str = "DELETE FROM users WHERE id = 1";
        let expected = "DELETE FROM users WHERE id = 1";
        let res = deletion(str.as_bytes());
        assert_eq!(format!("{}", res.unwrap().1), expected);
    }
}