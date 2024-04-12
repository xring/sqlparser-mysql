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
