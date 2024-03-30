use core::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{anychar, digit1, multispace0, multispace1};
use nom::combinator::{map, map_res, opt, recognize};
use nom::error::{Error, ParseError, VerboseError};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

use common::column::{Column, ColumnConstraint, ColumnSpecification, MySQLColumnPosition};
use common::{Literal, Real, SqlDataType};
use common_parsers::{
    column_identifier_without_alias, parse_comment, sql_identifier, type_identifier, ws_sep_comma,
};
use common_statement::index_option::{index_option, IndexOption};

pub mod index_option;
pub mod table_option;

// TODO support partition
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PartitionDefinition {}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AlgorithmOption {
    Default,
    Inplace,
    Copy,
}

/// algorithm_option:
///     ALGORITHM [=] {DEFAULT | INPLACE | COPY}
pub fn algorithm_option(i: &str) -> IResult<&str, AlgorithmOption, VerboseError<&str>> {
    map(
        tuple((
            tag_no_case("ALGORITHM"),
            multispace0,
            opt(tag("=")),
            multispace0,
            alt((
                map(tag_no_case("DEFAULT"), |_| AlgorithmOption::Default),
                map(tag_no_case("INPLACE"), |_| AlgorithmOption::Inplace),
                map(tag_no_case("COPY"), |_| AlgorithmOption::Copy),
            )),
        )),
        |x| x.4,
    )(i)
}

/// LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum LockType {
    DEFAULT,
    NONE,
    SHARED,
    EXCLUSIVE,
}

/// lock_option:
///     LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
pub fn lock_option(i: &str) -> IResult<&str, LockType, VerboseError<&str>> {
    map(
        tuple((
            tag_no_case("LOCK "),
            multispace0,
            opt(tag("= ")),
            multispace0,
            alt((
                map(tag_no_case("DEFAULT"), |_| LockType::DEFAULT),
                map(tag_no_case("NONE"), |_| LockType::NONE),
                map(tag_no_case("SHARED"), |_| LockType::SHARED),
                map(tag_no_case("EXCLUSIVE"), |_| LockType::EXCLUSIVE),
            )),
            multispace0,
        )),
        |x| x.4,
    )(i)
}

/// [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    Full,
    Partial,
    Simple,
}

/// [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
fn match_type(i: &str) -> IResult<&str, MatchType, VerboseError<&str>> {
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ReferenceDefinition {
    tbl_name: String,
    key_part: Vec<KeyPart>,
    match_type: Option<MatchType>,
    on_delete: Option<ReferenceOption>,
    on_update: Option<ReferenceOption>,
}

/// reference_option:
///     RESTRICT | CASCADE | SET NULL | NO ACTION | SET DEFAULT
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ReferenceOption {
    Restrict,
    Cascade,
    SetNull,
    NoAction,
    SetDefault,
}

/// reference_definition:
///     REFERENCES tbl_name (key_part,...)
///       [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
///       [ON DELETE reference_option]
///       [ON UPDATE reference_option]
pub fn reference_definition(i: &str) -> IResult<&str, ReferenceDefinition, VerboseError<&str>> {
    let opt_on_delete = opt(map(
        tuple((
            tag_no_case("ON"),
            multispace1,
            tag_no_case("DELETE"),
            multispace1,
            reference_option,
        )),
        |x| x.4,
    ));
    let opt_on_update = opt(map(
        tuple((
            tag_no_case("ON"),
            multispace1,
            tag_no_case("UPDATE"),
            multispace1,
            reference_option,
        )),
        |x| x.4,
    ));
    map(
        tuple((
            tuple((multispace0, tag_no_case("REFERENCES"), multispace1)),
            // tbl_name
            map(sql_identifier, |x| String::from(x)),
            multispace0,
            key_part, // (key_part,...)
            multispace0,
            opt(match_type), // [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
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

/// reference_option:
///     RESTRICT | CASCADE | SET NULL | NO ACTION | SET DEFAULT
pub fn reference_option(i: &str) -> IResult<&str, ReferenceOption, VerboseError<&str>> {
    alt((
        map(tag_no_case("RESTRICT"), |_| ReferenceOption::Restrict),
        map(tag_no_case("CASCADE"), |_| ReferenceOption::Cascade),
        map(
            tuple((tag_no_case("SET"), multispace1, tag_no_case("NULL"))),
            |_| ReferenceOption::SetNull,
        ),
        map(
            tuple((tag_no_case("NO"), multispace1, tag_no_case("ACTION"))),
            |_| ReferenceOption::NoAction,
        ),
        map(
            tuple((tag_no_case("SET"), multispace1, tag_no_case("DEFAULT"))),
            |_| ReferenceOption::SetDefault,
        ),
    ))(i)
}

/// {'ZLIB' | 'LZ4' | 'NONE'}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CompressionType {
    ZLIB,
    LZ4,
    NONE,
}

/// { NO | FIRST | LAST }
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum InsertMethodType {
    NO,
    FIRST,
    LAST,
}

/// {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum RowFormatType {
    DEFAULT,
    DYNAMIC,
    FIXED,
    COMPRESSED,
    REDUNDANT,
    COMPACT,
}

/// {DEFAULT | 0 | 1}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum DefaultOrZeroOrOne {
    Default,
    Zero,
    One,
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

/// {FULLTEXT | SPATIAL}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FulltextOrSpatialType {
    FULLTEXT,
    SPATIAL,
}

/// {INDEX | KEY}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IndexOrKeyType {
    INDEX,
    KEY,
}

/// [CONSTRAINT [symbol]] CHECK (expr) [[NOT] ENFORCED]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CheckConstraintDefinition {
    pub symbol: Option<String>,
    pub expr: String,
    pub enforced: bool,
}

/////////////////////////////////////
// {VISIBLE | INVISIBLE}
/////////////////////////////////////

/// {VISIBLE | INVISIBLE}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum VisibleType {
    VISIBLE,
    INVISIBLE,
}

pub fn visible_or_invisible(i: &str) -> IResult<&str, VisibleType, VerboseError<&str>> {
    alt((
        map(tag_no_case("VISIBLE"), |_| VisibleType::VISIBLE),
        map(tag_no_case("INVISIBLE"), |_| VisibleType::INVISIBLE),
    ))(i)
}

/// {col_name [(length)] | (expr)}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum KeyPartType {
    ColumnNameWithLength(String, Option<usize>),
    Expr(String),
}

/////////////////////////////////////
// order
/////////////////////////////////////

/// [ASC | DESC]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Asc,
    Desc,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            OrderType::Asc => write!(f, "ASC"),
            OrderType::Desc => write!(f, "DESC"),
        }
    }
}

/////////////////////////////////////
// index_type
/////////////////////////////////////

/// USING {BTREE | HASH}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IndexType {
    BTREE,
    HASH,
}

impl fmt::Display for IndexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexType::BTREE => write!(f, " USING BTREE")?,
            IndexType::HASH => write!(f, " USING HASH")?,
        };
        Ok(())
    }
}

/// index_type:
///    USING {BTREE | HASH}
pub fn index_type(i: &str) -> IResult<&str, IndexType, VerboseError<&str>> {
    map(
        tuple((
            tag_no_case("USING"),
            multispace1,
            alt((
                map(tag_no_case("BTREE"), |_| IndexType::BTREE),
                map(tag_no_case("HASH"), |_| IndexType::HASH),
            )),
            multispace0,
        )),
        |x| x.2,
    )(i)
}

/////////////////////////////////////
// key_part
/////////////////////////////////////

/// key_part: {col_name [(length)] | (expr)} [ASC | DESC]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct KeyPart {
    r#type: KeyPartType,
    order: Option<OrderType>,
}

/// [index_name]
pub fn opt_index_name(i: &str) -> IResult<&str, Option<String>, VerboseError<&str>> {
    opt(map(
        delimited(multispace1, sql_identifier, multispace0),
        |(x)| String::from(x),
    ))(i)
}

/// [index_type]
/// USING {BTREE | HASH}
pub fn opt_index_type(i: &str) -> IResult<&str, Option<IndexType>, VerboseError<&str>> {
    opt(map(delimited(multispace1, index_type, multispace0), |x| x))(i)
}

/// (key_part,...)
/// key_part: {col_name [(length)] | (expr)} [ASC | DESC]
pub fn key_part(i: &str) -> IResult<&str, Vec<KeyPart>, VerboseError<&str>> {
    map(
        tuple((
            multispace0,
            delimited(
                tag("("),
                delimited(multispace0, key_part_list, multispace0),
                tag(")"),
            ),
        )),
        |(_, val)| val,
    )(i)
}

/// [index_option]
/// index_option: {
///     KEY_BLOCK_SIZE [=] value
///   | index_type
///   | WITH PARSER parser_name
///   | COMMENT 'string'
///   | {VISIBLE | INVISIBLE}
///   |ENGINE_ATTRIBUTE [=] 'string'
///   |SECONDARY_ENGINE_ATTRIBUTE [=] 'string'
/// }
pub fn opt_index_option(i: &str) -> IResult<&str, Option<IndexOption>, VerboseError<&str>> {
    opt(map(preceded(multispace1, index_option), |x| x))(i)
}

/// (key_part,...)
pub fn key_part_list(i: &str) -> IResult<&str, Vec<KeyPart>, VerboseError<&str>> {
    many1(map(terminated(key_part_item, opt(ws_sep_comma)), |e| e))(i)
}

/// key_part: {col_name [(length)] | (expr)} [ASC | DESC]
pub fn key_part_item(i: &str) -> IResult<&str, KeyPart, VerboseError<&str>> {
    let col_with_length = tuple((
        multispace0,
        sql_identifier,
        multispace0,
        opt(delimited(
            tag("("),
            map_res(digit1, |digit_str: &str| digit_str.parse::<usize>()),
            tag(")"),
        )),
    ));
    let expr = preceded(
        multispace1,
        delimited(tag("("), recognize(many1(anychar)), tag(")")),
    );

    map(
        tuple((
            alt((
                map(col_with_length, |(_, col_name, _, length)| {
                    KeyPartType::ColumnNameWithLength(String::from(col_name), length)
                }),
                map(expr, |expr| KeyPartType::Expr(String::from(expr))),
            )),
            opt(map(
                tuple((multispace1, order_type, multispace0)),
                |(_, order, _)| order,
            )),
        )),
        |(r#type, order)| KeyPart { r#type, order },
    )(i)
}

/// {INDEX | KEY}
pub fn index_or_key_type(i: &str) -> IResult<&str, IndexOrKeyType, VerboseError<&str>> {
    alt((
        map(tag_no_case("KEY"), |_| IndexOrKeyType::KEY),
        map(tag_no_case("INDEX"), |_| IndexOrKeyType::INDEX),
    ))(i)
}

/// // {FULLTEXT | SPATIAL}
pub fn fulltext_or_spatial_type(
    i: &str,
) -> IResult<&str, FulltextOrSpatialType, VerboseError<&str>> {
    alt((
        map(tag_no_case("FULLTEXT"), |_| FulltextOrSpatialType::FULLTEXT),
        map(tag_no_case("SPATIAL"), |_| FulltextOrSpatialType::SPATIAL),
    ))(i)
}

pub fn index_col_list(i: &str) -> IResult<&str, Vec<Column>, VerboseError<&str>> {
    many0(map(
        terminated(index_col_name, opt(ws_sep_comma)),
        // XXX(malte): ignores length and order
        |e| e.0,
    ))(i)
}

pub fn index_col_name(
    i: &str,
) -> IResult<&str, (Column, Option<u16>, Option<OrderType>), VerboseError<&str>> {
    let (remaining_input, (column, len_u8, order)) = tuple((
        terminated(column_identifier_without_alias, multispace0),
        opt(delimited(tag("("), digit1, tag(")"))),
        opt(order_type),
    ))(i)?;
    let len = len_u8.map(|l| u16::from_str(l).unwrap());

    Ok((remaining_input, (column, len, order)))
}

pub fn order_type(i: &str) -> IResult<&str, OrderType, VerboseError<&str>> {
    alt((
        map(tag_no_case("DESC"), |_| OrderType::Desc),
        map(tag_no_case("ASC"), |_| OrderType::Asc),
    ))(i)
}

pub fn parse_position(i: &str) -> IResult<&str, MySQLColumnPosition, VerboseError<&str>> {
    let mut parser = alt((
        map(
            tuple((multispace0, tag_no_case("FIRST"), multispace0)),
            |_| MySQLColumnPosition::First,
        ),
        map(
            tuple((
                multispace0,
                tag_no_case("AFTER"),
                multispace1,
                sql_identifier,
            )),
            |(_, _, _, identifier)| MySQLColumnPosition::After(String::from(identifier).into()),
        ),
    ));
    let (remaining_input, position) = parser(i)?;
    Ok((remaining_input, position))
}

pub fn single_column_definition(i: &str) -> IResult<&str, ColumnSpecification, VerboseError<&str>> {
    map(
        tuple((
            column_identifier_without_alias,
            opt(delimited(multispace1, type_identifier, multispace0)),
            many0(column_constraint),
            opt(parse_comment),
            opt(parse_position),
            opt(ws_sep_comma),
        )),
        |(column, field_type, constraints, comment, position, _)| {
            let sql_type = match field_type {
                None => SqlDataType::Text,
                Some(ref t) => t.clone(),
            };
            ColumnSpecification {
                column,
                sql_type,
                constraints: constraints.into_iter().filter_map(|m| m).collect(),
                comment,
                position,
            }
        },
    )(i)
}

fn column_constraint(i: &str) -> IResult<&str, Option<ColumnConstraint>, VerboseError<&str>> {
    let not_null = map(
        delimited(multispace0, tag_no_case("NOT NULL"), multispace0),
        |_| Some(ColumnConstraint::NotNull),
    );
    let null = map(
        delimited(multispace0, tag_no_case("NULL"), multispace0),
        |_| Some(ColumnConstraint::Null),
    );
    let auto_increment = map(
        delimited(multispace0, tag_no_case("AUTO_INCREMENT"), multispace0),
        |_| Some(ColumnConstraint::AutoIncrement),
    );
    let primary_key = map(
        delimited(multispace0, tag_no_case("PRIMARY KEY"), multispace0),
        |_| Some(ColumnConstraint::PrimaryKey),
    );
    let unique = map(
        delimited(multispace0, tag_no_case("UNIQUE"), multispace0),
        |_| Some(ColumnConstraint::Unique),
    );
    let character_set = map(
        preceded(
            delimited(multispace0, tag_no_case("CHARACTER SET"), multispace1),
            sql_identifier,
        ),
        |cs| {
            let char_set = cs.to_owned();
            Some(ColumnConstraint::CharacterSet(char_set))
        },
    );
    let collate = map(
        preceded(
            delimited(multispace0, tag_no_case("COLLATE"), multispace1),
            sql_identifier,
        ),
        |c| {
            let collation = c.to_owned();
            Some(ColumnConstraint::Collation(collation))
        },
    );

    alt((
        not_null,
        null,
        auto_increment,
        default,
        primary_key,
        unique,
        character_set,
        collate,
    ))(i)
}

fn default(i: &str) -> IResult<&str, Option<ColumnConstraint>, VerboseError<&str>> {
    let (remaining_input, (_, _, _, def, _)) = tuple((
        multispace0,
        tag_no_case("DEFAULT"),
        multispace1,
        alt((
            map(delimited(tag("'"), take_until("'"), tag("'")), |s| {
                Literal::String(String::from(s))
            }),
            fixed_point,
            map(digit1, |d: &str| {
                let d_i64 = d.parse().unwrap();
                Literal::Integer(d_i64)
            }),
            map(tag("''"), |_| Literal::String(String::from(""))),
            map(tag_no_case("NULL"), |_| Literal::Null),
            map(tag_no_case("CURRENT_TIMESTAMP"), |_| {
                Literal::CurrentTimestamp
            }),
        )),
        multispace0,
    ))(i)?;

    Ok((remaining_input, Some(ColumnConstraint::DefaultValue(def))))
}

fn fixed_point(i: &str) -> IResult<&str, Literal, VerboseError<&str>> {
    let (remaining_input, (i, _, f)) = tuple((digit1, tag("."), digit1))(i)?;

    Ok((
        remaining_input,
        Literal::FixedPoint(Real {
            integral: i32::from_str(i).unwrap(),
            fractional: i32::from_str(f).unwrap(),
        }),
    ))
}
