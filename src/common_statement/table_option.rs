use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{digit1, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{delimited, tuple};
use nom::IResult;

use common_parsers::sql_identifier;
use common_statement::{CompressionType, DefaultOrZeroOrOne, index_col_list, InsertMethodType, RowFormatType, TablespaceType};

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

pub type TableOptions = Vec<TableOption>;

pub fn table_option(i: &[u8]) -> IResult<&[u8], TableOption> {
    alt((table_option_part_1, table_option_part_2))(i)
}

fn table_option_part_1(i: &[u8]) -> IResult<&[u8], TableOption> {
    alt((
        autoextend_size,
        auto_increment,
        avg_row_length,
        default_character_set,
        checksum,
        default_collate,
        comment,
        compression,
        connection,
        data_directory,
        index_directory,
        delay_key_write,
        encryption,
        engine,
        engine_attribute,
    ))(i)
}

fn table_option_part_2(i: &[u8]) -> IResult<&[u8], TableOption> {
    alt((
        insert_method,
        key_block_size,
        max_rows,
        min_rows,
        pack_keys,
        password,
        row_format,
        secondary_engine_attribute,
        stats_auto_recalc,
        stats_persistent,
        stats_sample_pages,
        tablespace,
        union,
    ))(i)
}

/// AUTOEXTEND_SIZE [=] value
fn autoextend_size(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("AUTOEXTEND_SIZE "),
            multispace0,
            opt(tag("=")),
            multispace0,
            digit1,
            multispace0,
        )),
        |x| TableOption::AutoextendSize(std::str::from_utf8(x.4).unwrap().parse::<u64>().unwrap()),
    )(i)
}

/// AUTO_INCREMENT [=] value
fn auto_increment(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("AUTO_INCREMENT "),
            multispace0,
            opt(tag("=")),
            multispace0,
            digit1,
            multispace0,
        )),
        |x| TableOption::AutoIncrement(std::str::from_utf8(x.4).unwrap().parse::<u64>().unwrap()),
    )(i)
}

/// AVG_ROW_LENGTH [=] value
fn avg_row_length(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("AVG_ROW_LENGTH "),
            multispace0,
            opt(tag("=")),
            multispace0,
            digit1,
            multispace0,
        )),
        |x| TableOption::AvgRowLength(std::str::from_utf8(x.4).unwrap().parse::<u64>().unwrap()),
    )(i)
}

/// [DEFAULT] CHARACTER SET [=] charset_name
fn default_character_set(i: &[u8]) -> IResult<&[u8], TableOption> {
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
            map(sql_identifier, |x| String::from_utf8(x.to_vec()).unwrap()),
            multispace0,
        )),
        |(_, _, _, charset_name, _)| TableOption::DefaultCharacterSet(charset_name),
    )(i)
}

/// CHECKSUM [=] {0 | 1}
fn checksum(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("CHECKSUM "),
            multispace0,
            opt(tag("=")),
            multispace0,
            alt((tag("0"), tag("1"))),
            multispace0,
        )),
        |x| TableOption::Checksum(std::str::from_utf8(x.4).unwrap().parse::<u8>().unwrap()),
    )(i)
}

/// [DEFAULT] COLLATE [=] collation_name
fn default_collate(i: &[u8]) -> IResult<&[u8], TableOption> {
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
                |(_, _, _, _, _, collation_name, _)| {
                    String::from_utf8(collation_name.to_vec()).unwrap()
                },
            ),
            multispace0,
        )),
        |(_, _, _, _, collation_name, _)| TableOption::DefaultCollate(collation_name),
    )(i)
}

/// COMMENT [=] 'string'
fn comment(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("COMMENT "),
            multispace0,
            opt(tag("=")),
            multispace0,
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
            ),
            multispace0,
        )),
        |(_, _, _, _, comment, _)| TableOption::Comment(comment),
    )(i)
}

/// COMPRESSION [=] {'ZLIB' | 'LZ4' | 'NONE'}
fn compression(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("COMPRESSION "),
            multispace0,
            opt(tag("=")),
            multispace0,
            alt((
                map(delimited(tag("'"), tag_no_case("ZLIB"), tag("'")), |_| {
                    CompressionType::ZLIB
                }),
                map(delimited(tag("'"), tag_no_case("LZ4"), tag("'")), |_| {
                    CompressionType::LZ4
                }),
                map(delimited(tag("'"), tag_no_case("NONE"), tag("'")), |_| {
                    CompressionType::NONE
                }),
            )),
            multispace0,
        )),
        |(_, _, _, _, compression_type, _)| TableOption::Compression(compression_type),
    )(i)
}

/// CONNECTION [=] 'connect_string'
fn connection(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("CONNECTION "),
            multispace0,
            opt(tag("=")),
            multispace0,
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
            ),
            multispace0,
        )),
        |(_, _, _, _, connect_string, _)| TableOption::Connection(connect_string),
    )(i)
}

/// {DATA | INDEX} DIRECTORY [=] 'absolute path to directory'
fn data_directory(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("DATA"),
            multispace1,
            tag_no_case("DIRECTORY "),
            multispace0,
            opt(tag("=")),
            multispace0,
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
            ),
            multispace0,
        )),
        |(_, _, _, _, _, _, path, _)| TableOption::DataDirectory(path),
    )(i)
}

/// {DATA | INDEX} DIRECTORY [=] 'absolute path to directory'
fn index_directory(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("INDEX"),
            multispace1,
            tag_no_case("DIRECTORY "),
            multispace0,
            opt(tag("=")),
            multispace0,
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
            ),
            multispace0,
        )),
        |(_, _, _, _, _, _, path, _)| TableOption::DataDirectory(path),
    )(i)
}

/// DELAY_KEY_WRITE [=] {0 | 1}
fn delay_key_write(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("DELAY_KEY_RITE "),
            multispace0,
            opt(tag("=")),
            multispace0,
            alt((tag("0"), tag("1"))),
            multispace0,
        )),
        |x| TableOption::DelayKeyWrite(std::str::from_utf8(x.4).unwrap().parse::<u8>().unwrap()),
    )(i)
}

/// ENCRYPTION [=] {'Y' | 'N'}
fn encryption(i: &[u8]) -> IResult<&[u8], TableOption> {
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
fn engine(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("ENGINE"),
            multispace1,
            opt(tag("=")),
            multispace0,
            sql_identifier,
            multispace0,
        )),
        |(_, _, _, _, engine, _)| TableOption::Engine(String::from_utf8(engine.to_vec()).unwrap()),
    )(i)
}

/// ENGINE_ATTRIBUTE [=] 'string'
fn engine_attribute(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("ENGINE_ATTRIBUTE "),
            multispace0,
            opt(tag("=")),
            multispace0,
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
            ),
            multispace0,
        )),
        |(_, _, _, _, attribute, _)| TableOption::EngineAttribute(attribute),
    )(i)
}

/// INSERT_METHOD [=] { NO | FIRST | LAST }
fn insert_method(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("INSERT METHOD "),
            multispace0,
            opt(tag("=")),
            multispace0,
            alt((
                map(tag_no_case("NO"), |_| InsertMethodType::NO),
                map(tag_no_case("FIRST"), |_| InsertMethodType::FIRST),
                map(tag_no_case("LAST"), |_| InsertMethodType::LAST),
            )),
            multispace0,
        )),
        |x| TableOption::InsertMethod(x.4),
    )(i)
}

/// KEY_BLOCK_SIZE [=] value
fn key_block_size(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("KEY_BLOCK_SIZE "),
            multispace0,
            opt(tag("=")),
            multispace0,
            digit1,
            multispace0,
        )),
        |x| TableOption::KeyBlockSize(std::str::from_utf8(x.3).unwrap().parse::<u64>().unwrap()),
    )(i)
}

/// MAX_ROWS [=] value
fn max_rows(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("MAX_ROWS "),
            multispace0,
            opt(tag("=")),
            multispace0,
            digit1,
            multispace0,
        )),
        |x| TableOption::MaxRows(std::str::from_utf8(x.3).unwrap().parse::<u64>().unwrap()),
    )(i)
}

/// MIN_ROWS [=] value
fn min_rows(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("MIN_ROWS "),
            multispace0,
            opt(tag("=")),
            multispace0,
            digit1,
            multispace0,
        )),
        |x| TableOption::MinRows(std::str::from_utf8(x.3).unwrap().parse::<u64>().unwrap()),
    )(i)
}

/// PACK_KEYS [=] {0 | 1 | DEFAULT}
fn pack_keys(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("INSERT_METHOD "),
            multispace0,
            opt(tag("=")),
            multispace0,
            alt((
                map(tag_no_case("0"), |_| DefaultOrZeroOrOne::Zero),
                map(tag_no_case("1"), |_| DefaultOrZeroOrOne::One),
                map(tag_no_case("DEFAULT"), |_| DefaultOrZeroOrOne::Default),
            )),
            multispace0,
        )),
        |x| TableOption::PackKeys(x.4),
    )(i)
}

/// PASSWORD [=] 'string'
fn password(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("PASSWORD "),
            multispace0,
            opt(tag("=")),
            multispace0,
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
            ),
            multispace0,
        )),
        |(_, _, _, _, password, _)| TableOption::Password(password),
    )(i)
}

/// ROW_FORMAT [=] {DEFAULT | DYNAMIC | FIXED | COMPRESSED | REDUNDANT | COMPACT}
fn row_format(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("INSERT METHOD "),
            multispace0,
            opt(tag("=")),
            multispace0,
            alt((
                map(tag_no_case("DEFAULT"), |_| RowFormatType::DEFAULT),
                map(tag_no_case("DYNAMIC"), |_| RowFormatType::DYNAMIC),
                map(tag_no_case("FIXED"), |_| RowFormatType::FIXED),
                map(tag_no_case("COMPRESSED"), |_| RowFormatType::COMPRESSED),
                map(tag_no_case("REDUNDANT"), |_| RowFormatType::REDUNDANT),
                map(tag_no_case("COMPACT"), |_| RowFormatType::COMPACT),
            )),
            multispace0,
        )),
        |x| TableOption::RowFormat(x.4),
    )(i)
}

/// START TRANSACTION
/// create table only
fn start_transaction(i: &[u8]) -> IResult<&[u8], TableOption> {
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
fn secondary_engine_attribute(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("SECONDARY_ENGINE_ATTRIBUTE "),
            multispace0,
            opt(tag("=")),
            multispace0,
            map(
                delimited(tag("'"), take_until("'"), tag("'")),
                |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
            ),
            multispace0,
        )),
        |(_, _, _, _, engine, _)| TableOption::SecondaryEngineAttribute(engine),
    )(i)
}

/// STATS_AUTO_RECALC [=] {DEFAULT | 0 | 1}
fn stats_auto_recalc(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("STATS_AUTO_RECALC "),
            multispace0,
            opt(tag("=")),
            multispace0,
            alt((
                map(tag_no_case("0"), |_| DefaultOrZeroOrOne::Zero),
                map(tag_no_case("1"), |_| DefaultOrZeroOrOne::One),
                map(tag_no_case("DEFAULT"), |_| DefaultOrZeroOrOne::Default),
            )),
            multispace0,
        )),
        |x| TableOption::StatsAutoRecalc(x.4),
    )(i)
}

/// STATS_PERSISTENT [=] {DEFAULT | 0 | 1}
fn stats_persistent(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("STATS_PERSISTENT "),
            multispace0,
            opt(tag("=")),
            multispace0,
            alt((
                map(tag_no_case("0"), |_| DefaultOrZeroOrOne::Zero),
                map(tag_no_case("1"), |_| DefaultOrZeroOrOne::One),
                map(tag_no_case("DEFAULT"), |_| DefaultOrZeroOrOne::Default),
            )),
            multispace0,
        )),
        |x| TableOption::StatsPersistent(x.4),
    )(i)
}

/// STATS_SAMPLE_PAGES [=] value
fn stats_sample_pages(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("STATS_SAMPLE_PAGES "),
            multispace0,
            opt(tag("=")),
            multispace0,
            digit1,
            multispace0,
        )),
        |x| {
            TableOption::StatsSamplePages(std::str::from_utf8(x.4).unwrap().parse::<u64>().unwrap())
        },
    )(i)
}

/// TABLESPACE tablespace_name [STORAGE {DISK | MEMORY}]
fn tablespace(i: &[u8]) -> IResult<&[u8], TableOption> {
    map(
        tuple((
            tag_no_case("TABLESPACE"),
            multispace1,
            map(sql_identifier, |x| String::from_utf8(x.to_vec()).unwrap()), // tablespace_name
            multispace0,
            opt(map(
                tuple((
                    tag_no_case("STORAGE"),
                    alt((
                        map(tag_no_case("DISK"), |_| TablespaceType::StorageDisk),
                        map(tag_no_case("MEMORY"), |_| TablespaceType::StorageMemory),
                    )),
                )),
                |x| x.1,
            )),
            multispace0,
        )),
        |(_, _, tablespace_name, _, storage, _)| TableOption::Tablespace(tablespace_name, storage),
    )(i)
}

/// UNION [=] (tbl_name[,tbl_name]...)
fn union(i: &[u8]) -> IResult<&[u8], TableOption> {
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
                        delimited(multispace0, index_col_list, multispace0),
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
