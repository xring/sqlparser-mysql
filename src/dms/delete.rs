use std::{fmt, str};

use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::opt;
use nom::sequence::{delimited, tuple};
use nom::IResult;

use base::condition::ConditionExpression;
use base::error::ParseSQLError;
use base::table::Table;
use base::{CommonParser, DisplayUtil};

// FIXME TODO
/// `DELETE [LOW_PRIORITY] [QUICK] [IGNORE] FROM tbl_name [[AS] tbl_alias]
///     [PARTITION (partition_name [, partition_name] ...)]
///     [WHERE where_condition]
///     [ORDER BY ...]
///     [LIMIT row_count]`
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DeleteStatement {
    pub table: Table,
    pub where_clause: Option<ConditionExpression>,
}

impl DeleteStatement {
    pub fn parse(i: &str) -> IResult<&str, DeleteStatement, ParseSQLError<&str>> {
        let (remaining_input, (_, _, table, where_clause, _)) = tuple((
            tag_no_case("DELETE"),
            delimited(multispace1, tag_no_case("FROM"), multispace1),
            Table::schema_table_reference,
            opt(ConditionExpression::parse),
            CommonParser::statement_terminator,
        ))(i)?;

        Ok((
            remaining_input,
            DeleteStatement {
                table,
                where_clause,
            },
        ))
    }
}

impl fmt::Display for DeleteStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DELETE FROM ")?;
        write!(f, "{}", DisplayUtil::escape_if_keyword(&self.table.name))?;
        if let Some(ref where_clause) = self.where_clause {
            write!(f, " WHERE ")?;
            write!(f, "{}", where_clause)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::column::Column;
    use base::condition::ConditionBase::Field;
    use base::condition::ConditionExpression::{Base, ComparisonOp};
    use base::condition::{ConditionBase, ConditionTree};
    use base::Literal;
    use base::Operator;

    use super::*;

    #[test]
    fn simple_delete() {
        let str = "DELETE FROM users;";
        let res = DeleteStatement::parse(str);
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
        let res = DeleteStatement::parse(str);
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
        let res = DeleteStatement::parse(str);

        let expected_left = Base(Field(Column::from("id")));
        let expected_where_cond = Some(ComparisonOp(ConditionTree {
            left: Box::new(expected_left),
            right: Box::new(Base(ConditionBase::Literal(Literal::Integer(1)))),
            operator: Operator::Equal,
        }));
        assert_eq!(
            res.unwrap().1,
            DeleteStatement {
                table: Table::from("users"),
                where_clause: expected_where_cond,
            }
        );
    }

    #[test]
    fn format_delete() {
        let str = "DELETE FROM users WHERE id = 1";
        let expected = "DELETE FROM users WHERE id = 1";
        let res = DeleteStatement::parse(str);
        assert_eq!(format!("{}", res.unwrap().1), expected);
    }
}
