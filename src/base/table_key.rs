use std::fmt;

use base::column::Column;
use common::keywords::escape_if_keyword;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TableKey {
    PrimaryKey(Vec<Column>),
    UniqueKey(Option<String>, Vec<Column>),
    FulltextKey(Option<String>, Vec<Column>),
    Key(String, Vec<Column>),
}

impl fmt::Display for TableKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TableKey::PrimaryKey(ref columns) => {
                write!(f, "PRIMARY KEY ")?;
                write!(
                    f,
                    "({})",
                    columns
                        .iter()
                        .map(|c| escape_if_keyword(&c.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TableKey::UniqueKey(ref name, ref columns) => {
                write!(f, "UNIQUE KEY ")?;
                if let Some(ref name) = *name {
                    write!(f, "{} ", escape_if_keyword(name))?;
                }
                write!(
                    f,
                    "({})",
                    columns
                        .iter()
                        .map(|c| escape_if_keyword(&c.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TableKey::FulltextKey(ref name, ref columns) => {
                write!(f, "FULLTEXT KEY ")?;
                if let Some(ref name) = *name {
                    write!(f, "{} ", escape_if_keyword(name))?;
                }
                write!(
                    f,
                    "({})",
                    columns
                        .iter()
                        .map(|c| escape_if_keyword(&c.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            TableKey::Key(ref name, ref columns) => {
                write!(f, "KEY {} ", escape_if_keyword(name))?;
                write!(
                    f,
                    "({})",
                    columns
                        .iter()
                        .map(|c| escape_if_keyword(&c.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}
