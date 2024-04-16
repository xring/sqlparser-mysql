use std::fmt;
use std::str;

use nom::bytes::complete::tag;
use nom::combinator::{map, opt};
use nom::sequence::{pair, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::{CommonParser, DisplayUtil};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub schema: Option<String>,
}

impl Trigger {
    pub fn parse(i: &str) -> IResult<&str, Trigger, ParseSQLError<&str>> {
        map(
            tuple((
                opt(pair(CommonParser::sql_identifier, tag("."))),
                CommonParser::sql_identifier,
            )),
            |tup| Trigger {
                name: String::from(tup.1),
                schema: tup.0.map(|(schema, _)| String::from(schema)),
            },
        )(i)
    }
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref schema) = self.schema {
            write!(f, "{}.", DisplayUtil::escape_if_keyword(schema))?;
        }
        write!(f, "{}", DisplayUtil::escape_if_keyword(&self.name))?;
        Ok(())
    }
}

impl<'a> From<&'a str> for Trigger {
    fn from(t: &str) -> Trigger {
        Trigger {
            name: String::from(t),
            schema: None,
        }
    }
}

impl<'a> From<(&'a str, &'a str)> for Trigger {
    fn from(t: (&str, &str)) -> Trigger {
        Trigger {
            name: String::from(t.1),
            schema: Some(String::from(t.0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use base::Trigger;
    use nom::combinator::into;

    #[test]
    fn parse_trigger() {
        let str1 = "trigger_name";
        let res1 = Trigger::parse(str1);
        let exp1 = Trigger {
            name: "trigger_name".to_string(),
            schema: None,
        };
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, exp1);

        let str2 = "foo.trigger_name";
        let res2 = Trigger::parse(str2);
        let exp2 = Trigger {
            name: "trigger_name".to_string(),
            schema: Some("foo".to_string()),
        };
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, exp2);
    }

    #[test]
    fn from_str() {
        let trigger1: Trigger = "trigger_name".into();
        let exp1 = Trigger {
            name: "trigger_name".to_string(),
            schema: None,
        };
        assert_eq!(trigger1, exp1);
    }

    #[test]
    fn from_tuple_str() {
        let trigger2: Trigger = ("foo", "trigger_name").into();
        let exp2 = Trigger {
            name: "trigger_name".to_string(),
            schema: Some("foo".to_string()),
        };
        assert_eq!(trigger2, exp2);
    }
}
