use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{alphanumeric1, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::error::VerboseError;
use nom::IResult;
use nom::multi::separated_list0;
use nom::sequence::tuple;

use base::Literal;
use common::{
    sql_identifier, ws_sep_comma, ws_sep_equals,
};

pub fn table_options(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    // TODO: make the create options accessible
    map(
        separated_list0(table_options_separator, create_option),
        |_| (),
    )(i)
}

fn table_options_separator(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    map(alt((multispace1, ws_sep_comma)), |_| ())(i)
}

fn create_option(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    alt((
        create_option_type,
        create_option_pack_keys,
        create_option_engine,
        create_option_auto_increment,
        create_option_default_charset,
        create_option_collate,
        create_option_comment,
        create_option_max_rows,
        create_option_avg_row_length,
        create_option_row_format,
        create_option_key_block_size,
    ))(i)
}

/// Helper to parse equals-separated create option pairs.
/// Throws away the create option and value
pub fn create_option_equals_pair<'a, O1, O2, F, G>(
    mut first: F,
    mut second: G,
) -> impl FnMut(&'a str) -> IResult<&'a str, (), VerboseError<&'a str>>
where
    F: FnMut(&'a str) -> IResult<&'a str, O1, VerboseError<&'a str>>,
    G: FnMut(&'a str) -> IResult<&'a str, O2, VerboseError<&'a str>>,
{
    move |i: &'a str| {
        let (i, _o1) = first(i)?;
        let (i, _) = ws_sep_equals(i)?;
        let (i, _o2) = second(i)?;
        Ok((i, ()))
    }
}

fn create_option_type(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    create_option_equals_pair(tag_no_case("type"), alphanumeric1)(i)
}

fn create_option_pack_keys(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    create_option_equals_pair(tag_no_case("pack_keys"), alt((tag("0"), tag("1"))))(i)
}

fn create_option_engine(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    create_option_equals_pair(tag_no_case("engine"), opt(alphanumeric1))(i)
}

fn create_option_auto_increment(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    create_option_equals_pair(tag_no_case("auto_increment"), Literal::integer_literal)(i)
}

fn create_option_default_charset(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    create_option_equals_pair(
        tag_no_case("default charset"),
        alt((
            tag("utf8mb4"),
            tag("utf8"),
            tag("binary"),
            tag("big5"),
            tag("ucs2"),
            tag("latin1"),
        )),
    )(i)
}

fn create_option_collate(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    create_option_equals_pair(
        tag_no_case("collate"),
        // TODO(malte): imprecise hack, should not accept everything
        sql_identifier,
    )(i)
}

fn create_option_comment(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    create_option_equals_pair(tag_no_case("comment"), Literal::string_literal)(i)
}

fn create_option_max_rows(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    create_option_equals_pair(tag_no_case("max_rows"), Literal::integer_literal)(i)
}

fn create_option_avg_row_length(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    create_option_equals_pair(tag_no_case("avg_row_length"), Literal::integer_literal)(i)
}

fn create_option_row_format(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    let (remaining_input, (_, _, _, _, _)) = tuple((
        tag_no_case("row_format"),
        multispace0,
        opt(tag("=")),
        multispace0,
        alt((
            tag_no_case("DEFAULT"),
            tag_no_case("DYNAMIC"),
            tag_no_case("FIXED"),
            tag_no_case("COMPRESSED"),
            tag_no_case("REDUNDANT"),
            tag_no_case("COMPACT"),
        )),
    ))(i)?;
    Ok((remaining_input, ()))
}

fn create_option_key_block_size(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    let (remaining_input, (_, _, _, _, _)) = tuple((
        tag_no_case("key_block_size"),
        multispace0,
        opt(tag("=")),
        multispace0,
        Literal::integer_literal,
    ))(i)?;
    Ok((remaining_input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn should_parse_all(str: &str) {
        assert_eq!(Ok(("", ())), table_options(str))
    }

    #[test]
    fn create_table_option_list_empty() {
        should_parse_all("");
    }

    #[test]
    fn create_table_option_list() {
        should_parse_all(
            "ENGINE=InnoDB AutoIncrement=44782967 \
             DEFAULT CHARSET=binary ROW_FORMAT=COMPRESSED KEY_BLOCK_SIZE=8",
        );
    }

    #[test]
    fn create_table_option_list_commaseparated() {
        should_parse_all("AutoIncrement=1,ENGINE=,KEY_BLOCK_SIZE=8");
    }
}
