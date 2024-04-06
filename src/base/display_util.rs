use base::CommonParser;

pub struct DisplayUtil;

impl DisplayUtil {
    pub fn escape_if_keyword(s: &str) -> String {
        if CommonParser::sql_keyword(s).is_ok() {
            format!("`{}`", s)
        } else {
            s.to_owned()
        }
    }
}
