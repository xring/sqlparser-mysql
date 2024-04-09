use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;

use base::error::ParseSQLError;
use base::reference_type::ReferenceType;
use base::{CommonParser, KeyPart, MatchType};

/// reference_definition:
///     `REFERENCES tbl_name (key_part,...)
///       [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
///       [ON DELETE reference_option]
///       [ON UPDATE reference_option]`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ReferenceDefinition {
    pub tbl_name: String,
    pub key_part: Vec<KeyPart>,
    pub match_type: Option<MatchType>,
    pub on_delete: Option<ReferenceType>,
    pub on_update: Option<ReferenceType>,
}

impl ReferenceDefinition {
    pub fn parse(i: &str) -> IResult<&str, ReferenceDefinition, ParseSQLError<&str>> {
        let opt_on_delete = opt(map(
            tuple((
                tag_no_case("ON"),
                multispace1,
                tag_no_case("DELETE"),
                multispace1,
                ReferenceType::parse,
            )),
            |(_, _, _, _, reference_type)| reference_type,
        ));
        let opt_on_update = opt(map(
            tuple((
                tag_no_case("ON"),
                multispace1,
                tag_no_case("UPDATE"),
                multispace1,
                ReferenceType::parse,
            )),
            |(_, _, _, _, reference_type)| reference_type,
        ));

        map(
            tuple((
                tuple((multispace0, tag_no_case("REFERENCES"), multispace1)),
                // tbl_name
                map(CommonParser::sql_identifier, String::from),
                multispace0,
                KeyPart::parse, // (key_part,...)
                multispace0,
                opt(MatchType::parse), // [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
                multispace0,
                opt_on_delete,
                multispace0,
                opt_on_update,
                multispace0,
            )),
            |(_, tbl_name, _, key_part, _, match_type, _, on_delete, _, on_update, _)| {
                ReferenceDefinition {
                    tbl_name,
                    key_part,
                    match_type,
                    on_delete,
                    on_update,
                }
            },
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use base::reference_type::ReferenceType;
    use base::{KeyPart, KeyPartType, ReferenceDefinition};

    #[test]
    fn parse_reference_definition() {
        let str1 = "references tbl_name (col_name1, col_name2)";
        let res1 = ReferenceDefinition::parse(str1);
        let exp1 = ReferenceDefinition {
            tbl_name: "tbl_name".to_string(),
            key_part: vec![
                KeyPart {
                    r#type: KeyPartType::ColumnNameWithLength {
                        col_name: "col_name1".to_string(),
                        length: None,
                    },
                    order: None,
                },
                KeyPart {
                    r#type: KeyPartType::ColumnNameWithLength {
                        col_name: "col_name2".to_string(),
                        length: None,
                    },
                    order: None,
                },
            ],
            match_type: None,
            on_update: None,
            on_delete: None,
        };
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, exp1);

        let str2 = "references tbl_name (col_name1) ON DELETE set null";
        let res2 = ReferenceDefinition::parse(str2);
        let exp2 = ReferenceDefinition {
            tbl_name: "tbl_name".to_string(),
            key_part: vec![KeyPart {
                r#type: KeyPartType::ColumnNameWithLength {
                    col_name: "col_name1".to_string(),
                    length: None,
                },
                order: None,
            }],
            match_type: None,
            on_update: None,
            on_delete: Some(ReferenceType::SetNull),
        };
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, exp2);
    }
}
