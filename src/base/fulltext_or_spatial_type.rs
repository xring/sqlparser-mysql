use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::IResult;

use base::ParseSQLError;

/// {FULLTEXT | SPATIAL}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FulltextOrSpatialType {
    Fulltext,
    Spatial,
}

impl FulltextOrSpatialType {
    /// {FULLTEXT | SPATIAL}
    pub fn parse(i: &str) -> IResult<&str, FulltextOrSpatialType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("FULLTEXT"), |_| FulltextOrSpatialType::Fulltext),
            map(tag_no_case("SPATIAL"), |_| FulltextOrSpatialType::Spatial),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use base::fulltext_or_spatial_type::FulltextOrSpatialType;

    #[test]
    fn parse_fulltext_or_spatial_type() {
        let str1 = "fulltext";
        let res1 = FulltextOrSpatialType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, FulltextOrSpatialType::Fulltext);

        let str2 = "SPATIAL";
        let res2 = FulltextOrSpatialType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, FulltextOrSpatialType::Spatial);
    }
}
