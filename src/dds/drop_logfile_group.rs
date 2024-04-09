use core::fmt;
use std::fmt::Formatter;
use std::str;

use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;

use base::error::ParseSQLError;
use base::CommonParser;

/// parse `DROP LOGFILE GROUP logfile_group
///     ENGINE [=] engine_name`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropLogfileGroupStatement {
    pub logfile_group: String,
    pub engine_name: String,
}

impl DropLogfileGroupStatement {
    pub fn parse(i: &str) -> IResult<&str, DropLogfileGroupStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tag_no_case("DROP"),
            multispace1,
            tag_no_case("LOGFILE"),
            multispace1,
            tag_no_case("GROUP"),
            multispace1,
            map(CommonParser::sql_identifier, String::from),
            multispace1,
            map(
                tuple((
                    tag_no_case("ENGINE"),
                    multispace0,
                    opt(tag("=")),
                    multispace0,
                    CommonParser::sql_identifier,
                )),
                |(_, _, _, _, engine)| String::from(engine),
            ),
            multispace0,
            CommonParser::statement_terminator,
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
}

impl fmt::Display for DropLogfileGroupStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DROP LOGFILE GROUP")?;
        write!(f, " {}", self.logfile_group)?;
        write!(f, " ENGINE = {}", self.engine_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dds::drop_logfile_group::DropLogfileGroupStatement;

    #[test]
    fn parse_drop_logfile_group_parser() {
        let sqls = ["DROP LOGFILE GROUP logfile_group ENGINE = demo;"];
        let exp_statements = [DropLogfileGroupStatement {
            logfile_group: "logfile_group".to_string(),
            engine_name: "demo".to_string(),
        }];

        for i in 0..sqls.len() {
            let res = DropLogfileGroupStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i])
        }
    }
}
