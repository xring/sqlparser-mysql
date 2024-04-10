use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{digit1, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, tuple};
use nom::{IResult, Parser};
use std::fmt::{write, Display, Formatter};

use base::column::Column;
use base::error::ParseSQLError;
use base::{
    CommonParser, CompressionType, DefaultOrZeroOrOne, InsertMethodType, RowFormatType,
    TablespaceType,
};

/// table_option: `{
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
///   | PASSWORD [=] 'string'
///   | ROW_FORMAT [=] {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}
///   | SECONDARY_ENGINE_ATTRIBUTE [=] 'string'
///   | STATS_AUTO_RECALC [=] {DEFAULT | 0 | 1}
///   | STATS_PERSISTENT [=] {DEFAULT | 0 | 1}
///   | STATS_SAMPLE_PAGES [=] value
///   | TABLESPACE tablespace_name [STORAGE {DISK | MEMORY}]
///   | UNION [=] (tbl_name[,tbl_name]...)
///  }`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TableOption {
    AutoextendSize(u64),
    AutoIncrement(u64),
    AvgRowLength(u64),
    DefaultCharacterSet(String),
    DefaultCharset(String),
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

impl Display for TableOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            TableOption::AutoextendSize(ref val) => write!(f, "AUTOEXTEND_SIZE {}", val),
            TableOption::AutoIncrement(ref val) => write!(f, "AUTO_INCREMENT {}", val),
            TableOption::AvgRowLength(ref val) => write!(f, "AVG_ROW_LENGTH {}", val),
            TableOption::DefaultCharacterSet(ref val) => write!(f, "CHARACTER SET {}", val),
            TableOption::DefaultCharset(ref val) => write!(f, "CHARSET {}", val),
            TableOption::Checksum(ref val) => write!(f, "CHECKSUM {}", val),
            TableOption::DefaultCollate(ref val) => write!(f, "COLLATE {}", val),
            TableOption::Comment(ref val) => write!(f, "COMMENT '{}'", val),
            TableOption::Compression(ref val) => write!(f, "COMPRESSION {}", val),
            TableOption::Connection(ref val) => write!(f, "CONNECTION {}", val),
            TableOption::DataDirectory(ref val) => write!(f, "DATA DIRECTORY '{}'", val),
            TableOption::IndexDirectory(ref val) => write!(f, "INDEX DIRECTORY '{}'", val),
            TableOption::DelayKeyWrite(ref val) => write!(f, "DELAY_KEY_WRITE {}", val),
            TableOption::Encryption(ref val) => write!(f, "ENCRYPTION '{}'", val),
            TableOption::Engine(ref val) => write!(f, "ENGINE {}", val),
            TableOption::EngineAttribute(ref val) => write!(f, "ENGINE_ATTRIBUTE {}", val),
            TableOption::InsertMethod(ref val) => write!(f, "INSERT_METHOD {}", val),
            TableOption::KeyBlockSize(ref val) => write!(f, "KEY_BLOCK_SIZE {}", val),
            TableOption::MaxRows(ref val) => write!(f, "MAX_ROWS {}", val),
            TableOption::MinRows(ref val) => write!(f, "MIN_ROWS {}", val),
            TableOption::PackKeys(ref val) => write!(f, "PACK_KEYS {}", val),
            TableOption::Password(ref val) => write!(f, "PASSWORD '{}'", val),
            TableOption::RowFormat(ref val) => write!(f, "ROW_FORMAT {}", val),
            TableOption::StartTransaction => write!(f, "START TRANSACTION"),
            TableOption::SecondaryEngineAttribute(ref val) => {
                write!(f, "SECONDARY_ENGINE_ATTRIBUTE '{}'", val)
            }
            TableOption::StatsAutoRecalc(ref val) => write!(f, "STATS_AUTO_RECALC {}", val),
            TableOption::StatsPersistent(ref val) => write!(f, "STATS_PERSISTENT {}", val),
            TableOption::StatsSamplePages(ref val) => write!(f, "STATS_SAMPLE_PAGES {}", val),
            TableOption::Tablespace(ref tablespace_name, ref tbl_space_type) => {
                write!(f, "TABLESPACE {}", tablespace_name);
                if let Some(tbl_space_type) = tbl_space_type {
                    write!(f, " {}", tbl_space_type);
                }
                Ok(())
            }
            TableOption::Union(ref tbl_names) => {
                let tbl_names = tbl_names.join(",");
                write!(f, "UNION ({})", tbl_names)
            }
        }
    }
}

impl TableOption {
    pub fn parse(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        alt((Self::table_option_part_1, Self::table_option_part_2))(i)
    }

    pub fn format_list(list: &[TableOption]) -> String {
        list.iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn table_option_part_1(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        alt((
            Self::autoextend_size,
            Self::auto_increment,
            Self::avg_row_length,
            Self::default_character_set,
            Self::default_charset,
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

    /// parse `AUTOEXTEND_SIZE [=] value`
    fn autoextend_size(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("AUTOEXTEND_SIZE "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
            )),
            |(_, _, _, _, value): (&str, &str, Option<&str>, &str, &str)| {
                TableOption::AutoextendSize(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// parse `AUTO_INCREMENT [=] value`
    fn auto_increment(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("AUTO_INCREMENT"),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
            )),
            |(_, _, _, _, value): (&str, &str, Option<&str>, &str, &str)| {
                TableOption::AutoIncrement(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// parse `AVG_ROW_LENGTH [=] value`
    fn avg_row_length(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("AVG_ROW_LENGTH "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
            )),
            |(_, _, _, _, value): (&str, &str, Option<&str>, &str, &str)| {
                TableOption::AvgRowLength(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// parse `[DEFAULT] CHARACTER SET [=] charset_name`
    fn default_character_set(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                opt(tag_no_case("DEFAULT ")),
                multispace0,
                tuple((
                    tag_no_case("CHARACTER"),
                    multispace1,
                    tag_no_case("SET"),
                    multispace0,
                    opt(tag("=")),
                    multispace0,
                )),
                map(CommonParser::sql_identifier, String::from),
            )),
            |(_, _, _, charset_name)| TableOption::DefaultCharacterSet(charset_name),
        )(i)
    }

    /// parse `[DEFAULT] CHARSET [=] charset_name`
    fn default_charset(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                opt(tag_no_case("DEFAULT")),
                multispace1,
                tuple((
                    tag_no_case("CHARSET"),
                    multispace0,
                    opt(tag("=")),
                    multispace0,
                )),
                map(CommonParser::sql_identifier, String::from),
            )),
            |(_, _, _, charset_name)| TableOption::DefaultCharset(charset_name),
        )(i)
    }

    /// parse `CHECKSUM [=] {0 | 1}`
    fn checksum(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("CHECKSUM "),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((map(tag("0"), |_| 0), map(tag("1"), |_| 1))),
            )),
            |(_, _, _, _, checksum)| TableOption::Checksum(checksum),
        )(i)
    }

    /// parse `[DEFAULT] COLLATE [=] collation_name`
    fn default_collate(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                opt(tag_no_case("DEFAULT ")),
                multispace0,
                map(
                    tuple((
                        tag_no_case("COLLATE"),
                        multispace1,
                        opt(tag("=")),
                        multispace0,
                        CommonParser::sql_identifier,
                    )),
                    |(_, _, _, _, collation_name)| String::from(collation_name),
                ),
            )),
            |(_, _, collation_name)| TableOption::DefaultCollate(collation_name),
        )(i)
    }

    /// parse COMMENT [=] 'string'`
    fn comment(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("COMMENT"),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
            )),
            |(_, _, _, _, comment)| TableOption::Comment(comment),
        )(i)
    }

    /// parse `COMPRESSION [=] {'ZLIB' | 'LZ4' | 'NONE'}`
    fn compression(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(CompressionType::parse, TableOption::Compression)(i)
    }

    /// parse `CONNECTION [=] 'connect_string'`
    fn connection(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("CONNECTION"),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
            )),
            |(_, _, _, _, connect_string)| TableOption::Connection(connect_string),
        )(i)
    }

    /// parse `{DATA | INDEX} DIRECTORY [=] 'absolute path to directory'`
    fn data_directory(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("DATA"),
                multispace1,
                tag_no_case("DIRECTORY"),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(
                    alt((
                        delimited(tag("'"), take_until("'"), tag("'")),
                        delimited(tag("\""), take_until("\""), tag("\"")),
                    )),
                    String::from,
                ),
            )),
            |(_, _, _, _, _, _, path)| TableOption::DataDirectory(path),
        )(i)
    }

    /// parse `{DATA | INDEX} DIRECTORY [=] 'absolute path to directory'`
    fn index_directory(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("INDEX"),
                multispace1,
                tag_no_case("DIRECTORY"),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
            )),
            |(_, _, _, _, _, _, path)| TableOption::DataDirectory(path),
        )(i)
    }

    /// parse `DELAY_KEY_WRITE [=] {0 | 1}`
    fn delay_key_write(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("DELAY_KEY_RITE"),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((map(tag("0"), |_| 0), map(tag("1"), |_| 1))),
            )),
            |(_, _, _, _, delay_key_rite)| TableOption::DelayKeyWrite(delay_key_rite),
        )(i)
    }

    /// parse `ENCRYPTION [=] {'Y' | 'N'}`
    fn encryption(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ENCRYPTION"),
                multispace0,
                opt(tag("=")),
                multispace0,
                alt((map(tag("'Y'"), |_| true), map(tag("'N'"), |_| false))),
            )),
            |(_, _, _, _, encryption)| TableOption::Encryption(encryption),
        )(i)
    }

    /// parse `ENGINE [=] engine_name`
    fn engine(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ENGINE"),
                multispace0,
                opt(tag("=")),
                multispace0,
                CommonParser::sql_identifier,
            )),
            |(_, _, _, _, engine)| TableOption::Engine(String::from(engine)),
        )(i)
    }

    /// parse `ENGINE_ATTRIBUTE [=] 'string'`
    fn engine_attribute(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ENGINE_ATTRIBUTE"),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
            )),
            |(_, _, _, _, attribute)| TableOption::EngineAttribute(attribute),
        )(i)
    }

    /// parse `INSERT_METHOD [=] { NO | FIRST | LAST }`
    fn insert_method(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(InsertMethodType::parse, TableOption::InsertMethod)(i)
    }

    /// parse `KEY_BLOCK_SIZE [=] value`
    fn key_block_size(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("KEY_BLOCK_SIZE"),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
            )),
            |(_, _, _, _, value): (&str, &str, Option<&str>, &str, &str)| {
                TableOption::KeyBlockSize(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// parse `MAX_ROWS [=] value`
    fn max_rows(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("MAX_ROWS"),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
            )),
            |(_, _, _, _, value): (&str, &str, Option<&str>, &str, &str)| {
                TableOption::MaxRows(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// parse `MIN_ROWS [=] value`
    fn min_rows(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("MIN_ROWS"),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
            )),
            |(_, _, _, _, value): (&str, &str, Option<&str>, &str, &str)| {
                TableOption::MinRows(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// parse `PACK_KEYS [=] {0 | 1 | DEFAULT}`
    fn pack_keys(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("PACK_KEYS"),
                multispace0,
                opt(tag("=")),
                multispace0,
                DefaultOrZeroOrOne::parse,
            )),
            |(_, _, _, _, value)| TableOption::PackKeys(value),
        )(i)
    }

    /// parse `PASSWORD [=] 'string'`
    fn password(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("PASSWORD"),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
            )),
            |(_, _, _, _, password)| TableOption::Password(password),
        )(i)
    }

    /// parse `ROW_FORMAT [=] {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}`
    fn row_format(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(RowFormatType::parse, TableOption::RowFormat)(i)
    }

    /// parse `START TRANSACTION`
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

    /// parse `SECONDARY_ENGINE_ATTRIBUTE [=] 'string'`
    fn secondary_engine_attribute(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("SECONDARY_ENGINE_ATTRIBUTE"),
                multispace0,
                opt(tag("=")),
                multispace0,
                map(delimited(tag("'"), take_until("'"), tag("'")), |x| {
                    String::from(x)
                }),
            )),
            |(_, _, _, _, engine)| TableOption::SecondaryEngineAttribute(engine),
        )(i)
    }

    /// parse `STATS_AUTO_RECALC [=] {DEFAULT | 0 | 1}`
    fn stats_auto_recalc(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("STATS_AUTO_RECALC"),
                multispace0,
                opt(tag("=")),
                multispace0,
                DefaultOrZeroOrOne::parse,
            )),
            |(_, _, _, _, value)| TableOption::StatsAutoRecalc(value),
        )(i)
    }

    /// parse `STATS_PERSISTENT [=] {DEFAULT | 0 | 1}`
    fn stats_persistent(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("STATS_PERSISTENT"),
                multispace0,
                opt(tag("=")),
                multispace0,
                DefaultOrZeroOrOne::parse,
            )),
            |(_, _, _, _, value)| TableOption::StatsAutoRecalc(value),
        )(i)
    }

    /// parse `STATS_SAMPLE_PAGES [=] value`
    fn stats_sample_pages(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("STATS_SAMPLE_PAGES "),
                multispace0,
                opt(tag("=")),
                multispace0,
                digit1,
            )),
            |(_, _, _, _, value): (&str, &str, Option<&str>, &str, &str)| {
                TableOption::StatsSamplePages(value.parse::<u64>().unwrap())
            },
        )(i)
    }

    /// parse `TABLESPACE tablespace_name [STORAGE {DISK | MEMORY}]`
    fn tablespace(i: &str) -> IResult<&str, TableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("TABLESPACE"),
                multispace1,
                map(CommonParser::sql_identifier, String::from), // tablespace_name
                multispace0,
                opt(TablespaceType::parse),
            )),
            |(_, _, tablespace_name, _, storage)| TableOption::Tablespace(tablespace_name, storage),
        )(i)
    }

    /// parse `UNION [=] (tbl_name[,tbl_name]...)`
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
            )),
            |(_, _, _, _, union)| TableOption::Union(union),
        )(i)
    }
}

/// `[CONSTRAINT [symbol]] CHECK (expr) [[NOT] ENFORCED]`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CheckConstraintDefinition {
    pub symbol: Option<String>,
    pub expr: String,
    pub enforced: bool,
}

impl Display for CheckConstraintDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CONSTRAINT");
        if let Some(symbol) = &self.symbol {
            write!(f, " {}", symbol);
        }
        write!(f, " CHECK {}", &self.expr);
        if !&self.enforced {
            write!(f, " NOT ENFORCED");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use base::table_option::TableOption;
    use base::DefaultOrZeroOrOne;

    #[test]
    fn parse_table_option() {
        let str1 = "PACK_KEYS=1;";
        let res1 = TableOption::parse(str1);
        let exp = TableOption::PackKeys(DefaultOrZeroOrOne::One);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, exp);

        let str2 = "DEFAULT CHARSET=utf8;";
        let res2 = TableOption::parse(str2);
        let exp = TableOption::DefaultCharset("utf8".to_string());
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, exp);

        let str3 = "DATA DIRECTORY='/some/path';";
        let res3 = TableOption::parse(str3);
        let exp = TableOption::DataDirectory("/some/path".to_string());
        assert!(res3.is_ok());
        assert_eq!(res3.unwrap().1, exp);
    }
}
