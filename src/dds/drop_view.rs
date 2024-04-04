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
use common::{parse_if_exists, sql_identifier, statement_terminator, ws_sep_comma};

/// DROP VIEW [IF EXISTS]
///     view_name [, view_name] ...
///     [RESTRICT | CASCADE]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
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
            parse_if_exists,
            multispace0,
            map(many0(terminated(sql_identifier, opt(ws_sep_comma))), |x| {
                x.iter().map(|v| String::from(*v)).collect::<Vec<String>>()
            }),
            opt(delimited(multispace1, tag_no_case("RESTRICT"), multispace0)),
            opt(delimited(multispace1, tag_no_case("CASCADE"), multispace0)),
            statement_terminator,
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

impl Default for DropViewStatement {
    fn default() -> Self {
        DropViewStatement {
            if_exists: false,
            views: vec![],
            if_restrict: false,
            if_cascade: false,
        }
    }
}

impl fmt::Display for DropViewStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP VIEW")?;
        if self.if_exists {
            write!(f, " IF EXISTS ")?;
        }

        let view_name = self.views.join(", ");
        write!(f, "{}", view_name)?;

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
    fn test_drop_view() {
        let sqls = vec![
            "DROP VIEW view_name;",
            "DROP VIEW IF EXISTS view_name;",
            "DROP VIEW view_name CASCADE;",
            "DROP VIEW  view_name1, view_name2;",
            "DROP VIEW  view_name1, view_name2 RESTRICT;",
        ];

        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = DropViewStatement::parse(sqls[i]);
            // res.unwrap();
            println!("{:?}", res);
            // assert!(res.is_ok());
            // println!("{:?}", res);
        }
    }
}
