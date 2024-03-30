use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::error::VerboseError;
use nom::sequence::tuple;
use nom::IResult;

use common_parsers::{sql_identifier, statement_terminator};

/// DROP LOGFILE GROUP logfile_group
///     ENGINE [=] engine_name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropLogfileGroupStatement {
    pub logfile_group: String,
    pub engine_name: String,
}

impl fmt::Display for DropLogfileGroupStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP LOGFILE GROUP")?;
        write!(f, " {}", self.logfile_group)?;
        write!(f, " ENGINE = {}", self.engine_name)?;
        Ok(())
    }
}

/// DROP LOGFILE GROUP logfile_group
///     ENGINE [=] engine_name
pub fn drop_logfile_group_parser(i: &str) -> IResult<&str, DropLogfileGroupStatement, VerboseError<&str>> {
    let mut parser = tuple((
        tag_no_case("DROP "),
        multispace0,
        tag_no_case("LOGFILE "),
        multispace0,
        tag_no_case("GROUP"),
        multispace0,
        map(sql_identifier, |logfile_group| {
            String::from(logfile_group)
        }),
        multispace0,
        map(
            tuple((
                tag_no_case("ENGINE"),
                multispace1,
                opt(tag("=")),
                multispace0,
                sql_identifier,
                multispace0,
            )),
            |(_, _, _, _, engine, _)| String::from(engine),
        ),
        multispace0,
        statement_terminator,
    ));
    let (remaining_input, (_, _, _, _, _, _, logfile_group, _, engine_name, _, _)) = parser(i)?;

    Ok((
        remaining_input,
        DropLogfileGroupStatement {
            logfile_group,
            engine_name,
        },
    ))
}

#[cfg(test)]
mod tests {
    use data_definition_statement::drop_logfile_group_parser;

    #[test]
    fn test_drop_logfile_group_parser() {
        let sqls = vec!["DROP LOGFILE GROUP logfile_group ENGINE = demo;"];

        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = drop_logfile_group_parser(sqls[i]);
            println!("{:?}", res);
            assert!(res.is_ok());
        }
    }
}
