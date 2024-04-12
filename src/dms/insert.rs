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
use base::{CommonParser, DisplayUtil, FieldValueExpression, Literal};

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
            (_, ignore_res, _, _, _, table, _, fields, _, _, data, on_duplicate, _, _),
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
            multispace0,
            CommonParser::statement_terminator,
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
            preceded(tag(")"), opt(CommonParser::ws_sep_comma)),
        )(i)
    }

    pub fn on_duplicate(
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
                    tag_no_case("KEY"),
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
        write!(
            f,
            "INSERT INTO {}",
            DisplayUtil::escape_if_keyword(&self.table.name)
        )?;
        if let Some(ref fields) = self.fields {
            write!(
                f,
                " ({})",
                fields
                    .iter()
                    .map(|col| col.name.to_owned())
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
                    data.iter()
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
