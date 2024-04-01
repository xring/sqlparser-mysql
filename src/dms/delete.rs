use std::{fmt, str};

use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::opt;
use nom::error::VerboseError;
use nom::IResult;
use nom::sequence::{delimited, tuple};

use common::keywords::escape_if_keyword;
use base::table::Table;
use common::statement_terminator;
use dms::condition::ConditionExpression;
use dms::select::where_clause;

/// DELETE [LOW_PRIORITY] [QUICK] [IGNORE] FROM tbl_name [[AS] tbl_alias]
///     [PARTITION (partition_name [, partition_name] ...)]
///     [WHERE where_condition]
///     [ORDER BY ...]
///     [LIMIT row_count]
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

pub fn deletion(i: &str) -> IResult<&str, DeleteStatement, VerboseError<&str>> {
    let (remaining_input, (_, _, table, where_clause, _)) = tuple((
        tag_no_case("DELETE"),
        delimited(multispace1, tag_no_case("FROM"), multispace1),
        Table::schema_table_reference,
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
    use base::column::Column;
    use base::Literal;
    use base::Operator;
    use dms::condition::ConditionBase::*;
    use dms::condition::ConditionExpression::*;
    use dms::condition::ConditionExpression::ComparisonOp;
    use dms::condition::ConditionTree;

    use super::*;

    #[test]
    fn simple_delete() {
        let str = "DELETE FROM users;";
        let res = deletion(str);
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
        let str = "DELETE FROM db1.users;";
        let res = deletion(str);
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
        let res = deletion(str);

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
        let res = deletion(str);
        assert_eq!(format!("{}", res.unwrap().1), expected);
    }
}
