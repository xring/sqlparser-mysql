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
pub fn index_option(i: &[u8]) -> IResult<&[u8], IndexOption> {
    let mut parser = alt((
        map(key_block_size, |x| IndexOption::KeyBlockSize(x)),
        map(index_type, |x| IndexOption::IndexType(x)),
        map(with_parser, |x| IndexOption::WithParser(x)),
        map(parse_comment, |x| IndexOption::Comment(x)),
        map(visible_or_invisible, |x| IndexOption::VisibleType(x)),
        map(engine_attribute, |x| IndexOption::EngineAttribute(x)),
        map(secondary_engine_attribute, |x| {
            IndexOption::SecondaryEngineAttribute(x)
        }),
    ));

    match parser(i) {
        Ok(res) => Ok(res),
        Err(err) => {
            let template = r###"
            KEY_BLOCK_SIZE [=] value
            | index_type
            | WITH PARSER parser_name
            | COMMENT 'string'
            | {VISIBLE | INVISIBLE}
            "###;
            println!(
                "failed to parse ---{}--- to '{}': {}",
                String::from(std::str::from_utf8(i).unwrap()),
                template,
                err
            );
            Err(err)
        }
    }
}

/// KEY_BLOCK_SIZE [=] value
fn key_block_size(i: &[u8]) -> IResult<&[u8], u64> {
    let mut parser = tuple((
        multispace0,
        tag_no_case("KEY_BLOCK_SIZE"),
        multispace0,
        opt(tag("=")),
        multispace0,
        complete::u64,
    ));

    match parser(i) {
        Ok((input, (_, _, _, _, _, size))) => Ok((input, size)),
        Err(err) => {
            println!(
                "failed to parse ---{}--- to 'KEY_BLOCK_SIZE [=] value': {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// WITH PARSER parser_name
fn with_parser(i: &[u8]) -> IResult<&[u8], String> {
    let mut parser = tuple((
        multispace0,
        tag_no_case("WITH"),
        multispace1,
        tag_no_case("PARSER"),
        multispace1,
        sql_identifier,
        multispace0,
    ));

    match parser(i) {
        Ok((remaining_input, (_, _, _, _, _, parser_name, _))) => {
            let parser_name = String::from(std::str::from_utf8(parser_name).unwrap());
            Ok((remaining_input, parser_name))
        }
        Err(err) => {
            println!(
                "failed to parse ---{}--- to 'WITH PARSER parser_name': {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// ENGINE_ATTRIBUTE [=] value
fn engine_attribute(i: &[u8]) -> IResult<&[u8], String> {
    let mut parser = map(
        tuple((
            tag_no_case("ENGINE_ATTRIBUTE "),
            multispace0,
            opt(tag("=")),
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
            ),
            multispace0,
        )),
        |(_, _, _, engine, _)| engine,
    );

    match parser(i) {
        Ok((input, (engine))) => Ok((input, engine)),
        Err(err) => {
            println!(
                "failed to parse ---{}--- to 'ENGINE_ATTRIBUTE [=] value': {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// SECONDARY_ENGINE_ATTRIBUTE [=] value
fn secondary_engine_attribute(i: &[u8]) -> IResult<&[u8], String> {
    let mut parser = map(
        tuple((
            tag_no_case("SECONDARY_ENGINE_ATTRIBUTE "),
            multispace0,
            opt(tag("=")),
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
            ),
            multispace0,
        )),
        |(_, _, _, engine, _)| engine,
    );

    match parser(i) {
        Ok((input, (engine))) => Ok((input, engine)),
        Err(err) => {
            println!(
                "failed to parse ---{}--- to 'SECONDARY_ENGINE_ATTRIBUTE [=] value': {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}
