use std::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ItemPlaceholder {
    /// ?
    QuestionMark,
    /// $1 $2 $3
    DollarNumber(i32),
    /// :1 :2 :3
    ColonNumber(i32),
}

impl Display for ItemPlaceholder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ItemPlaceholder::QuestionMark => write!(f, "?"),
            ItemPlaceholder::DollarNumber(ref i) => write!(f, "${}", i),
            ItemPlaceholder::ColonNumber(ref i) => write!(f, ":{}", i),
        }
    }
}
