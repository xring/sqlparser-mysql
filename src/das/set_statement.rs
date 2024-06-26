use std::{fmt, str};

use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::sequence::tuple;
use nom::IResult;

use base::error::ParseSQLError;
use base::{CommonParser, Literal};

/// parse `SET variable = expr [, variable = expr] ...`
///
/// `variable: {
///     user_var_name
///   | param_name
///   | local_var_name
///   | {GLOBAL | @@GLOBAL.} system_var_name
///   | {PERSIST | @@PERSIST.} system_var_name
///   | {PERSIST_ONLY | @@PERSIST_ONLY.} system_var_name
///   | [SESSION | @@SESSION. | @@] system_var_name
/// }`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SetStatement {
    pub variable: String,
    pub value: Literal,
}

impl SetStatement {
    pub fn parse(i: &str) -> IResult<&str, SetStatement, ParseSQLError<&str>> {
        let (remaining_input, (_, _, var, _, _, _, value, _)) = tuple((
            tag_no_case("SET"),
            multispace1,
            CommonParser::sql_identifier,
            multispace0,
            tag_no_case("="),
            multispace0,
            Literal::parse,
            CommonParser::statement_terminator,
        ))(i)?;
        let variable = String::from(var);
        Ok((remaining_input, SetStatement { variable, value }))
    }
}

impl fmt::Display for SetStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SET ")?;
        write!(f, "{} = {}", self.variable, self.value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_set() {
        let str = "SET SQL_AUTO_IS_NULL = 0;";
        let res = SetStatement::parse(str);
        let exp = SetStatement {
            variable: "SQL_AUTO_IS_NULL".to_owned(),
            value: 0.into(),
        };
        assert_eq!(res.unwrap().1, exp);
    }

    #[test]
    fn user_defined_vars() {
        let str = "SET @var = 123;";
        let res = SetStatement::parse(str);
        let exp = SetStatement {
            variable: "@var".to_owned(),
            value: 123.into(),
        };
        assert_eq!(res.unwrap().1, exp);
    }

    #[test]
    fn format_set() {
        let str = "set autocommit=1";
        let expected = "SET autocommit = 1";
        let res = SetStatement::parse(str);
        assert_eq!(format!("{}", res.unwrap().1), expected);
    }
}
