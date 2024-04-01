use std::fmt;
use std::str;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::opt;
use nom::error::VerboseError;
use nom::IResult;
use nom::multi::many0;
use nom::sequence::{delimited, terminated, tuple};

use base::column::Column;
use base::FieldDefinitionExpression;
use base::table::Table;
use common::{JoinConstraint, JoinOperator, JoinRightSide, OrderClause, statement_terminator, unsigned_number};
use common::condition::ConditionExpression;

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
    pub fn parse(i: &str) -> IResult<&str, SelectStatement, VerboseError<&str>> {
        terminated(Self::nested_selection, statement_terminator)(i)
    }

    pub fn nested_selection(i: &str) -> IResult<&str, SelectStatement, VerboseError<&str>> {
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

        if self.tables.len() > 0 {
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
    pub fn parse(i: &str) -> IResult<&str, GroupByClause, VerboseError<&str>> {
        let (remaining_input, (_, _, _, columns, having)) = tuple((
            multispace0,
            tag_no_case("group by"),
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct JoinClause {
    pub operator: JoinOperator,
    pub right: JoinRightSide,
    pub constraint: JoinConstraint,
}

impl JoinClause {
    pub fn parse(i: &str) -> IResult<&str, JoinClause, VerboseError<&str>> {
        let (remaining_input, (_, _natural, operator, _, right, _, constraint)) = tuple((
            multispace0,
            opt(terminated(tag_no_case("natural"), multispace1)),
            JoinOperator::parse,
            multispace1,
            JoinRightSide::parse,
            multispace1,
            JoinConstraint::parse,
        ))(i)?;

        Ok((
            remaining_input,
            JoinClause {
                operator,
                right,
                constraint,
            },
        ))
    }
}

impl fmt::Display for JoinClause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.operator)?;
        write!(f, " {}", self.right)?;
        write!(f, " {}", self.constraint)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct LimitClause {
    pub limit: u64,
    pub offset: u64,
}

impl LimitClause {
    pub fn parse(i: &str) -> IResult<&str, LimitClause, VerboseError<&str>> {
        let (remaining_input, (_, _, _, limit, opt_offset)) = tuple((
            multispace0,
            tag_no_case("limit"),
            multispace1,
            unsigned_number,
            opt(offset),
        ))(i)?;
        let offset = opt_offset.unwrap_or_else(|| 0);

        Ok((remaining_input, LimitClause { limit, offset }))
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

fn offset(i: &str) -> IResult<&str, u64, VerboseError<&str>> {
    let (remaining_input, (_, _, _, val)) = tuple((
        multispace0,
        tag_no_case("OFFSET"),
        multispace1,
        unsigned_number,
    ))(i)?;

    Ok((remaining_input, val))
}

#[cfg(test)]
mod tests {
    use base::{FieldValueExpression, ItemPlaceholder, Operator};
    use base::column::{Column, FunctionArgument, FunctionArguments, FunctionExpression};
    use base::Literal;
    use base::table::Table;
    use common::{JoinConstraint, JoinOperator, JoinRightSide, OrderClause, OrderType};
    use common::case::{CaseWhenExpression, ColumnOrLiteral};
    use common::arithmetic::{ArithmeticBase, ArithmeticExpression, ArithmeticOperator};
    use common::condition::{ConditionExpression, ConditionTree};
    use common::condition::ConditionBase;
    use common::condition::ConditionBase::LiteralList;
    use common::condition::ConditionExpression::{Base, ComparisonOp, LogicalOp};

    use super::*;

    fn columns(cols: &[&str]) -> Vec<FieldDefinitionExpression> {
        cols.iter()
            .map(|c| FieldDefinitionExpression::Col(Column::from(*c)))
            .collect()
    }

    #[test]
    fn simple_select() {
        let str = "SELECT id, name FROM users;";

        let res = SelectStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("users")],
                fields: columns(&["id", "name"]),
                ..Default::default()
            }
        );
    }

    #[test]
    fn more_involved_select() {
        let str = "SELECT users.id, users.name FROM users;";

        let res = SelectStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("users")],
                fields: columns(&["users.id", "users.name"]),
                ..Default::default()
            }
        );
    }

    #[test]
    fn select_literals() {
        use base::Literal;

        let str = "SELECT NULL, 1, \"foo\", CURRENT_TIME FROM users;";
        // TODO: doesn't support selecting literals without a FROM clause, which is still valid SQL
        //        let str = "SELECT NULL, 1, \"foo\";";

        let res = SelectStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("users")],
                fields: vec![
                    FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                        Literal::Null.into(),
                    )),
                    FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                        Literal::Integer(1).into(),
                    )),
                    FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                        Literal::String("foo".to_owned()).into(),
                    )),
                    FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                        Literal::CurrentTime.into(),
                    )),
                ],
                ..Default::default()
            }
        );
    }

    #[test]
    fn select_all() {
        let str = "SELECT * FROM users;";

        let res = SelectStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("users")],
                fields: vec![FieldDefinitionExpression::All],
                ..Default::default()
            }
        );
    }

    #[test]
    fn select_all_in_table() {
        let str = "SELECT users.* FROM users, votes;";

        let res = SelectStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("users"), Table::from("votes")],
                fields: vec![FieldDefinitionExpression::AllInTable(String::from("users"))],
                ..Default::default()
            }
        );
    }

    #[test]
    fn spaces_optional() {
        let str = "SELECT id,name FROM users;";

        let res = SelectStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("users")],
                fields: columns(&["id", "name"]),
                ..Default::default()
            }
        );
    }

    #[test]
    fn case_sensitivity() {
        let str_lc = "select id, name from users;";
        let str_uc = "SELECT id, name FROM users;";

        assert_eq!(
            SelectStatement::parse(str_lc).unwrap(),
            SelectStatement::parse(str_uc).unwrap()
        );
    }

    #[test]
    fn termination() {
        let str_sem = "select id, name from users;";
        let str_nosem = "select id, name from users";
        let str_linebreak = "select id, name from users\n";

        let r1 = SelectStatement::parse(str_sem).unwrap();
        let r2 = SelectStatement::parse(str_nosem).unwrap();
        let r3 = SelectStatement::parse(str_linebreak).unwrap();
        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }

    #[test]
    fn where_clause() {
        where_clause_with_variable_placeholder(
            "select * from ContactInfo where email=?;",
            Literal::Placeholder(ItemPlaceholder::QuestionMark),
        );
    }

    #[test]
    fn where_clause_with_dollar_variable() {
        where_clause_with_variable_placeholder(
            "select * from ContactInfo where email= $3;",
            Literal::Placeholder(ItemPlaceholder::DollarNumber(3)),
        );
    }

    #[test]
    fn where_clause_with_colon_variable() {
        where_clause_with_variable_placeholder(
            "select * from ContactInfo where email= :5;",
            Literal::Placeholder(ItemPlaceholder::ColonNumber(5)),
        );
    }

    fn where_clause_with_variable_placeholder(str: &str, literal: Literal) {
        let res = SelectStatement::parse(str);

        let expected_left = Base(ConditionBase::Field(Column::from("email")));
        let expected_where_cond = Some(ComparisonOp(ConditionTree {
            left: Box::new(expected_left),
            right: Box::new(Base(ConditionBase::Literal(literal))),
            operator: Operator::Equal,
        }));
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("ContactInfo")],
                fields: vec![FieldDefinitionExpression::All],
                where_clause: expected_where_cond,
                ..Default::default()
            }
        );
    }

    #[test]
    fn limit_clause() {
        let str1 = "select * from users limit 10\n";
        let str2 = "select * from users limit 10 offset 10\n";

        let expected_lim1 = LimitClause {
            limit: 10,
            offset: 0,
        };
        let expected_lim2 = LimitClause {
            limit: 10,
            offset: 10,
        };

        let res1 = SelectStatement::parse(str1);
        let res2 = SelectStatement::parse(str2);
        assert_eq!(res1.unwrap().1.limit, Some(expected_lim1));
        assert_eq!(res2.unwrap().1.limit, Some(expected_lim2));
    }

    #[test]
    fn table_alias() {
        let str1 = "select * from PaperTag as t;";
        // let str2 = "select * from PaperTag t;";

        let res1 = SelectStatement::parse(str1);
        assert_eq!(
            res1.unwrap().1,
            SelectStatement {
                tables: vec![Table {
                    name: String::from("PaperTag"),
                    alias: Some(String::from("t")),
                    schema: None,
                },],
                fields: vec![FieldDefinitionExpression::All],
                ..Default::default()
            }
        );
        // let res2 = SelectStatement::parse(str2);
        // assert_eq!(res1.unwrap().1, res2.unwrap().1);
    }

    #[test]
    fn table_schema() {
        let str1 = "select * from db1.PaperTag as t;";

        let res1 = SelectStatement::parse(str1);
        assert_eq!(
            res1.unwrap().1,
            SelectStatement {
                tables: vec![Table {
                    name: String::from("PaperTag"),
                    alias: Some(String::from("t")),
                    schema: Some(String::from("db1")),
                },],
                fields: vec![FieldDefinitionExpression::All],
                ..Default::default()
            }
        );
        // let res2 = SelectStatement::parse(str2);
        // assert_eq!(res1.unwrap().1, res2.unwrap().1);
    }

    #[test]
    fn column_alias() {
        let str1 = "select name as TagName from PaperTag;";
        let str2 = "select PaperTag.name as TagName from PaperTag;";

        let res1 = SelectStatement::parse(str1);
        assert_eq!(
            res1.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("PaperTag")],
                fields: vec![FieldDefinitionExpression::Col(Column {
                    name: String::from("name"),
                    alias: Some(String::from("TagName")),
                    table: None,
                    function: None,
                }),],
                ..Default::default()
            }
        );
        let res2 = SelectStatement::parse(str2);
        assert_eq!(
            res2.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("PaperTag")],
                fields: vec![FieldDefinitionExpression::Col(Column {
                    name: String::from("name"),
                    alias: Some(String::from("TagName")),
                    table: Some(String::from("PaperTag")),
                    function: None,
                }),],
                ..Default::default()
            }
        );
    }

    #[test]
    fn column_alias_no_as() {
        let str1 = "select name TagName from PaperTag;";
        let str2 = "select PaperTag.name TagName from PaperTag;";

        let res1 = SelectStatement::parse(str1);
        assert_eq!(
            res1.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("PaperTag")],
                fields: vec![FieldDefinitionExpression::Col(Column {
                    name: String::from("name"),
                    alias: Some(String::from("TagName")),
                    table: None,
                    function: None,
                }),],
                ..Default::default()
            }
        );
        let res2 = SelectStatement::parse(str2);
        assert_eq!(
            res2.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("PaperTag")],
                fields: vec![FieldDefinitionExpression::Col(Column {
                    name: String::from("name"),
                    alias: Some(String::from("TagName")),
                    table: Some(String::from("PaperTag")),
                    function: None,
                }),],
                ..Default::default()
            }
        );
    }

    #[test]
    fn distinct() {
        let str = "select distinct tag from PaperTag where paperId=?;";

        let res = SelectStatement::parse(str);
        let expected_left = Base(ConditionBase::Field(Column::from("paperId")));
        let expected_where_cond = Some(ComparisonOp(ConditionTree {
            left: Box::new(expected_left),
            right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
                ItemPlaceholder::QuestionMark,
            )))),
            operator: Operator::Equal,
        }));
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("PaperTag")],
                distinct: true,
                fields: columns(&["tag"]),
                where_clause: expected_where_cond,
                ..Default::default()
            }
        );
    }

    #[test]
    fn simple_condition_expr() {
        let str = "select infoJson from PaperStorage where paperId=? and paperStorageId=?;";

        let res = SelectStatement::parse(str);

        let left_ct = ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("paperId")))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
                ItemPlaceholder::QuestionMark,
            )))),
            operator: Operator::Equal,
        };
        let left_comp = Box::new(ComparisonOp(left_ct));
        let right_ct = ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("paperStorageId")))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
                ItemPlaceholder::QuestionMark,
            )))),
            operator: Operator::Equal,
        };
        let right_comp = Box::new(ComparisonOp(right_ct));
        let expected_where_cond = Some(LogicalOp(ConditionTree {
            left: left_comp,
            right: right_comp,
            operator: Operator::And,
        }));
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("PaperStorage")],
                fields: columns(&["infoJson"]),
                where_clause: expected_where_cond,
                ..Default::default()
            }
        );
    }

    #[test]
    fn where_and_limit_clauses() {
        let str = "select * from users where id = ? limit 10\n";
        let res = SelectStatement::parse(str);

        let expected_lim = Some(LimitClause {
            limit: 10,
            offset: 0,
        });
        let ct = ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("id")))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
                ItemPlaceholder::QuestionMark,
            )))),
            operator: Operator::Equal,
        };
        let expected_where_cond = Some(ComparisonOp(ct));

        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("users")],
                fields: vec![FieldDefinitionExpression::All],
                where_clause: expected_where_cond,
                limit: expected_lim,
                ..Default::default()
            }
        );
    }

    #[test]
    fn aggregation_column() {
        let str = "SELECT max(addr_id) FROM address;";

        let res = SelectStatement::parse(str);
        let agg_expr = FunctionExpression::Max(FunctionArgument::Column(Column::from("addr_id")));
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("address")],
                fields: vec![FieldDefinitionExpression::Col(Column {
                    name: String::from("max(addr_id)"),
                    alias: None,
                    table: None,
                    function: Some(Box::new(agg_expr)),
                }),],
                ..Default::default()
            }
        );
    }

    #[test]
    fn aggregation_column_with_alias() {
        let str = "SELECT max(addr_id) AS max_addr FROM address;";

        let res = SelectStatement::parse(str);
        let agg_expr = FunctionExpression::Max(FunctionArgument::Column(Column::from("addr_id")));
        let expected_stmt = SelectStatement {
            tables: vec![Table::from("address")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("max_addr"),
                alias: Some(String::from("max_addr")),
                table: None,
                function: Some(Box::new(agg_expr)),
            })],
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected_stmt);
    }

    #[test]
    fn count_all() {
        let str = "SELECT COUNT(*) FROM votes GROUP BY aid;";

        let res = SelectStatement::parse(str);
        let agg_expr = FunctionExpression::CountStar;
        let expected_stmt = SelectStatement {
            tables: vec![Table::from("votes")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("count(*)"),
                alias: None,
                table: None,
                function: Some(Box::new(agg_expr)),
            })],
            group_by: Some(GroupByClause {
                columns: vec![Column::from("aid")],
                having: None,
            }),
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected_stmt);
    }

    #[test]
    fn count_distinct() {
        let str = "SELECT COUNT(DISTINCT vote_id) FROM votes GROUP BY aid;";

        let res = SelectStatement::parse(str);
        let agg_expr =
            FunctionExpression::Count(FunctionArgument::Column(Column::from("vote_id")), true);
        let expected_stmt = SelectStatement {
            tables: vec![Table::from("votes")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("count(distinct vote_id)"),
                alias: None,
                table: None,
                function: Some(Box::new(agg_expr)),
            })],
            group_by: Some(GroupByClause {
                columns: vec![Column::from("aid")],
                having: None,
            }),
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected_stmt);
    }

    #[test]
    fn count_filter() {
        let str = "SELECT COUNT(CASE WHEN vote_id > 10 THEN vote_id END) FROM votes GROUP BY aid;";
        let res = SelectStatement::parse(str);

        let filter_cond = ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("vote_id")))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Integer(10.into())))),
            operator: Operator::Greater,
        });
        let agg_expr = FunctionExpression::Count(
            FunctionArgument::Conditional(CaseWhenExpression {
                then_expr: ColumnOrLiteral::Column(Column::from("vote_id")),
                else_expr: None,
                condition: filter_cond,
            }),
            false,
        );
        let expected_stmt = SelectStatement {
            tables: vec![Table::from("votes")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: format!("{}", agg_expr),
                alias: None,
                table: None,
                function: Some(Box::new(agg_expr)),
            })],
            group_by: Some(GroupByClause {
                columns: vec![Column::from("aid")],
                having: None,
            }),
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected_stmt);
    }

    #[test]
    fn sum_filter() {
        let str = "SELECT SUM(CASE WHEN sign = 1 THEN vote_id END) FROM votes GROUP BY aid;";

        let res = SelectStatement::parse(str);

        let filter_cond = ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("sign")))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Integer(1.into())))),
            operator: Operator::Equal,
        });
        let agg_expr = FunctionExpression::Sum(
            FunctionArgument::Conditional(CaseWhenExpression {
                then_expr: ColumnOrLiteral::Column(Column::from("vote_id")),
                else_expr: None,
                condition: filter_cond,
            }),
            false,
        );
        let expected_stmt = SelectStatement {
            tables: vec![Table::from("votes")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: format!("{}", agg_expr),
                alias: None,
                table: None,
                function: Some(Box::new(agg_expr)),
            })],
            group_by: Some(GroupByClause {
                columns: vec![Column::from("aid")],
                having: None,
            }),
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected_stmt);
    }

    #[test]
    fn sum_filter_else_literal() {
        let str = "SELECT SUM(CASE WHEN sign = 1 THEN vote_id ELSE 6 END) FROM votes GROUP BY aid;";

        let res = SelectStatement::parse(str);

        let filter_cond = ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("sign")))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Integer(1.into())))),
            operator: Operator::Equal,
        });
        let agg_expr = FunctionExpression::Sum(
            FunctionArgument::Conditional(CaseWhenExpression {
                then_expr: ColumnOrLiteral::Column(Column::from("vote_id")),
                else_expr: Some(ColumnOrLiteral::Literal(Literal::Integer(6))),
                condition: filter_cond,
            }),
            false,
        );
        let expected_stmt = SelectStatement {
            tables: vec![Table::from("votes")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: format!("{}", agg_expr),
                alias: None,
                table: None,
                function: Some(Box::new(agg_expr)),
            })],
            group_by: Some(GroupByClause {
                columns: vec![Column::from("aid")],
                having: None,
            }),
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected_stmt);
    }

    #[test]
    fn count_filter_lobsters() {
        let str = "SELECT
            COUNT(CASE WHEN votes.story_id IS NULL AND votes.vote = 0 THEN votes.vote END) as votes
            FROM votes
            GROUP BY votes.comment_id;";

        let res = SelectStatement::parse(str);

        let filter_cond = LogicalOp(ConditionTree {
            left: Box::new(ComparisonOp(ConditionTree {
                left: Box::new(Base(ConditionBase::Field(Column::from("votes.story_id")))),
                right: Box::new(Base(ConditionBase::Literal(Literal::Null))),
                operator: Operator::Equal,
            })),
            right: Box::new(ComparisonOp(ConditionTree {
                left: Box::new(Base(ConditionBase::Field(Column::from("votes.vote")))),
                right: Box::new(Base(ConditionBase::Literal(Literal::Integer(0)))),
                operator: Operator::Equal,
            })),
            operator: Operator::And,
        });
        let agg_expr = FunctionExpression::Count(
            FunctionArgument::Conditional(CaseWhenExpression {
                then_expr: ColumnOrLiteral::Column(Column::from("votes.vote")),
                else_expr: None,
                condition: filter_cond,
            }),
            false,
        );
        let expected_stmt = SelectStatement {
            tables: vec![Table::from("votes")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("votes"),
                alias: Some(String::from("votes")),
                table: None,
                function: Some(Box::new(agg_expr)),
            })],
            group_by: Some(GroupByClause {
                columns: vec![Column::from("votes.comment_id")],
                having: None,
            }),
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected_stmt);
    }

    #[test]
    fn generic_function_query() {
        let str = "SELECT coalesce(a, b,c) as x,d FROM sometable;";

        let res = SelectStatement::parse(str);
        let agg_expr = FunctionExpression::Generic(
            String::from("coalesce"),
            FunctionArguments {
                arguments: vec![
                    FunctionArgument::Column(Column {
                        name: String::from("a"),
                        alias: None,
                        table: None,
                        function: None,
                    }),
                    FunctionArgument::Column(Column {
                        name: String::from("b"),
                        alias: None,
                        table: None,
                        function: None,
                    }),
                    FunctionArgument::Column(Column {
                        name: String::from("c"),
                        alias: None,
                        table: None,
                        function: None,
                    }),
                ],
            },
        );
        let expected_stmt = SelectStatement {
            tables: vec![Table::from("sometable")],
            fields: vec![
                FieldDefinitionExpression::Col(Column {
                    name: String::from("x"),
                    alias: Some(String::from("x")),
                    table: None,
                    function: Some(Box::new(agg_expr)),
                }),
                FieldDefinitionExpression::Col(Column {
                    name: String::from("d"),
                    alias: None,
                    table: None,
                    function: None,
                }),
            ],
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected_stmt);
    }

    #[test]
    fn moderately_complex_selection() {
        let str = "SELECT * FROM item, author WHERE item.i_a_id = author.a_id AND \
                       item.i_subject = ? ORDER BY item.i_title limit 50;";

        let res = SelectStatement::parse(str);
        let expected_where_cond = Some(LogicalOp(ConditionTree {
            left: Box::new(ComparisonOp(ConditionTree {
                left: Box::new(Base(ConditionBase::Field(Column::from("item.i_a_id")))),
                right: Box::new(Base(ConditionBase::Field(Column::from("author.a_id")))),
                operator: Operator::Equal,
            })),
            right: Box::new(ComparisonOp(ConditionTree {
                left: Box::new(Base(ConditionBase::Field(Column::from("item.i_subject")))),
                right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
                    ItemPlaceholder::QuestionMark,
                )))),
                operator: Operator::Equal,
            })),
            operator: Operator::And,
        }));
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("item"), Table::from("author")],
                fields: vec![FieldDefinitionExpression::All],
                where_clause: expected_where_cond,
                order: Some(OrderClause {
                    columns: vec![("item.i_title".into(), OrderType::Asc)],
                }),
                limit: Some(LimitClause {
                    limit: 50,
                    offset: 0,
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn simple_joins() {
        let str = "select paperId from PaperConflict join PCMember using (contactId);";

        let res = SelectStatement::parse(str);
        let expected_stmt = SelectStatement {
            tables: vec![Table::from("PaperConflict")],
            fields: columns(&["paperId"]),
            join: vec![JoinClause {
                operator: JoinOperator::Join,
                right: JoinRightSide::Table(Table::from("PCMember")),
                constraint: JoinConstraint::Using(vec![Column::from("contactId")]),
            }],
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected_stmt);

        // slightly simplified from
        // "select PCMember.contactId, group_concat(reviewType separator '')
        // from PCMember left join PaperReview on (PCMember.contactId=PaperReview.contactId)
        // group by PCMember.contactId"
        let str = "select PCMember.contactId \
                       from PCMember \
                       join PaperReview on (PCMember.contactId=PaperReview.contactId) \
                       order by contactId;";

        let res = SelectStatement::parse(str);
        let ct = ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from(
                "PCMember.contactId",
            )))),
            right: Box::new(Base(ConditionBase::Field(Column::from(
                "PaperReview.contactId",
            )))),
            operator: Operator::Equal,
        };
        let join_cond = ConditionExpression::ComparisonOp(ct);
        let expected = SelectStatement {
            tables: vec![Table::from("PCMember")],
            fields: columns(&["PCMember.contactId"]),
            join: vec![JoinClause {
                operator: JoinOperator::Join,
                right: JoinRightSide::Table(Table::from("PaperReview")),
                constraint: JoinConstraint::On(join_cond),
            }],
            order: Some(OrderClause {
                columns: vec![("contactId".into(), OrderType::Asc)],
            }),
            ..Default::default()
        };
        assert_eq!(res.unwrap().1, expected);

        // Same as above, but no brackets
        let str = "select PCMember.contactId \
                       from PCMember \
                       join PaperReview on PCMember.contactId=PaperReview.contactId \
                       order by contactId;";
        let res = SelectStatement::parse(str);
        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn multi_join() {
        // simplified from
        // "select max(conflictType), PaperReview.contactId as reviewer, PCMember.contactId as
        //  pcMember, ChairAssistant.contactId as assistant, Chair.contactId as chair,
        //  max(PaperReview.reviewNeedsSubmit) as reviewNeedsSubmit from ContactInfo
        //  left join PaperReview using (contactId) left join PaperConflict using (contactId)
        //  left join PCMember using (contactId) left join ChairAssistant using (contactId)
        //  left join Chair using (contactId) where ContactInfo.contactId=?
        //  group by ContactInfo.contactId;";
        let str = "select PCMember.contactId, ChairAssistant.contactId, \
                       Chair.contactId from ContactInfo left join PaperReview using (contactId) \
                       left join PaperConflict using (contactId) left join PCMember using \
                       (contactId) left join ChairAssistant using (contactId) left join Chair \
                       using (contactId) where ContactInfo.contactId=?;";

        let res = SelectStatement::parse(str);
        let ct = ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from(
                "ContactInfo.contactId",
            )))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
                ItemPlaceholder::QuestionMark,
            )))),
            operator: Operator::Equal,
        };
        let expected_where_cond = Some(ComparisonOp(ct));
        let mkjoin = |tbl: &str, col: &str| -> JoinClause {
            JoinClause {
                operator: JoinOperator::LeftJoin,
                right: JoinRightSide::Table(Table::from(tbl)),
                constraint: JoinConstraint::Using(vec![Column::from(col)]),
            }
        };
        assert_eq!(
            res.unwrap().1,
            SelectStatement {
                tables: vec![Table::from("ContactInfo")],
                fields: columns(&[
                    "PCMember.contactId",
                    "ChairAssistant.contactId",
                    "Chair.contactId"
                ]),
                join: vec![
                    mkjoin("PaperReview", "contactId"),
                    mkjoin("PaperConflict", "contactId"),
                    mkjoin("PCMember", "contactId"),
                    mkjoin("ChairAssistant", "contactId"),
                    mkjoin("Chair", "contactId"),
                ],
                where_clause: expected_where_cond,
                ..Default::default()
            }
        );
    }

    #[test]
    fn nested_select() {
        let qstr = "SELECT ol_i_id FROM orders, order_line \
                    WHERE orders.o_c_id IN (SELECT o_c_id FROM orders, order_line \
                    WHERE orders.o_id = order_line.ol_o_id);";

        let res = SelectStatement::parse(qstr);
        let inner_where_clause = ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_id")))),
            right: Box::new(Base(ConditionBase::Field(Column::from(
                "order_line.ol_o_id",
            )))),
            operator: Operator::Equal,
        });

        let inner_select = SelectStatement {
            tables: vec![Table::from("orders"), Table::from("order_line")],
            fields: columns(&["o_c_id"]),
            where_clause: Some(inner_where_clause),
            ..Default::default()
        };

        let outer_where_clause = ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_c_id")))),
            right: Box::new(Base(ConditionBase::NestedSelect(Box::new(inner_select)))),
            operator: Operator::In,
        });

        let outer_select = SelectStatement {
            tables: vec![Table::from("orders"), Table::from("order_line")],
            fields: columns(&["ol_i_id"]),
            where_clause: Some(outer_where_clause),
            ..Default::default()
        };

        assert_eq!(res.unwrap().1, outer_select);
    }

    #[test]
    fn recursive_nested_select() {
        let qstr = "SELECT ol_i_id FROM orders, order_line WHERE orders.o_c_id \
                    IN (SELECT o_c_id FROM orders, order_line \
                    WHERE orders.o_id = order_line.ol_o_id \
                    AND orders.o_id > (SELECT MAX(o_id) FROM orders));";

        let res = SelectStatement::parse(qstr);

        let agg_expr = FunctionExpression::Max(FunctionArgument::Column(Column::from("o_id")));
        let recursive_select = SelectStatement {
            tables: vec![Table::from("orders")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("max(o_id)"),
                alias: None,
                table: None,
                function: Some(Box::new(agg_expr)),
            })],
            ..Default::default()
        };

        let cop1 = ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_id")))),
            right: Box::new(Base(ConditionBase::Field(Column::from(
                "order_line.ol_o_id",
            )))),
            operator: Operator::Equal,
        });

        let cop2 = ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_id")))),
            right: Box::new(Base(ConditionBase::NestedSelect(Box::new(
                recursive_select,
            )))),
            operator: Operator::Greater,
        });

        let inner_where_clause = LogicalOp(ConditionTree {
            left: Box::new(cop1),
            right: Box::new(cop2),
            operator: Operator::And,
        });

        let inner_select = SelectStatement {
            tables: vec![Table::from("orders"), Table::from("order_line")],
            fields: columns(&["o_c_id"]),
            where_clause: Some(inner_where_clause),
            ..Default::default()
        };

        let outer_where_clause = ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_c_id")))),
            right: Box::new(Base(ConditionBase::NestedSelect(Box::new(inner_select)))),
            operator: Operator::In,
        });

        let outer_select = SelectStatement {
            tables: vec![Table::from("orders"), Table::from("order_line")],
            fields: columns(&["ol_i_id"]),
            where_clause: Some(outer_where_clause),
            ..Default::default()
        };

        assert_eq!(res.unwrap().1, outer_select);
    }

    #[test]
    fn join_against_nested_select() {
        let t0 = "(SELECT ol_i_id FROM order_line)";
        let t1 = "(SELECT ol_i_id FROM order_line) AS ids";

        assert!(JoinRightSide::parse(t0).is_ok());
        assert!(JoinRightSide::parse(t1).is_ok());

        let t0 = "JOIN (SELECT ol_i_id FROM order_line) ON (orders.o_id = ol_i_id)";
        let t1 = "JOIN (SELECT ol_i_id FROM order_line) AS ids ON (orders.o_id = ids.ol_i_id)";

        assert!(JoinClause::parse(t0).is_ok());
        assert!(JoinClause::parse(t1).is_ok());

        let qstr_with_alias = "SELECT o_id, ol_i_id FROM orders JOIN \
                               (SELECT ol_i_id FROM order_line) AS ids \
                               ON (orders.o_id = ids.ol_i_id);";
        let res = SelectStatement::parse(qstr_with_alias);

        // N.B.: Don't alias the inner select to `inner`, which is, well, a SQL keyword!
        let inner_select = SelectStatement {
            tables: vec![Table::from("order_line")],
            fields: columns(&["ol_i_id"]),
            ..Default::default()
        };

        let outer_select = SelectStatement {
            tables: vec![Table::from("orders")],
            fields: columns(&["o_id", "ol_i_id"]),
            join: vec![JoinClause {
                operator: JoinOperator::Join,
                right: JoinRightSide::NestedSelect(Box::new(inner_select), Some("ids".into())),
                constraint: JoinConstraint::On(ComparisonOp(ConditionTree {
                    operator: Operator::Equal,
                    left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_id")))),
                    right: Box::new(Base(ConditionBase::Field(Column::from("ids.ol_i_id")))),
                })),
            }],
            ..Default::default()
        };

        assert_eq!(res.unwrap().1, outer_select);
    }

    #[test]
    fn project_arithmetic_expressions() {
        let qstr = "SELECT MAX(o_id)-3333 FROM orders;";
        let res = SelectStatement::parse(qstr);

        let expected = SelectStatement {
            tables: vec![Table::from("orders")],
            fields: vec![FieldDefinitionExpression::Value(
                FieldValueExpression::Arithmetic(ArithmeticExpression::new(
                    ArithmeticOperator::Subtract,
                    ArithmeticBase::Column(Column {
                        name: String::from("max(o_id)"),
                        alias: None,
                        table: None,
                        function: Some(Box::new(FunctionExpression::Max(
                            FunctionArgument::Column("o_id".into()),
                        ))),
                    }),
                    ArithmeticBase::Scalar(3333.into()),
                    None,
                )),
            )],
            ..Default::default()
        };

        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn project_arithmetic_expressions_with_aliases() {
        let qstr = "SELECT max(o_id) * 2 as double_max FROM orders;";
        let res = SelectStatement::parse(qstr);

        let expected = SelectStatement {
            tables: vec![Table::from("orders")],
            fields: vec![FieldDefinitionExpression::Value(
                FieldValueExpression::Arithmetic(ArithmeticExpression::new(
                    ArithmeticOperator::Multiply,
                    ArithmeticBase::Column(Column {
                        name: String::from("max(o_id)"),
                        alias: None,
                        table: None,
                        function: Some(Box::new(FunctionExpression::Max(
                            FunctionArgument::Column("o_id".into()),
                        ))),
                    }),
                    ArithmeticBase::Scalar(2.into()),
                    Some(String::from("double_max")),
                )),
            )],
            ..Default::default()
        };

        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn where_in_clause() {
        let qstr = "SELECT `auth_permission`.`content_type_id`, `auth_permission`.`codename`
                    FROM `auth_permission`
                    JOIN `django_content_type`
                      ON ( `auth_permission`.`content_type_id` = `django_content_type`.`id` )
                    WHERE `auth_permission`.`content_type_id` IN (0);";
        let res = SelectStatement::parse(qstr);

        let expected_where_clause = Some(ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from(
                "auth_permission.content_type_id",
            )))),
            right: Box::new(Base(LiteralList(vec![0.into()]))),
            operator: Operator::In,
        }));

        let expected = SelectStatement {
            tables: vec![Table::from("auth_permission")],
            fields: vec![
                FieldDefinitionExpression::Col(Column::from("auth_permission.content_type_id")),
                FieldDefinitionExpression::Col(Column::from("auth_permission.codename")),
            ],
            join: vec![JoinClause {
                operator: JoinOperator::Join,
                right: JoinRightSide::Table(Table::from("django_content_type")),
                constraint: JoinConstraint::On(ComparisonOp(ConditionTree {
                    operator: Operator::Equal,
                    left: Box::new(Base(ConditionBase::Field(Column::from(
                        "auth_permission.content_type_id",
                    )))),
                    right: Box::new(Base(ConditionBase::Field(Column::from(
                        "django_content_type.id",
                    )))),
                })),
            }],
            where_clause: expected_where_clause,
            ..Default::default()
        };

        assert_eq!(res.unwrap().1, expected);
    }
}
