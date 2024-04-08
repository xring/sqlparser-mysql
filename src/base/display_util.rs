use base::CommonParser;

pub struct DisplayUtil;

impl DisplayUtil {
    /// add `` to string if string is a MySQL keyword
    pub fn escape_if_keyword(s: &str) -> String {
        if CommonParser::sql_keyword(s).is_ok() {
            format!("`{}`", s)
        } else {
            s.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ParseConfig, Parser};

    #[test]
    fn escaped_keyword() {
        let str0 = "delete from articles where `key`='aaa'";
        let str1 = "delete from `where` where user=?";

        let expected0 = "DELETE FROM articles WHERE `key` = 'aaa'";
        let expected1 = "DELETE FROM `where` WHERE user = ?";
        let config = ParseConfig::default();
        let res0 = Parser::parse(&config, str0);
        let res1 = Parser::parse(&config, str1);
        assert!(res0.is_ok());
        assert!(res1.is_ok());
        assert_eq!(expected0, format!("{}", res0.unwrap()));
        assert_eq!(expected1, format!("{}", res1.unwrap()));
    }
}
