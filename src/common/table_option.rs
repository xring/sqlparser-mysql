use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{digit1, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, tuple};
use nom::{IResult, Parser};

use base::column::Column;
use base::error::ParseSQLError;
use common::sql_identifier;
use common::{
    CompressionType, DefaultOrZeroOrOne, InsertMethodType, RowFormatType, TablespaceType,
};

/// table_option: {
///     AUTOEXTEND_SIZE [=] value
///   | AUTO_INCREMENT [=] value
///   | AVG_ROW_LENGTH [=] value
///   | [DEFAULT] CHARACTER SET [=] charset_name
///   | CHECKSUM [=] {0 | 1}
///   | [DEFAULT] COLLATE [=] collation_name
///   | COMMENT [=] 'string'
///   | COMPRESSION [=] {'ZLIB' | 'LZ4' | 'NONE'}
///   | CONNECTION [=] 'connect_string'
///   | {DATA | INDEX} DIRECTORY [=] 'absolute path to directory'
///   | DELAY_KEY_WRITE [=] {0 | 1}
///   | ENCRYPTION [=] {'Y' | 'N'}
///   | ENGINE [=] engine_name
///   | ENGINE_ATTRIBUTE [=] 'string'
///   | INSERT_METHOD [=] { NO | FIRST | LAST }
///   | KEY_BLOCK_SIZE [=] value
///   | MAX_ROWS [=] value
///   | MIN_ROWS [=] value
///   | PACK_KEYS [=] {0 | 1 | DEFAULT}
///  | PASSWORD [=] 'string'
///   | ROW_FORMAT [=] {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}
///   | SECONDARY_ENGINE_ATTRIBUTE [=] 'string'
///   | STATS_AUTO_RECALC [=] {DEFAULT | 0 | 1}
///   | STATS_PERSISTENT [=] {DEFAULT | 0 | 1}
///   | STATS_SAMPLE_PAGES [=] value
///   | TABLESPACE tablespace_name [STORAGE {DISK | MEMORY}]
///   | UNION [=] (tbl_name[,tbl_name]...)
///  }
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TableOption {
    AutoextendSize(u64),
    AutoIncrement(u64),
    AvgRowLength(u64),
    DefaultCharacterSet(String),
    Checksum(u8),
    DefaultCollate(String),
    Comment(String),
    Compression(CompressionType),
    Connection(String),
    DataDirectory(String),
    IndexDirectory(String),
    DelayKeyWrite(u8),
    Encryption(bool),
    Engine(String),
    EngineAttribute(String),
    InsertMethod(InsertMethodType),
    KeyBlockSize(u64),
    MaxRows(u64),
    MinRows(u64),
    PackKeys(DefaultOrZeroOrOne),
    Password(String),
    RowFormat(RowFormatType),
    StartTransaction, // create table only
    SecondaryEngineAttribute(String),
    StatsAutoRecalc(DefaultOrZeroOrOne),
    StatsPersistent(DefaultOrZeroOrOne),
    StatsSamplePages(u64),
    Tablespace(String, Option<TablespaceType>),
    Union(Vec<String>),
}

impl TableOption {
    /// table_option: {
    ///     AUTOEXTEND_SIZE [=] value
    ///   | AUTO_INCREMENT [=] value
    ///   | AVG_ROW_LENGTH [=] value
    ///   | [DEFAULT] CHARACTER SET [=] charset_name
    ///   | CHECKSUM [=] {0 | 1}
    ///   | [DEFAULT] COLLATE [=] collation_name
    ///   | COMMENT [=] 'string'
    ///   | COMPRESSION [=] {'ZLIB' | 'LZ4' | 'NONE'}
    ///   | CONNECTION [=] 'connect_string'
    ///   | {DATA | INDEX} DIRECTORY [=] 'absolute path to directory'
    ///   | DELAY_KEY_WRITE [=] {0 | 1}
    ///   | ENCRYPTION [=] {'Y' | 'N'}
    ///   | ENGINE [=] engine_name
    ///   | ENGINE_ATTRIBUTE [=] 'string'
    ///   | INSERT_METHOD [=] { NO | FIRST | LAST }
    ///   | KEY_BLOCK_SIZE [=] value
    ///   | MAX_ROWS [=] value
    ///   | MIN_ROWS [=] value
    ///   | PACK_KEYS [=] {0 | 1 | DEFAULT}
    ///  | PASSWORD [=] 'string'
    ///   | ROW_FORMAT [=] {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}
    ///   | SECONDARY_ENGINE_ATTRIBUTE [=] 'string'
    ///   | STATS_AUTO_RECALC [=] {DEFAULT | 0 | 1}
    ///   | STATS_PERSISTENT [=] {DEFAULT | 0 | 1}
    ///   | STATS_SAMPLE_PAGES [=] value
    ///   | TABLESPACE tablespace_name [STORAGE {DISK | MEMORY}]
    ///   | UNION [=] (tbl_name[,tbl_name]...)
    ///  }
    pub fn parse(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        alt((Self::table_option_part_1, Self::table_option_part_2))(i)
    }

    fn table_option_part_1(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        alt((
            Self::autoextend_size,
            Self::auto_increment,
            Self::avg_row_length,
            Self::default_character_set,
            Self::checksum,
            Self::default_collate,
            Self::comment,
            Self::compression,
            Self::connection,
            Self::data_directory,
            Self::index_directory,
            Self::delay_key_write,
            Self::encryption,
            Self::engine,
            Self::engine_attribute,
        ))(i)
    }

    fn table_option_part_2(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        alt((
            Self::insert_method,
            Self::key_block_size,
            Self::max_rows,
            Self::min_rows,
            Self::pack_keys,
            Self::password,
            Self::row_format,
            Self::secondary_engine_attribute,
            Self::stats_auto_recalc,
            Self::stats_persistent,
            Self::stats_sample_pages,
            Self::tablespace,
            Self::union,
        ))(i)
    }

    /// AUTOEXTEND_SIZE [=] value
    fn autoextend_size(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("AUTOEXTEND_SIZE "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
                multispace0,
            )),
            |(_, _, _, value, _, _): (&str, &str, Option<&str>, &str, &str, &str)| {
                TableOption::AutoextendSize(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// AUTO_INCREMENT [=] value
    fn auto_increment(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("AUTO_INCREMENT "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
                multispace0,
            )),
            |(_, _, _, value, _, _): (&str, &str, Option<&str>, &str, &str, &str)| {
                TableOption::AutoIncrement(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// AVG_ROW_LENGTH [=] value
    fn avg_row_length(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("AVG_ROW_LENGTH "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
                multispace0,
            )),
            |(_, _, _, value, _, _): (&str, &str, Option<&str>, &str, &str, &str)| {
                TableOption::AvgRowLength(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// [DEFAULT] CHARACTER SET [=] charset_name
    fn default_character_set(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                opt(tag_no_case("DEFAULT ")),
                multispace0,
                tuple((
                    multispace1,
                    tag_no_case("CHARACTER"),
                    multispace1,
                    tag_no_case("SET"),
                    multispace0,
                    opt(tag("=")),
                    multispace0,
                )),
                map(sql_identifier, |x| String::from(x)),
                multispace0,
            )),
            |(_, _, _, charset_name, _)| TableOption::DefaultCharacterSet(charset_name),
        )(i)
    }

    /// CHECKSUM [=] {0 | 1}
    fn checksum(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("CHECKSUM "),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((map(tag("0"), |_| 0), map(tag("1"), |_| 1))),
                multispace0,
            )),
            |x| TableOption::Checksum(x.4),
        )(i)
    }

    /// [DEFAULT] COLLATE [=] collation_name
    fn default_collate(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                opt(tag_no_case("DEFAULT ")),
                multispace0,
                tag_no_case("CHARACTER"),
                multispace1,
                map(
                    tuple((
                        multispace1,
                        tag_no_case("COLLATE"),
                        multispace1,
                        opt(tag("=")),
                        multispace0,
                        sql_identifier,
                        multispace0,
                    )),
                    |(_, _, _, _, _, collation_name, _)| String::from(collation_name),
                ),
                multispace0,
            )),
            |(_, _, _, _, collation_name, _)| TableOption::DefaultCollate(collation_name),
        )(i)
    }

    /// COMMENT [=] 'string'
    fn comment(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("COMMENT "),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
                multispace0,
            )),
            |(_, _, _, _, comment, _)| TableOption::Comment(comment),
        )(i)
    }

    /// COMPRESSION [=] {'ZLIB' | 'LZ4' | 'NONE'}
    fn compression(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("COMPRESSION "),
                multispace0,
                opt(tag("=")),
                multispace0,
                CompressionType::parse,
                multispace0,
            )),
            |(_, _, _, _, compression_type, _)| TableOption::Compression(compression_type),
        )(i)
    }

    /// CONNECTION [=] 'connect_string'
    fn connection(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("CONNECTION "),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
                multispace0,
            )),
            |(_, _, _, _, connect_string, _)| TableOption::Connection(connect_string),
        )(i)
    }

    /// {DATA | INDEX} DIRECTORY [=] 'absolute path to directory'
    fn data_directory(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("DATA"),
                multispace1,
                tag_no_case("DIRECTORY "),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
                multispace0,
            )),
            |(_, _, _, _, _, _, path, _)| TableOption::DataDirectory(path),
        )(i)
    }

    /// {DATA | INDEX} DIRECTORY [=] 'absolute path to directory'
    fn index_directory(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("INDEX"),
                multispace1,
                tag_no_case("DIRECTORY "),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
                multispace0,
            )),
            |(_, _, _, _, _, _, path, _)| TableOption::DataDirectory(path),
        )(i)
    }

    /// DELAY_KEY_WRITE [=] {0 | 1}
    fn delay_key_write(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("DELAY_KEY_RITE "),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((map(tag("0"), |_| 0), map(tag("1"), |_| 1))),
                multispace0,
            )),
            |x| TableOption::DelayKeyWrite(x.4),
        )(i)
    }

    /// ENCRYPTION [=] {'Y' | 'N'}
    fn encryption(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ENCRYPTION "),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((map(tag("'Y'"), |_| true), map(tag("'N'"), |_| false))),
                multispace0,
            )),
            |x| TableOption::Encryption(x.4),
        )(i)
    }

    /// ENGINE [=] engine_name
    fn engine(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ENGINE"),
                multispace1,
                opt(tag("=")),
                multispace0,
                sql_identifier,
                multispace0,
            )),
            |(_, _, _, _, engine, _)| TableOption::Engine(String::from(engine)),
        )(i)
    }

    /// ENGINE_ATTRIBUTE [=] 'string'
    fn engine_attribute(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ENGINE_ATTRIBUTE "),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
                multispace0,
            )),
            |(_, _, _, _, attribute, _)| TableOption::EngineAttribute(attribute),
        )(i)
    }

    /// INSERT_METHOD [=] { NO | FIRST | LAST }
    fn insert_method(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("INSERT METHOD "),
                multispace0,
                opt(tag("=")),
                multispace0,
                InsertMethodType::parse,
                multispace0,
            )),
            |x| TableOption::InsertMethod(x.4),
        )(i)
    }

    /// KEY_BLOCK_SIZE [=] value
    fn key_block_size(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("KEY_BLOCK_SIZE "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
                multispace0,
            )),
            |(_, _, _, value, _, _): (&str, &str, Option<&str>, &str, &str, &str)| {
                TableOption::KeyBlockSize(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// MAX_ROWS [=] value
    fn max_rows(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("MAX_ROWS "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
                multispace0,
            )),
            |(_, _, _, value, _, _): (&str, &str, Option<&str>, &str, &str, &str)| {
                TableOption::MaxRows(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// MIN_ROWS [=] value
    fn min_rows(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("MIN_ROWS "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
                multispace0,
            )),
            |(_, _, _, value, _, _): (&str, &str, Option<&str>, &str, &str, &str)| {
                TableOption::MinRows(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// PACK_KEYS [=] {0 | 1 | DEFAULT}
    fn pack_keys(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("INSERT_METHOD "),
                multispace0,
                opt(tag("=")),
                multispace0,
                DefaultOrZeroOrOne::parse,
                multispace0,
            )),
            |x| TableOption::PackKeys(x.4),
        )(i)
    }

    /// PASSWORD [=] 'string'
    fn password(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("PASSWORD "),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
                multispace0,
            )),
            |(_, _, _, _, password, _)| TableOption::Password(password),
        )(i)
    }

    /// ROW_FORMAT [=] {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}
    fn row_format(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("INSERT METHOD "),
                multispace0,
                opt(tag("=")),
                multispace0,
                RowFormatType::parse,
                multispace0,
            )),
            |x| TableOption::RowFormat(x.4),
        )(i)
    }

    /// START TRANSACTION
    /// create table only
    fn start_transaction(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("START"),
                multispace1,
                tag_no_case("TRANSACTION"),
            )),
            |_| TableOption::StartTransaction,
        )(i)
    }

    /// SECONDARY_ENGINE_ATTRIBUTE [=] 'string'
    fn secondary_engine_attribute(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("SECONDARY_ENGINE_ATTRIBUTE "),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
                multispace0,
            )),
            |(_, _, _, _, engine, _)| TableOption::SecondaryEngineAttribute(engine),
        )(i)
    }

    /// STATS_AUTO_RECALC [=] {DEFAULT | 0 | 1}
    fn stats_auto_recalc(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("STATS_AUTO_RECALC "),
                multispace0,
                opt(tag("=")),
                multispace0,
                DefaultOrZeroOrOne::parse,
                multispace0,
            )),
            |x| TableOption::StatsAutoRecalc(x.4),
        )(i)
    }

    /// STATS_PERSISTENT [=] {DEFAULT | 0 | 1}
    fn stats_persistent(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("STATS_PERSISTENT "),
                multispace0,
                opt(tag("=")),
                multispace0,
                DefaultOrZeroOrOne::parse,
                multispace0,
            )),
            |x| TableOption::StatsPersistent(x.4),
        )(i)
    }

    /// STATS_SAMPLE_PAGES [=] value
    fn stats_sample_pages(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("STATS_SAMPLE_PAGES "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
                multispace0,
            )),
            |(_, _, _, value, _, _): (&str, &str, Option<&str>, &str, &str, &str)| {
                TableOption::StatsSamplePages(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// TABLESPACE tablespace_name [STORAGE {DISK | MEMORY}]
    fn tablespace(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("TABLESPACE"),
                multispace1,
                map(sql_identifier, |x| String::from(x)), // tablespace_name
                multispace0,
                opt(map(
                    tuple((tag_no_case("STORAGE"), multispace0, TablespaceType::parse)),
                    |x| x.2,
                )),
                multispace0,
            )),
            |(_, _, tablespace_name, _, storage, _)| {
                TableOption::Tablespace(tablespace_name, storage)
            },
        )(i)
    }

    /// UNION [=] (tbl_name[,tbl_name]...)
    fn union(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("UNION "),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(
                    tuple((
                        multispace0,
                        delimited(
                            tag("("),
                            delimited(multispace0, Column::index_col_list, multispace0),
                            tag(")"),
                        ),
                    )),
                    |(_, value)| value.iter().map(|x| x.name.clone()).collect(),
                ),
                multispace0,
            )),
            |x| TableOption::Union(x.4),
        )(i)
    }
}
