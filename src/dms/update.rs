use std::{fmt, str};

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::opt;
use nom::sequence::tuple;
use nom::IResult;

use base::column::Column;
use base::condition::ConditionExpression;
use base::error::ParseSQLError;
use base::table::Table;
use base::{CommonParser, DisplayUtil, FieldValueExpression};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UpdateStatement {
    pub table: Table,
    pub fields: Vec<(Column, FieldValueExpression)>,
    pub where_clause: Option<ConditionExpression>,
}

impl UpdateStatement {
    pub fn parse(i: &str) -> IResult<&str, UpdateStatement, ParseSQLError<&str>> {
        let (remaining_input, (_, _, table, _, _, _, fields, _, where_clause, _)) = tuple((
            tag_no_case("UPDATE"),
            multispace1,
            Table::table_reference,
            multispace1,
            tag_no_case("SET"),
            multispace1,
            FieldValueExpression::assignment_expr_list,
            multispace0,
            opt(ConditionExpression::parse),
            CommonParser::statement_terminator,
        ))(i)?;
        Ok((
            remaining_input,
            UpdateStatement {
                table,
                fields,
                where_clause,
            },
        ))
    }
}

impl fmt::Display for UpdateStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "UPDATE {} ",
            DisplayUtil::escape_if_keyword(&self.table.name)
        )?;
        assert!(!self.fields.is_empty());
        write!(
            f,
            "SET {}",
            self.fields
                .iter()
                .map(|(col, literal)| format!("{} = {}", col, literal))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if let Some(ref where_clause) = self.where_clause {
            write!(f, " WHERE ")?;
            write!(f, "{}", where_clause)?;
        }
        Ok(())
    }
}
