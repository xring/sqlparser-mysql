use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, tuple};
use nom::IResult;

use common_parsers::{parse_comment, sql_identifier};
use common_statement::{index_type, visible_or_invisible, IndexType, VisibleType};

/// index_option: {
///     KEY_BLOCK_SIZE [=] value
///   | index_type
///   | WITH PARSER parser_name
///   | COMMENT 'string'
///   | {VISIBLE | INVISIBLE}
///   | ENGINE_ATTRIBUTE [=] 'string' > FROM create table
///   | SECONDARY_ENGINE_ATTRIBUTE [=] 'string' > FROM create table
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

/// index_option: {
///     KEY_BLOCK_SIZE [=] value
///   | index_type
///   | WITH PARSER parser_name
///   | COMMENT 'string'
///   | {VISIBLE | INVISIBLE}
/// }
pub fn index_option(i: &str) -> IResult<&str, IndexOption> {
    alt((
        map(key_block_size, |x| IndexOption::KeyBlockSize(x)),
        map(index_type, |x| IndexOption::IndexType(x)),
        map(with_parser, |x| IndexOption::WithParser(x)),
        map(parse_comment, |x| IndexOption::Comment(x)),
        map(visible_or_invisible, |x| IndexOption::VisibleType(x)),
        map(engine_attribute, |x| IndexOption::EngineAttribute(x)),
        map(secondary_engine_attribute, |x| {
            IndexOption::SecondaryEngineAttribute(x)
        }),
    ))(i)
}

/// KEY_BLOCK_SIZE [=] value
fn key_block_size(i: &str) -> IResult<&str, u64> {
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
fn with_parser(i: &str) -> IResult<&str, String> {
    map(
        tuple((
            multispace0,
            tag_no_case("WITH"),
            multispace1,
            tag_no_case("PARSER"),
            multispace1,
            sql_identifier,
            multispace0,
        )),
        |(_, _, _, _, _, parser_name, _)| String::from(parser_name),
    )(i)
}

/// ENGINE_ATTRIBUTE [=] value
fn engine_attribute(i: &str) -> IResult<&str, String> {
    map(
        tuple((
            tag_no_case("ENGINE_ATTRIBUTE "),
            multispace0,
            opt(tag("=")),
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x| String::from(x),
            ),
            multispace0,
        )),
        |(_, _, _, engine, _)| engine,
    )(i)
}

/// SECONDARY_ENGINE_ATTRIBUTE [=] value
fn secondary_engine_attribute(i: &str) -> IResult<&str, String> {
    map(
        tuple((
            tag_no_case("SECONDARY_ENGINE_ATTRIBUTE "),
            multispace0,
            opt(tag("=")),
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x| String::from(x),
            ),
            multispace0,
        )),
        |(_, _, _, engine, _)| engine,
    )(i)
}
