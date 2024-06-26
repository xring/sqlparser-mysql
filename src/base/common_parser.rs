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
use base::{DefaultOrZeroOrOne, OrderType, ParseSQLError};

/// collection of common used parsers
pub struct CommonParser;

impl CommonParser {
    fn keyword_follow_char(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        peek(alt((
            tag(" "),
            tag("\n"),
            tag(";"),
            tag("("),
            tag(")"),
            tag("\t"),
            tag(","),
            tag("="),
            CommonParser::eof,
        )))(i)
    }

    fn keywords_part_1(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        alt((
            terminated(tag_no_case("ABORT"), Self::keyword_follow_char),
            terminated(tag_no_case("ACTION"), Self::keyword_follow_char),
            terminated(tag_no_case("ADD"), Self::keyword_follow_char),
            terminated(tag_no_case("AFTER"), Self::keyword_follow_char),
            terminated(tag_no_case("ALL"), Self::keyword_follow_char),
            terminated(tag_no_case("ALTER"), Self::keyword_follow_char),
            terminated(tag_no_case("ANALYZE"), Self::keyword_follow_char),
            terminated(tag_no_case("AND"), Self::keyword_follow_char),
            terminated(tag_no_case("AS"), Self::keyword_follow_char),
            terminated(tag_no_case("ASC"), Self::keyword_follow_char),
            terminated(tag_no_case("ATTACH"), Self::keyword_follow_char),
            terminated(tag_no_case("AUTOINCREMENT"), Self::keyword_follow_char),
            terminated(tag_no_case("BEFORE"), Self::keyword_follow_char),
            terminated(tag_no_case("BEGIN"), Self::keyword_follow_char),
            terminated(tag_no_case("BETWEEN"), Self::keyword_follow_char),
            terminated(tag_no_case("BY"), Self::keyword_follow_char),
            terminated(tag_no_case("CASCADE"), Self::keyword_follow_char),
            terminated(tag_no_case("CASE"), Self::keyword_follow_char),
            terminated(tag_no_case("CAST"), Self::keyword_follow_char),
            terminated(tag_no_case("CHECK"), Self::keyword_follow_char),
            terminated(tag_no_case("COLLATE"), Self::keyword_follow_char),
        ))(i)
    }

    fn keywords_part_2(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        alt((
            terminated(tag_no_case("COLUMN"), Self::keyword_follow_char),
            terminated(tag_no_case("COMMIT"), Self::keyword_follow_char),
            terminated(tag_no_case("CONFLICT"), Self::keyword_follow_char),
            terminated(tag_no_case("CONSTRAINT"), Self::keyword_follow_char),
            terminated(tag_no_case("CREATE"), Self::keyword_follow_char),
            terminated(tag_no_case("CROSS"), Self::keyword_follow_char),
            terminated(tag_no_case("CURRENT_DATE"), Self::keyword_follow_char),
            terminated(tag_no_case("CURRENT_TIME"), Self::keyword_follow_char),
            terminated(tag_no_case("CURRENT_TIMESTAMP"), Self::keyword_follow_char),
            terminated(tag_no_case("DATABASE"), Self::keyword_follow_char),
            terminated(tag_no_case("DEFAULT"), Self::keyword_follow_char),
            terminated(tag_no_case("DEFERRABLE"), Self::keyword_follow_char),
            terminated(tag_no_case("DEFERRED"), Self::keyword_follow_char),
            terminated(tag_no_case("DELETE"), Self::keyword_follow_char),
            terminated(tag_no_case("DESC"), Self::keyword_follow_char),
            terminated(tag_no_case("DETACH"), Self::keyword_follow_char),
            terminated(tag_no_case("DISTINCT"), Self::keyword_follow_char),
            terminated(tag_no_case("DROP"), Self::keyword_follow_char),
            terminated(tag_no_case("EACH"), Self::keyword_follow_char),
            terminated(tag_no_case("ELSE"), Self::keyword_follow_char),
            terminated(tag_no_case("END"), Self::keyword_follow_char),
        ))(i)
    }

    fn keywords_part_3(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        alt((
            terminated(tag_no_case("ESCAPE"), Self::keyword_follow_char),
            terminated(tag_no_case("EXCEPT"), Self::keyword_follow_char),
            terminated(tag_no_case("EXCLUSIVE"), Self::keyword_follow_char),
            terminated(tag_no_case("EXISTS"), Self::keyword_follow_char),
            terminated(tag_no_case("EXPLAIN"), Self::keyword_follow_char),
            terminated(tag_no_case("FAIL"), Self::keyword_follow_char),
            terminated(tag_no_case("FOR"), Self::keyword_follow_char),
            terminated(tag_no_case("FOREIGN"), Self::keyword_follow_char),
            terminated(tag_no_case("FROM"), Self::keyword_follow_char),
            terminated(tag_no_case("FULL"), Self::keyword_follow_char),
            terminated(tag_no_case("FULLTEXT"), Self::keyword_follow_char),
            terminated(tag_no_case("GLOB"), Self::keyword_follow_char),
            terminated(tag_no_case("GROUP"), Self::keyword_follow_char),
            terminated(tag_no_case("HAVING"), Self::keyword_follow_char),
            terminated(tag_no_case("IF"), Self::keyword_follow_char),
            terminated(tag_no_case("IGNORE"), Self::keyword_follow_char),
            terminated(tag_no_case("IMMEDIATE"), Self::keyword_follow_char),
            terminated(tag_no_case("IN"), Self::keyword_follow_char),
            terminated(tag_no_case("INDEX"), Self::keyword_follow_char),
            terminated(tag_no_case("INDEXED"), Self::keyword_follow_char),
            terminated(tag_no_case("INITIALLY"), Self::keyword_follow_char),
        ))(i)
    }

    fn keywords_part_4(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        alt((
            terminated(tag_no_case("INNER"), Self::keyword_follow_char),
            terminated(tag_no_case("INSERT"), Self::keyword_follow_char),
            terminated(tag_no_case("INSTEAD"), Self::keyword_follow_char),
            terminated(tag_no_case("INTERSECT"), Self::keyword_follow_char),
            terminated(tag_no_case("INTO"), Self::keyword_follow_char),
            terminated(tag_no_case("IS"), Self::keyword_follow_char),
            terminated(tag_no_case("ISNULL"), Self::keyword_follow_char),
            terminated(tag_no_case("ORDER"), Self::keyword_follow_char),
            terminated(tag_no_case("JOIN"), Self::keyword_follow_char),
            terminated(tag_no_case("KEY"), Self::keyword_follow_char),
            terminated(tag_no_case("LEFT"), Self::keyword_follow_char),
            terminated(tag_no_case("LIKE"), Self::keyword_follow_char),
            terminated(tag_no_case("LIMIT"), Self::keyword_follow_char),
            terminated(tag_no_case("MATCH"), Self::keyword_follow_char),
            terminated(tag_no_case("NATURAL"), Self::keyword_follow_char),
            terminated(tag_no_case("NO"), Self::keyword_follow_char),
            terminated(tag_no_case("NOT"), Self::keyword_follow_char),
            terminated(tag_no_case("NOTNULL"), Self::keyword_follow_char),
            terminated(tag_no_case("NULL"), Self::keyword_follow_char),
            terminated(tag_no_case("OF"), Self::keyword_follow_char),
            terminated(tag_no_case("OFFSET"), Self::keyword_follow_char),
        ))(i)
    }

    fn keywords_part_5(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        alt((
            terminated(tag_no_case("ON"), Self::keyword_follow_char),
            terminated(tag_no_case("OR"), Self::keyword_follow_char),
            terminated(tag_no_case("OUTER"), Self::keyword_follow_char),
            terminated(tag_no_case("PLAN"), Self::keyword_follow_char),
            terminated(tag_no_case("PRAGMA"), Self::keyword_follow_char),
            terminated(tag_no_case("PRIMARY"), Self::keyword_follow_char),
            terminated(tag_no_case("QUERY"), Self::keyword_follow_char),
            terminated(tag_no_case("RAISE"), Self::keyword_follow_char),
            terminated(tag_no_case("RECURSIVE"), Self::keyword_follow_char),
            terminated(tag_no_case("REFERENCES"), Self::keyword_follow_char),
            terminated(tag_no_case("REGEXP"), Self::keyword_follow_char),
            terminated(tag_no_case("REINDEX"), Self::keyword_follow_char),
            terminated(tag_no_case("RELEASE"), Self::keyword_follow_char),
            terminated(tag_no_case("RENAME"), Self::keyword_follow_char),
            terminated(tag_no_case("REPLACE"), Self::keyword_follow_char),
            terminated(tag_no_case("RESTRICT"), Self::keyword_follow_char),
            terminated(tag_no_case("RIGHT"), Self::keyword_follow_char),
            terminated(tag_no_case("ROLLBACK"), Self::keyword_follow_char),
            terminated(tag_no_case("ROW"), Self::keyword_follow_char),
            terminated(tag_no_case("SAVEPOINT"), Self::keyword_follow_char),
            terminated(tag_no_case("SELECT"), Self::keyword_follow_char),
        ))(i)
    }

    fn keywords_part_6(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        alt((
            terminated(tag_no_case("SET"), Self::keyword_follow_char),
            terminated(tag_no_case("SPATIAL"), Self::keyword_follow_char),
            terminated(tag_no_case("TABLE"), Self::keyword_follow_char),
            terminated(tag_no_case("TEMP"), Self::keyword_follow_char),
            terminated(tag_no_case("TEMPORARY"), Self::keyword_follow_char),
            terminated(tag_no_case("THEN"), Self::keyword_follow_char),
            terminated(tag_no_case("TO"), Self::keyword_follow_char),
            terminated(tag_no_case("TRANSACTION"), Self::keyword_follow_char),
            terminated(tag_no_case("TRIGGER"), Self::keyword_follow_char),
            terminated(tag_no_case("UNION"), Self::keyword_follow_char),
            terminated(tag_no_case("UNIQUE"), Self::keyword_follow_char),
            terminated(tag_no_case("UPDATE"), Self::keyword_follow_char),
            terminated(tag_no_case("USING"), Self::keyword_follow_char),
            terminated(tag_no_case("VACUUM"), Self::keyword_follow_char),
            terminated(tag_no_case("VALUES"), Self::keyword_follow_char),
            terminated(tag_no_case("VIEW"), Self::keyword_follow_char),
            terminated(tag_no_case("VIRTUAL"), Self::keyword_follow_char),
            terminated(tag_no_case("WHEN"), Self::keyword_follow_char),
            terminated(tag_no_case("WHERE"), Self::keyword_follow_char),
            terminated(tag_no_case("WITH"), Self::keyword_follow_char),
            terminated(tag_no_case("WITHOUT"), Self::keyword_follow_char),
        ))(i)
    }

    // Matches any SQL reserved keyword
    pub fn sql_keyword(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        alt((
            Self::keywords_part_1,
            Self::keywords_part_2,
            Self::keywords_part_3,
            Self::keywords_part_4,
            Self::keywords_part_5,
            Self::keywords_part_6,
        ))(i)
    }

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
                    not(peek(CommonParser::sql_keyword)),
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

    /// extract String quoted by `'` or `"`
    pub fn parse_quoted_string(i: &str) -> IResult<&str, String, ParseSQLError<&str>> {
        alt((
            map(delimited(tag("'"), take_until("'"), tag("'")), String::from),
            map(
                delimited(tag("\""), take_until("\""), tag("\"")),
                String::from,
            ),
        ))(i)
    }

    /// extract value from `key [=] 'value'` or `key [=] "value"`
    pub fn parse_quoted_string_value_with_key(
        i: &str,
        key: String,
    ) -> IResult<&str, String, ParseSQLError<&str>> {
        alt((
            map(
                tuple((
                    tag_no_case(key.as_str()),
                    multispace1,
                    CommonParser::parse_quoted_string,
                )),
                |(_, _, value)| value,
            ),
            map(
                tuple((
                    tag_no_case(key.as_str()),
                    multispace0,
                    tag("="),
                    multispace0,
                    CommonParser::parse_quoted_string,
                )),
                |(_, _, _, _, value)| value,
            ),
        ))(i)
    }

    /// extract value from `key [=] value`
    pub fn parse_string_value_with_key(
        i: &str,
        key: String,
    ) -> IResult<&str, String, ParseSQLError<&str>> {
        alt((
            map(
                tuple((tag_no_case(key.as_str()), multispace1, Self::sql_identifier)),
                |(_, _, value)| String::from(value),
            ),
            map(
                tuple((
                    tag_no_case(key.as_str()),
                    multispace0,
                    tag("="),
                    multispace0,
                    Self::sql_identifier,
                )),
                |(_, _, _, _, value)| String::from(value),
            ),
        ))(i)
    }

    /// extract value from `key [=] value`
    pub fn parse_digit_value_with_key(
        i: &str,
        key: String,
    ) -> IResult<&str, String, ParseSQLError<&str>> {
        alt((
            map(
                tuple((tag_no_case(key.as_str()), multispace1, Self::sql_identifier)),
                |(_, _, value)| String::from(value),
            ),
            map(
                tuple((
                    tag_no_case(key.as_str()),
                    multispace0,
                    tag("="),
                    multispace0,
                    digit1,
                )),
                |(_, _, _, _, value)| String::from(value),
            ),
        ))(i)
    }

    /// extract value from `key [=] {DEFAULT | 0 | 1}`
    pub fn parse_default_value_with_key(
        i: &str,
        key: String,
    ) -> IResult<&str, DefaultOrZeroOrOne, ParseSQLError<&str>> {
        alt((
            map(
                tuple((
                    tag_no_case(key.as_str()),
                    multispace1,
                    DefaultOrZeroOrOne::parse,
                )),
                |(_, _, value)| value,
            ),
            map(
                tuple((
                    tag_no_case(key.as_str()),
                    multispace0,
                    tag("="),
                    multispace0,
                    DefaultOrZeroOrOne::parse,
                )),
                |(_, _, _, _, value)| value,
            ),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use nom::bytes::complete::tag;
    use nom::IResult;

    use base::CommonParser;

    #[test]
    fn parse_sql_identifiers() {
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
    fn parse_opt_delimited() {
        assert_eq!(test_opt_delimited_fn_call("abc"), Ok(("", "abc")));
        assert_eq!(test_opt_delimited_fn_call("(abc)"), Ok(("", "abc")));
        assert!(test_opt_delimited_fn_call("(abc").is_err());
        assert_eq!(test_opt_delimited_fn_call("abc)"), Ok((")", "abc")));
        assert!(test_opt_delimited_fn_call("ab").is_err());
    }

    #[test]
    fn parse_comment() {
        let res = CommonParser::parse_comment(" COMMENT 'test'");
        assert_eq!(res.unwrap().1, "test");

        let res = CommonParser::parse_comment(" COMMENT \"test\"");
        assert_eq!(res.unwrap().1, "test");
    }

    #[test]
    fn parse_statement_terminator() {
        let res = CommonParser::statement_terminator("   ;  ");
        assert_eq!(res, Ok(("", ())));
    }
}
