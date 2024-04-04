use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;

use base::error::ParseSQLError;
use common::{sql_identifier, KeyPart};
use common::{MatchType, ReferenceType};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ReferenceDefinition {
    tbl_name: String,
    key_part: Vec<KeyPart>,
    match_type: Option<MatchType>,
    on_delete: Option<ReferenceType>,
    on_update: Option<ReferenceType>,
}

impl ReferenceDefinition {
    /// reference_definition:
    ///     REFERENCES tbl_name (key_part,...)
    ///       [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
    ///       [ON DELETE reference_option]
    ///       [ON UPDATE reference_option]
    pub fn parse(i: &str) -> IResult<&str, ReferenceDefinition, ParseSQLError<&str>> {
        let opt_on_delete = opt(map(
            tuple((
                tag_no_case("ON"),
                multispace1,
                tag_no_case("DELETE"),
                multispace1,
                ReferenceType::parse,
            )),
            |x| x.4,
        ));
        let opt_on_update = opt(map(
            tuple((
                tag_no_case("ON"),
                multispace1,
                tag_no_case("UPDATE"),
                multispace1,
                ReferenceType::parse,
            )),
            |x| x.4,
        ));
        map(
            tuple((
                tuple((multispace0, tag_no_case("REFERENCES"), multispace1)),
                // tbl_name
                map(sql_identifier, |x| String::from(x)),
                multispace0,
                KeyPart::key_part_list, // (key_part,...)
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
