use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace0;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::CommonParser;

/// parse `DROP VIEW [IF EXISTS]
///     view_name [, view_name] ...
///     [RESTRICT | CASCADE]`
#[derive(Default, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropViewStatement {
    pub if_exists: bool,
    /// A name of a table, view, custom type, etc., possibly multipart, i.e. db.schema.obj
    pub views: Vec<String>,
    pub if_restrict: bool,
    pub if_cascade: bool,
}

impl DropViewStatement {
    /// DROP VIEW [IF EXISTS]
    ///     view_name [, view_name] ...
    ///     [RESTRICT | CASCADE]
    pub fn parse(i: &str) -> IResult<&str, DropViewStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tag_no_case("DROP "),
            multispace0,
            tag_no_case("VIEW "),
            CommonParser::parse_if_exists,
            multispace0,
            map(
                many0(terminated(
                    CommonParser::sql_identifier,
                    opt(CommonParser::ws_sep_comma),
                )),
                |x| x.iter().map(|v| String::from(*v)).collect::<Vec<String>>(),
            ),
            opt(delimited(multispace1, tag_no_case("RESTRICT"), multispace0)),
            opt(delimited(multispace1, tag_no_case("CASCADE"), multispace0)),
            CommonParser::statement_terminator,
        ));
        let (
            remaining_input,
            (_, _, _, opt_if_exists, _, views, opt_if_restrict, opt_if_cascade, _),
        ) = parser(i)?;

        Ok((
            remaining_input,
            DropViewStatement {
                views,
                if_exists: opt_if_exists.is_some(),
                if_restrict: opt_if_restrict.is_some(),
                if_cascade: opt_if_cascade.is_some(),
            },
        ))
    }
}

impl fmt::Display for DropViewStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP VIEW")?;
        if self.if_exists {
            write!(f, " IF EXISTS")?;
        }

        let view_name = self.views.join(", ");
        write!(f, " {}", view_name)?;

        if self.if_restrict {
            write!(f, " RESTRICT")?;
        }
        if self.if_cascade {
            write!(f, " CASCADE")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dds::drop_view::DropViewStatement;

    #[test]
    fn parse_drop_view() {
        let sqls = [
            "DROP VIEW view_name;",
            "DROP VIEW IF EXISTS view_name;",
            "DROP VIEW view_name CASCADE;",
            "DROP VIEW  view_name1, view_name2;",
            "DROP VIEW  view_name1, view_name2 RESTRICT;",
        ];

        let exp_statements = [
            DropViewStatement {
                if_exists: false,
                views: vec!["view_name".to_string()],
                if_restrict: false,
                if_cascade: false,
            },
            DropViewStatement {
                if_exists: true,
                views: vec!["view_name".to_string()],
                if_restrict: false,
                if_cascade: false,
            },
            DropViewStatement {
                if_exists: false,
                views: vec!["view_name".to_string()],
                if_restrict: false,
                if_cascade: true,
            },
            DropViewStatement {
                if_exists: false,
                views: vec!["view_name1".to_string(), "view_name2".to_string()],
                if_restrict: false,
                if_cascade: false,
            },
            DropViewStatement {
                if_exists: false,
                views: vec!["view_name1".to_string(), "view_name2".to_string()],
                if_restrict: true,
                if_cascade: false,
            },
        ];

        for i in 0..sqls.len() {
            let res = DropViewStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i]);
        }
    }
}
