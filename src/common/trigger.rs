use common::table::Table;
use std::fmt;
use std::str;

use keywords::escape_if_keyword;

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub schema: Option<String>,
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref schema) = self.schema {
            write!(f, "{}.", escape_if_keyword(schema))?;
        }
        write!(f, "{}", escape_if_keyword(&self.name))?;
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
