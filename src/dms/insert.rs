use std::fmt;
use std::str;

use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::opt;
use nom::multi::many1;
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

use base::column::Column;
use base::error::ParseSQLError;
use base::table::Table;
use base::{FieldValueExpression, Literal};
use common::keywords::escape_if_keyword;
use common::{statement_terminator, ws_sep_comma};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct InsertStatement {
    pub table: Table,
    pub fields: Option<Vec<Column>>,
    pub data: Vec<Vec<Literal>>,
    pub ignore: bool,
    pub on_duplicate: Option<Vec<(Column, FieldValueExpression)>>,
}

impl InsertStatement {
    // Parse rule for a SQL insert query.
    // TODO(malte): support REPLACE, nested selection, DEFAULT VALUES
    pub fn parse(i: &str) -> IResult<&str, InsertStatement, ParseSQLError<&str>> {
        let (
            remaining_input,
            (_, ignore_res, _, _, _, table, _, fields, _, _, data, on_duplicate, _),
        ) = tuple((
            tag_no_case("INSERT"),
            opt(preceded(multispace1, tag_no_case("IGNORE"))),
            multispace1,
            tag_no_case("INTO"),
            multispace1,
            Table::schema_table_reference,
            multispace0,
            opt(Self::fields),
            tag_no_case("VALUES"),
            multispace0,
            many1(Self::data),
            opt(Self::on_duplicate),
            statement_terminator,
        ))(i)?;
        assert!(table.alias.is_none());
        let ignore = ignore_res.is_some();

        Ok((
            remaining_input,
            InsertStatement {
                table,
                fields,
                data,
                ignore,
                on_duplicate,
            },
        ))
    }

    fn fields(i: &str) -> IResult<&str, Vec<Column>, ParseSQLError<&str>> {
        delimited(
            preceded(tag("("), multispace0),
            Column::field_list,
            delimited(multispace0, tag(")"), multispace1),
        )(i)
    }

    fn data(i: &str) -> IResult<&str, Vec<Literal>, ParseSQLError<&str>> {
        delimited(
            tag("("),
            Literal::value_list,
            preceded(tag(")"), opt(ws_sep_comma)),
        )(i)
    }

    fn on_duplicate(
        i: &str,
    ) -> IResult<&str, Vec<(Column, FieldValueExpression)>, ParseSQLError<&str>> {
        preceded(
            multispace0,
            preceded(
                tuple((
                    tag_no_case("ON"),
                    multispace1,
                    tag_no_case("DUPLICATE"),
                    multispace1,
                    tag_no_case("KYE"),
                    multispace1,
                    tag_no_case("UPDATE"),
                )),
                preceded(multispace1, FieldValueExpression::assignment_expr_list),
            ),
        )(i)
    }
}

impl fmt::Display for InsertStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "INSERT INTO {}", escape_if_keyword(&self.table.name))?;
        if let Some(ref fields) = self.fields {
            write!(
                f,
                " ({})",
                fields
                    .iter()
                    .map(|ref col| col.name.to_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        write!(
            f,
            " VALUES {}",
            self.data
                .iter()
                .map(|data| format!(
                    "({})",
                    data.into_iter()
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use base::{FieldValueExpression, ItemPlaceholder};
    use common::arithmetic::{ArithmeticBase, ArithmeticExpression, ArithmeticOperator};

    use super::*;

    #[test]
    fn simple_insert() {
        let str = "INSERT INTO users VALUES (42, \"test\");";

        let res = InsertStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: Table::from("users"),
                fields: None,
                data: vec![vec![42.into(), "test".into()]],
                ..Default::default()
            }
        );
    }

    #[test]
    fn simple_insert_schema() {
        let str = "INSERT INTO db1.users VALUES (42, \"test\");";

        let res = InsertStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: Table::from(("db1", "users")),
                fields: None,
                data: vec![vec![42.into(), "test".into()]],
                ..Default::default()
            }
        );
    }

    #[test]
    fn complex_insert() {
        let str = "INSERT INTO users VALUES (42, 'test', \"test\", CURRENT_TIMESTAMP);";

        let res = InsertStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: Table::from("users"),
                fields: None,
                data: vec![vec![
                    42.into(),
                    "test".into(),
                    "test".into(),
                    Literal::CurrentTimestamp,
                ],],
                ..Default::default()
            }
        );
    }

    #[test]
    fn insert_with_field_names() {
        let str = "INSERT INTO users (id, name) VALUES (42, \"test\");";

        let res = InsertStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: Table::from("users"),
                fields: Some(vec![Column::from("id"), Column::from("name")]),
                data: vec![vec![42.into(), "test".into()]],
                ..Default::default()
            }
        );
    }

    // Issue #3
    #[test]
    fn insert_without_spaces() {
        let str = "INSERT INTO users(id, name) VALUES(42, \"test\");";

        let res = InsertStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: Table::from("users"),
                fields: Some(vec![Column::from("id"), Column::from("name")]),
                data: vec![vec![42.into(), "test".into()]],
                ..Default::default()
            }
        );
    }

    #[test]
    fn multi_insert() {
        let str = "INSERT INTO users (id, name) VALUES (42, \"test\"),(21, \"test2\");";

        let res = InsertStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: Table::from("users"),
                fields: Some(vec![Column::from("id"), Column::from("name")]),
                data: vec![
                    vec![42.into(), "test".into()],
                    vec![21.into(), "test2".into()],
                ],
                ..Default::default()
            }
        );
    }

    #[test]
    fn insert_with_parameters() {
        let str = "INSERT INTO users (id, name) VALUES (?, ?);";

        let res = InsertStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: Table::from("users"),
                fields: Some(vec![Column::from("id"), Column::from("name")]),
                data: vec![vec![
                    Literal::Placeholder(ItemPlaceholder::QuestionMark),
                    Literal::Placeholder(ItemPlaceholder::QuestionMark),
                ]],
                ..Default::default()
            }
        );
    }

    #[test]
    fn insert_with_on_dup_update() {
        let str = "INSERT INTO keystores (`key`, `value`) VALUES ($1, :2) \
                       ON DUPLICATE KEY UPDATE `value` = `value` + 1";

        let res = InsertStatement::parse(str);
        let expected_ae = ArithmeticExpression::new(
            ArithmeticOperator::Add,
            ArithmeticBase::Column(Column::from("value")),
            ArithmeticBase::Scalar(1.into()),
            None,
        );
        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: Table::from("keystores"),
                fields: Some(vec![Column::from("key"), Column::from("value")]),
                data: vec![vec![
                    Literal::Placeholder(ItemPlaceholder::DollarNumber(1)),
                    Literal::Placeholder(ItemPlaceholder::ColonNumber(2)),
                ]],
                on_duplicate: Some(vec![(
                    Column::from("value"),
                    FieldValueExpression::Arithmetic(expected_ae),
                ),]),
                ..Default::default()
            }
        );
    }

    #[test]
    fn insert_with_leading_value_whitespace() {
        let str = "INSERT INTO users (id, name) VALUES ( 42, \"test\");";

        let res = InsertStatement::parse(str);
        assert_eq!(
            res.unwrap().1,
            InsertStatement {
                table: Table::from("users"),
                fields: Some(vec![Column::from("id"), Column::from("name")]),
                data: vec![vec![42.into(), "test".into()]],
                ..Default::default()
            }
        );
    }
}
