use std::fmt;
use std::str;

use nom::bytes::complete::{tag_no_case, take_till, take_until};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

use base::column::Column;
use base::condition::ConditionExpression;
use base::error::ParseSQLError;
use base::table::Table;
use base::{
    CommonParser, FieldDefinitionExpression, JoinClause, JoinConstraint, JoinOperator,
    JoinRightSide, OrderClause,
};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SelectStatement {
    pub tables: Vec<Table>,
    pub distinct: bool,
    pub fields: Vec<FieldDefinitionExpression>,
    pub join: Vec<JoinClause>,
    pub where_clause: Option<ConditionExpression>,
    pub group_by: Option<GroupByClause>,
    pub order: Option<OrderClause>,
    pub limit: Option<LimitClause>,
}

impl SelectStatement {
    // Parse rule for a SQL selection query.
    pub fn parse(i: &str) -> IResult<&str, SelectStatement, ParseSQLError<&str>> {
        terminated(Self::nested_selection, CommonParser::statement_terminator)(i)
    }

    pub fn nested_selection(i: &str) -> IResult<&str, SelectStatement, ParseSQLError<&str>> {
        let (
            remaining_input,
            (_, _, distinct, _, fields, _, tables, join, where_clause, group_by, order, limit),
        ) = tuple((
            tag_no_case("SELECT"),
            multispace1,
            opt(tag_no_case("DISTINCT")),
            multispace0,
            FieldDefinitionExpression::parse,
            delimited(multispace0, tag_no_case("FROM"), multispace0),
            Table::table_list,
            many0(JoinClause::parse),
            opt(ConditionExpression::parse),
            opt(GroupByClause::parse),
            opt(OrderClause::parse),
            opt(LimitClause::parse),
        ))(i)?;
        Ok((
            remaining_input,
            SelectStatement {
                tables,
                distinct: distinct.is_some(),
                fields,
                join,
                where_clause,
                group_by,
                order,
                limit,
            },
        ))
    }
}

impl fmt::Display for SelectStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SELECT ")?;
        if self.distinct {
            write!(f, "DISTINCT ")?;
        }
        write!(
            f,
            "{}",
            self.fields
                .iter()
                .map(|field| format!("{}", field))
                .collect::<Vec<_>>()
                .join(", ")
        )?;

        if !self.tables.is_empty() {
            write!(f, " FROM ")?;
            write!(
                f,
                "{}",
                self.tables
                    .iter()
                    .map(|table| format!("{}", table))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        for jc in &self.join {
            write!(f, " {}", jc)?;
        }
        if let Some(ref where_clause) = self.where_clause {
            write!(f, " WHERE ")?;
            write!(f, "{}", where_clause)?;
        }
        if let Some(ref group_by) = self.group_by {
            write!(f, " {}", group_by)?;
        }
        if let Some(ref order) = self.order {
            write!(f, " {}", order)?;
        }
        if let Some(ref limit) = self.limit {
            write!(f, " {}", limit)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GroupByClause {
    pub columns: Vec<Column>,
    pub having: Option<ConditionExpression>,
}

impl GroupByClause {
    // Parse GROUP BY clause
    pub fn parse(i: &str) -> IResult<&str, GroupByClause, ParseSQLError<&str>> {
        let (remaining_input, (_, _, _, columns, having)) = tuple((
            multispace0,
            tag_no_case("GROUP BY"),
            multispace1,
            Column::field_list,
            opt(ConditionExpression::having_clause),
        ))(i)?;

        Ok((remaining_input, GroupByClause { columns, having }))
    }
}

impl fmt::Display for GroupByClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GROUP BY ")?;
        write!(
            f,
            "{}",
            self.columns
                .iter()
                .map(|c| format!("{}", c))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if let Some(ref having) = self.having {
            write!(f, " HAVING {}", having)?;
        }
        Ok(())
    }
}

// TODO need parse as detailed data type
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct BetweenAndClause {
    pub left: String,
    pub right: String,
}

impl BetweenAndClause {
    pub fn parse(i: &str) -> IResult<&str, BetweenAndClause, ParseSQLError<&str>> {
        map(
            tuple((
                CommonParser::sql_identifier,
                multispace1,
                tag_no_case("BETWEEN"),
                multispace1,
                take_until(" "),
                multispace1,
                tag_no_case("AND"),
                multispace1,
                take_till(|c| c == ' '),
            )),
            |x| BetweenAndClause {
                left: String::from(x.4),
                right: String::from(x.8),
            },
        )(i)
    }
}

impl fmt::Display for BetweenAndClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " BETWEEN {}", self.left)?;
        write!(f, " AND {}", self.right)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct LimitClause {
    pub limit: u64,
    pub offset: u64,
}

impl LimitClause {
    pub fn parse(i: &str) -> IResult<&str, LimitClause, ParseSQLError<&str>> {
        let (remaining_input, (_, _, _, limit, opt_offset)) = tuple((
            multispace0,
            tag_no_case("LIMIT"),
            multispace1,
            CommonParser::unsigned_number,
            opt(Self::offset),
        ))(i)?;
        let offset = opt_offset.unwrap_or(0);

        Ok((remaining_input, LimitClause { limit, offset }))
    }

    fn offset(i: &str) -> IResult<&str, u64, ParseSQLError<&str>> {
        let (remaining_input, (_, _, _, val)) = tuple((
            multispace0,
            tag_no_case("OFFSET"),
            multispace1,
            CommonParser::unsigned_number,
        ))(i)?;

        Ok((remaining_input, val))
    }
}

impl fmt::Display for LimitClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LIMIT {}", self.limit)?;
        if self.offset > 0 {
            write!(f, " OFFSET {}", self.offset)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::arithmetic::{ArithmeticBase, ArithmeticExpression, ArithmeticOperator};
    use base::column::{Column, FunctionArgument, FunctionArguments, FunctionExpression};
    use base::condition::ConditionBase::LiteralList;
    use base::condition::ConditionExpression::{Base, ComparisonOp, LogicalOp};
    use base::condition::{ConditionBase, ConditionExpression, ConditionTree};
    use base::table::Table;
    use base::{
        CaseWhenExpression, ColumnOrLiteral, FieldValueExpression, ItemPlaceholder, JoinClause,
        JoinConstraint, JoinOperator, JoinRightSide, Operator, OrderClause,
    };
    use base::{Literal, OrderType};

    use super::*;


}
