/// common parsers
use core::fmt;
use std::fmt::Formatter;
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
use base::ParseSQLError;
use common::keywords::sql_keyword;
pub use common::order::{OrderClause, OrderType};

pub use self::case::{CaseWhenExpression, ColumnOrLiteral};
pub use self::join::{JoinConstraint, JoinOperator, JoinRightSide};
pub use self::key_part::{KeyPart, KeyPartType};
pub use self::partition_definition::PartitionDefinition;
pub use self::reference_definition::ReferenceDefinition;

pub mod index_option;
pub mod table_option;

pub mod arithmetic;

#[macro_use]
pub mod keywords;
mod key_part;
mod partition_definition;
mod reference_definition;

pub mod condition;

mod order;

pub mod case;

mod join;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AlgorithmType {
    Default,
    Inplace,
    Copy,
}

impl AlgorithmType {
    /// algorithm_option:
    ///     ALGORITHM [=] {DEFAULT | INPLACE | COPY}
    pub fn parse(i: &str) -> IResult<&str, AlgorithmType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ALGORITHM"),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((
                    map(tag_no_case("DEFAULT"), |_| AlgorithmType::Default),
                    map(tag_no_case("INPLACE"), |_| AlgorithmType::Inplace),
                    map(tag_no_case("COPY"), |_| AlgorithmType::Copy),
                )),
            )),
            |x| x.4,
        )(i)
    }
}

/// LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum LockType {
    Default,
    None,
    Shared,
    Exclusive,
}

impl LockType {
    /// lock_option:
    ///     LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
    pub fn parse(i: &str) -> IResult<&str, LockType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("LOCK "),
                multispace0,
                opt(tag("= ")),
                multispace0,
                alt((
                    map(tag_no_case("DEFAULT"), |_| LockType::Default),
                    map(tag_no_case("NONE"), |_| LockType::None),
                    map(tag_no_case("SHARED"), |_| LockType::Shared),
                    map(tag_no_case("EXCLUSIVE"), |_| LockType::Exclusive),
                )),
                multispace0,
            )),
            |x| x.4,
        )(i)
    }
}

/// [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    Full,
    Partial,
    Simple,
}

impl MatchType {
    /// [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
    fn parse(i: &str) -> IResult<&str, MatchType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("MATCH"),
                multispace1,
                alt((
                    map(tag_no_case("FULL"), |_| MatchType::Full),
                    map(tag_no_case("PARTIAL"), |_| MatchType::Partial),
                    map(tag_no_case("SIMPLE"), |_| MatchType::Simple),
                )),
            )),
            |x| x.2,
        )(i)
    }
}

/// reference_option:
///     RESTRICT | CASCADE | SET NULL | NO ACTION | SET DEFAULT
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ReferenceType {
    Restrict,
    Cascade,
    SetNull,
    NoAction,
    SetDefault,
}

impl ReferenceType {
    /// reference_option:
    ///     RESTRICT | CASCADE | SET NULL | NO ACTION | SET DEFAULT
    pub fn parse(i: &str) -> IResult<&str, ReferenceType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("RESTRICT"), |_| ReferenceType::Restrict),
            map(tag_no_case("CASCADE"), |_| ReferenceType::Cascade),
            map(
                tuple((tag_no_case("SET"), multispace1, tag_no_case("NULL"))),
                |_| ReferenceType::SetNull,
            ),
            map(
                tuple((tag_no_case("NO"), multispace1, tag_no_case("ACTION"))),
                |_| ReferenceType::NoAction,
            ),
            map(
                tuple((tag_no_case("SET"), multispace1, tag_no_case("DEFAULT"))),
                |_| ReferenceType::SetDefault,
            ),
        ))(i)
    }
}

/// {'ZLIB' | 'LZ4' | 'NONE'}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CompressionType {
    ZLIB,
    LZ4,
    NONE,
}

impl CompressionType {
    fn parse(i: &str) -> IResult<&str, CompressionType, ParseSQLError<&str>> {
        alt((
            map(
                delimited(
                    alt((tag("'"), tag("\""))),
                    tag_no_case("ZLIB"),
                    alt((tag("'"), tag("\""))),
                ),
                |_| CompressionType::ZLIB,
            ),
            map(
                delimited(
                    alt((tag("'"), tag("\""))),
                    tag_no_case("LZ4"),
                    alt((tag("'"), tag("\""))),
                ),
                |_| CompressionType::LZ4,
            ),
            map(
                delimited(
                    alt((tag("'"), tag("\""))),
                    tag_no_case("NONE"),
                    alt((tag("'"), tag("\""))),
                ),
                |_| CompressionType::NONE,
            ),
        ))(i)
    }
}

/// { NO | FIRST | LAST }
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum InsertMethodType {
    No,
    First,
    Last,
}

impl InsertMethodType {
    fn parse(i: &str) -> IResult<&str, InsertMethodType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("NO"), |_| InsertMethodType::No),
            map(tag_no_case("FIRST"), |_| InsertMethodType::First),
            map(tag_no_case("LAST"), |_| InsertMethodType::Last),
        ))(i)
    }
}

/// {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum RowFormatType {
    Default,
    Dynamic,
    Fixed,
    Compressed,
    Redundant,
    Compact,
}

impl RowFormatType {
    fn parse(i: &str) -> IResult<&str, RowFormatType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("DEFAULT"), |_| RowFormatType::Default),
            map(tag_no_case("DYNAMIC"), |_| RowFormatType::Dynamic),
            map(tag_no_case("FIXED"), |_| RowFormatType::Fixed),
            map(tag_no_case("COMPRESSED"), |_| RowFormatType::Compressed),
            map(tag_no_case("REDUNDANT"), |_| RowFormatType::Redundant),
            map(tag_no_case("COMPACT"), |_| RowFormatType::Compact),
        ))(i)
    }
}

/// {DEFAULT | 0 | 1}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum DefaultOrZeroOrOne {
    Default,
    Zero,
    One,
}

impl DefaultOrZeroOrOne {
    pub fn parse(i: &str) -> IResult<&str, DefaultOrZeroOrOne, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("0"), |_| DefaultOrZeroOrOne::Zero),
            map(tag_no_case("1"), |_| DefaultOrZeroOrOne::One),
            map(tag_no_case("DEFAULT"), |_| DefaultOrZeroOrOne::Default),
        ))(i)
    }
}

impl fmt::Display for DefaultOrZeroOrOne {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DefaultOrZeroOrOne::Default => write!(f, "DEFAULT")?,
            DefaultOrZeroOrOne::Zero => write!(f, "1")?,
            DefaultOrZeroOrOne::One => write!(f, "0")?,
        }
        Ok(())
    }
}

/// STORAGE {DISK | MEMORY}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TablespaceType {
    StorageDisk,
    StorageMemory,
}

impl TablespaceType {
    pub fn parse(i: &str) -> IResult<&str, TablespaceType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("DISK"), |_| TablespaceType::StorageDisk),
            map(tag_no_case("MEMORY"), |_| TablespaceType::StorageMemory),
        ))(i)
    }
}

/// {FULLTEXT | SPATIAL}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FulltextOrSpatialType {
    Fulltext,
    Spatial,
}

impl FulltextOrSpatialType {
    /// // {FULLTEXT | SPATIAL}
    pub fn parse(i: &str) -> IResult<&str, FulltextOrSpatialType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("FULLTEXT"), |_| FulltextOrSpatialType::Fulltext),
            map(tag_no_case("SPATIAL"), |_| FulltextOrSpatialType::Spatial),
        ))(i)
    }
}

/// {INDEX | KEY}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IndexOrKeyType {
    Index,
    Key,
}

impl IndexOrKeyType {
    /// {INDEX | KEY}
    pub fn parse(i: &str) -> IResult<&str, IndexOrKeyType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("KEY"), |_| IndexOrKeyType::Key),
            map(tag_no_case("INDEX"), |_| IndexOrKeyType::Index),
        ))(i)
    }
}

/// [CONSTRAINT [symbol]] CHECK (expr) [[NOT] ENFORCED]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CheckConstraintDefinition {
    pub symbol: Option<String>,
    pub expr: String,
    pub enforced: bool,
}

/// {VISIBLE | INVISIBLE}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum VisibleType {
    Visible,
    Invisible,
}

impl VisibleType {
    pub fn parse(i: &str) -> IResult<&str, VisibleType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("VISIBLE"), |_| VisibleType::Visible),
            map(tag_no_case("INVISIBLE"), |_| VisibleType::Invisible),
        ))(i)
    }
}

/// USING {BTREE | HASH}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IndexType {
    Btree,
    Hash,
}

impl IndexType {
    fn parse(i: &str) -> IResult<&str, IndexType, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("USING"),
                multispace1,
                alt((
                    map(tag_no_case("BTREE"), |_| IndexType::Btree),
                    map(tag_no_case("HASH"), |_| IndexType::Hash),
                )),
            )),
            |x| x.2,
        )(i)
    }

    /// [index_type]
    /// USING {BTREE | HASH}
    pub fn opt_index_type(i: &str) -> IResult<&str, Option<IndexType>, ParseSQLError<&str>> {
        opt(map(
            delimited(multispace1, IndexType::parse, multispace0),
            |x| x,
        ))(i)
    }
}

impl fmt::Display for IndexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexType::Btree => write!(f, " USING BTREE")?,
            IndexType::Hash => write!(f, " USING HASH")?,
        };
        Ok(())
    }
}

/// [index_name]
pub fn opt_index_name(i: &str) -> IResult<&str, Option<String>, ParseSQLError<&str>> {
    opt(map(
        delimited(multispace1, sql_identifier, multispace0),
        |(x)| String::from(x),
    ))(i)
}

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
pub(crate) fn opt_delimited<I: Clone, O1, O2, O3, E: ParseError<I>, F, G, H>(
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
    delimited(tag("("), precision_helper, tag(")"))(i)
}

pub fn delim_digit(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
    delimited(tag("("), digit1, tag(")"))(i)
}

pub fn sql_identifier(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
    alt((
        alt((
            preceded(
                not(peek(sql_keyword)),
                recognize(pair(alpha1, take_while(is_sql_identifier))),
            ),
            recognize(pair(tag("_"), take_while1(is_sql_identifier))),
            // variable only
            recognize(pair(tag("@"), take_while1(is_sql_identifier))),
        )),
        delimited(tag("`"), take_while1(is_sql_identifier), tag("`")),
        delimited(tag("["), take_while1(is_sql_identifier), tag("]")),
    ))(i)
}

// Parse an unsigned integer.
pub fn unsigned_number(i: &str) -> IResult<&str, u64, ParseSQLError<&str>> {
    map(digit1, |d| FromStr::from_str(d).unwrap())(i)
}

pub(crate) fn eof<I: Copy + InputLength, E: ParseError<I>>(input: I) -> IResult<I, I, E> {
    if input.input_len() == 0 {
        Ok((input, input))
    } else {
        Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Eof)))
    }
}

// Parse a terminator that ends a SQL statement.
pub fn statement_terminator(i: &str) -> IResult<&str, (), ParseSQLError<&str>> {
    let (remaining_input, _) =
        delimited(multispace0, alt((tag(";"), line_ending, eof)), multispace0)(i)?;
    Ok((remaining_input, ()))
}

// Parse rule for AS-based aliases for SQL entities.
pub fn as_alias(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
    map(
        tuple((
            multispace1,
            opt(pair(tag_no_case("as"), multispace1)),
            sql_identifier,
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
            |comment| String::from(comment),
        ),
        map(
            preceded(
                delimited(multispace0, tag_no_case("COMMENT"), multispace1),
                delimited(tag("\""), take_until("\""), tag("\"")),
            ),
            |comment| String::from(comment),
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

#[cfg(test)]
mod tests {
    use nom::bytes::complete::tag;
    use nom::IResult;

    use common::{opt_delimited, parse_comment, sql_identifier, statement_terminator};

    #[test]
    fn sql_identifiers() {
        let id1 = "foo";
        let id2 = "f_o_o";
        let id3 = "foo12";
        let id4 = ":fo oo";
        let id5 = "primary ";
        let id6 = "`primary`";

        assert!(sql_identifier(id1).is_ok());
        assert!(sql_identifier(id2).is_ok());
        assert!(sql_identifier(id3).is_ok());
        assert!(sql_identifier(id4).is_err());
        assert!(sql_identifier(id5).is_err());
        assert!(sql_identifier(id6).is_ok());
    }

    fn test_opt_delimited_fn_call(i: &str) -> IResult<&str, &str> {
        opt_delimited(tag("("), tag("abc"), tag(")"))(i)
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
        let res = parse_comment(" COMMENT 'test'");
        assert_eq!(res.unwrap().1, "test");
    }

    #[test]
    fn terminated_by_semicolon() {
        let res = statement_terminator("   ;  ");
        assert_eq!(res, Ok(("", ())));
    }
}
