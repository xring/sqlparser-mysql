use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

use base::error::ParseSQLError;
use base::index_type::IndexType;
use base::visible_type::VisibleType;
use base::CommonParser;

/// index_option: {
///     KEY_BLOCK_SIZE [=] value
///   | index_type
///   | WITH PARSER parser_name
///   | COMMENT 'string'
///   | {VISIBLE | INVISIBLE}
///   | ENGINE_ATTRIBUTE [=] 'string' >>> create table only
///   | SECONDARY_ENGINE_ATTRIBUTE [=] 'string' >>> create table only
/// }
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IndexOption {
    KeyBlockSize(u64),
    IndexType(IndexType),
    WithParser(String),
    Comment(String),
    VisibleType(VisibleType),
    EngineAttribute(String),          // create table only
    SecondaryEngineAttribute(String), // create table only
}

impl IndexOption {
    pub fn parse(i: &str) -> IResult<&str, IndexOption, ParseSQLError<&str>> {
        alt((
            map(Self::key_block_size, IndexOption::KeyBlockSize),
            map(IndexType::parse, IndexOption::IndexType),
            map(Self::with_parser, IndexOption::WithParser),
            map(CommonParser::parse_comment, IndexOption::Comment),
            map(VisibleType::parse, IndexOption::VisibleType),
            map(Self::engine_attribute, IndexOption::EngineAttribute),
            map(
                Self::secondary_engine_attribute,
                IndexOption::SecondaryEngineAttribute,
            ),
        ))(i)
    }

    /// `[index_option]`
    /// index_option: {
    ///     KEY_BLOCK_SIZE [=] value
    ///   | index_type
    ///   | WITH PARSER parser_name
    ///   | COMMENT 'string'
    ///   | {VISIBLE | INVISIBLE}
    ///   |ENGINE_ATTRIBUTE [=] 'string'
    ///   |SECONDARY_ENGINE_ATTRIBUTE [=] 'string'
    /// }
    pub fn opt_index_option(i: &str) -> IResult<&str, Option<IndexOption>, ParseSQLError<&str>> {
        opt(map(preceded(multispace1, IndexOption::parse), |x| x))(i)
    }

    /// KEY_BLOCK_SIZE [=] value
    fn key_block_size(i: &str) -> IResult<&str, u64, ParseSQLError<&str>> {
        map(
            tuple((
                multispace0,
                tag_no_case("KEY_BLOCK_SIZE"),
                multispace0,
                opt(tag("=")),
                multispace0,
                complete::u64,
            )),
            |(_, _, _, _, _, size)| size,
        )(i)
    }

    /// WITH PARSER parser_name
    fn with_parser(i: &str) -> IResult<&str, String, ParseSQLError<&str>> {
        map(
            tuple((
                multispace0,
                tag_no_case("WITH"),
                multispace1,
                tag_no_case("PARSER"),
                multispace1,
                CommonParser::sql_identifier,
                multispace0,
            )),
            |(_, _, _, _, _, parser_name, _)| String::from(parser_name),
        )(i)
    }

    /// ENGINE_ATTRIBUTE [=] value
    fn engine_attribute(i: &str) -> IResult<&str, String, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ENGINE_ATTRIBUTE"),
                multispace0,
                opt(tag("=")),
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
                multispace0,
            )),
            |(_, _, _, engine, _)| engine,
        )(i)
    }

    /// SECONDARY_ENGINE_ATTRIBUTE [=] value
    fn secondary_engine_attribute(i: &str) -> IResult<&str, String, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("SECONDARY_ENGINE_ATTRIBUTE"),
                multispace0,
                opt(tag("=")),
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
                multispace0,
            )),
            |(_, _, _, engine, _)| engine,
        )(i)
    }
}
