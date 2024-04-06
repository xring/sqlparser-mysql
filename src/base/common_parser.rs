use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until, take_while, take_while1};
use nom::character::complete::{alpha1, digit1, line_ending, multispace0, multispace1};
use nom::character::is_alphanumeric;
use nom::combinator::{map, not, opt, peek, recognize};
use nom::error::{ErrorKind, ParseError};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{IResult, InputLength, Parser};

use base::column::Column;
use base::keywords::sql_keyword;
use base::{OrderType, ParseSQLError};

/// collection of common used parsers
pub struct CommonParser;

impl CommonParser {
    /// `[index_name]`
    pub fn opt_index_name(i: &str) -> IResult<&str, Option<String>, ParseSQLError<&str>> {
        opt(map(
            delimited(multispace1, CommonParser::sql_identifier, multispace0),
            String::from,
        ))(i)
    }

    #[allow(clippy::type_complexity)]
    pub fn index_col_name(
        i: &str,
    ) -> IResult<&str, (Column, Option<u16>, Option<OrderType>), ParseSQLError<&str>> {
        let (remaining_input, (column, len_u8, order)) = tuple((
            terminated(Column::without_alias, multispace0),
            opt(delimited(tag("("), digit1, tag(")"))),
            opt(OrderType::parse),
        ))(i)?;
        let len = len_u8.map(|l| u16::from_str(l).unwrap());

        Ok((remaining_input, (column, len, order)))
    }

    #[inline]
    fn is_sql_identifier(chr: char) -> bool {
        is_alphanumeric(chr as u8) || chr == '_' || chr == '@'
    }

    /// first and third are opt
    pub fn opt_delimited<I: Clone, O1, O2, O3, E: ParseError<I>, F, G, H>(
        mut first: F,
        mut second: G,
        mut third: H,
    ) -> impl FnMut(I) -> IResult<I, O2, E>
    where
        F: Parser<I, O1, E>,
        G: Parser<I, O2, E>,
        H: Parser<I, O3, E>,
    {
        move |input: I| {
            let inp = input.clone();
            match second.parse(input) {
                Ok((i, o)) => Ok((i, o)),
                _ => {
                    let (inp, _) = first.parse(inp)?;
                    let (inp, o2) = second.parse(inp)?;
                    third.parse(inp).map(|(i, _)| (i, o2))
                }
            }
        }
    }

    fn precision_helper(i: &str) -> IResult<&str, (u8, Option<u8>), ParseSQLError<&str>> {
        let (remaining_input, (m, d)) = tuple((
            digit1,
            opt(preceded(tag(","), preceded(multispace0, digit1))),
        ))(i)?;

        Ok((
            remaining_input,
            (m.parse().unwrap(), d.map(|r| r.parse().unwrap())),
        ))
    }

    pub fn precision(i: &str) -> IResult<&str, (u8, Option<u8>), ParseSQLError<&str>> {
        delimited(tag("("), Self::precision_helper, tag(")"))(i)
    }

    pub fn delim_digit(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        delimited(tag("("), digit1, tag(")"))(i)
    }

    pub fn sql_identifier(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        alt((
            alt((
                preceded(
                    not(peek(sql_keyword)),
                    recognize(pair(alpha1, take_while(Self::is_sql_identifier))),
                ),
                recognize(pair(tag("_"), take_while1(Self::is_sql_identifier))),
                // variable only
                recognize(pair(tag("@"), take_while1(Self::is_sql_identifier))),
            )),
            delimited(tag("`"), take_while1(Self::is_sql_identifier), tag("`")),
            delimited(tag("["), take_while1(Self::is_sql_identifier), tag("]")),
        ))(i)
    }

    // Parse an unsigned integer.
    pub fn unsigned_number(i: &str) -> IResult<&str, u64, ParseSQLError<&str>> {
        map(digit1, |d| FromStr::from_str(d).unwrap())(i)
    }

    pub fn eof<I: Copy + InputLength, E: ParseError<I>>(input: I) -> IResult<I, I, E> {
        if input.input_len() == 0 {
            Ok((input, input))
        } else {
            Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Eof)))
        }
    }

    // Parse a terminator that ends a SQL statement.
    pub fn statement_terminator(i: &str) -> IResult<&str, (), ParseSQLError<&str>> {
        let (remaining_input, _) = delimited(
            multispace0,
            alt((tag(";"), line_ending, CommonParser::eof)),
            multispace0,
        )(i)?;
        Ok((remaining_input, ()))
    }

    // Parse rule for AS-based aliases for SQL entities.
    pub fn as_alias(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        map(
            tuple((
                multispace1,
                opt(pair(tag_no_case("AS"), multispace1)),
                // FIXME as can starts with number
                CommonParser::sql_identifier,
            )),
            |a| a.2,
        )(i)
    }

    pub fn ws_sep_comma(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        delimited(multispace0, tag(","), multispace0)(i)
    }

    pub(crate) fn ws_sep_equals(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        delimited(multispace0, tag("="), multispace0)(i)
    }

    /// Parse rule for a comment part.
    /// COMMENT 'comment content'
    /// or
    /// COMMENT "comment content"
    pub fn parse_comment(i: &str) -> IResult<&str, String, ParseSQLError<&str>> {
        alt((
            map(
                preceded(
                    delimited(multispace0, tag_no_case("COMMENT"), multispace1),
                    delimited(tag("'"), take_until("'"), tag("'")),
                ),
                String::from,
            ),
            map(
                preceded(
                    delimited(multispace0, tag_no_case("COMMENT"), multispace1),
                    delimited(tag("\""), take_until("\""), tag("\"")),
                ),
                String::from,
            ),
        ))(i)
    }

    /// IF EXISTS
    pub fn parse_if_exists(i: &str) -> IResult<&str, Option<&str>, ParseSQLError<&str>> {
        opt(delimited(
            multispace0,
            delimited(tag_no_case("IF"), multispace1, tag_no_case("EXISTS")),
            multispace0,
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use nom::bytes::complete::tag;
    use nom::IResult;

    use base::CommonParser;

    #[test]
    fn sql_identifiers() {
        let id1 = "foo";
        let id2 = "f_o_o";
        let id3 = "foo12";
        let id4 = ":fo oo";
        let id5 = "primary ";
        let id6 = "`primary`";

        assert!(CommonParser::sql_identifier(id1).is_ok());
        assert!(CommonParser::sql_identifier(id2).is_ok());
        assert!(CommonParser::sql_identifier(id3).is_ok());
        assert!(CommonParser::sql_identifier(id4).is_err());
        assert!(CommonParser::sql_identifier(id5).is_err());
        assert!(CommonParser::sql_identifier(id6).is_ok());
    }

    fn test_opt_delimited_fn_call(i: &str) -> IResult<&str, &str> {
        CommonParser::opt_delimited(tag("("), tag("abc"), tag(")"))(i)
    }

    #[test]
    fn opt_delimited_tests() {
        // let ok1 = IResult::Ok(("".as_bytes(), "abc".as_bytes()));
        assert_eq!(test_opt_delimited_fn_call("abc"), IResult::Ok(("", "abc")));
        assert_eq!(
            test_opt_delimited_fn_call("(abc)"),
            IResult::Ok(("", "abc"))
        );
        assert!(test_opt_delimited_fn_call("(abc").is_err());
        assert_eq!(
            test_opt_delimited_fn_call("abc)"),
            IResult::Ok((")", "abc"))
        );
        assert!(test_opt_delimited_fn_call("ab").is_err());
    }

    #[test]
    fn comment_data() {
        let res = CommonParser::parse_comment(" COMMENT 'test'");
        assert_eq!(res.unwrap().1, "test");
    }

    #[test]
    fn terminated_by_semicolon() {
        let res = CommonParser::statement_terminator("   ;  ");
        assert_eq!(res, Ok(("", ())));
    }
}
