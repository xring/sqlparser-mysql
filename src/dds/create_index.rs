use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::{terminated, tuple};
use nom::IResult;

use base::algorithm_type::AlgorithmType;
use base::error::ParseSQLError;
use base::index_option::IndexOption;
use base::lock_type::LockType;
use base::table::Table;
use base::{CommonParser, KeyPart};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CreateIndexStatement {
    pub index_name: String,
    pub index_type: Option<Index>,
    pub table: Table,
    pub key_part: Vec<KeyPart>,
    pub index_option: Option<IndexOption>,
    pub algorithm_option: Option<AlgorithmType>,
    pub lock_option: Option<LockType>,
}

impl CreateIndexStatement {
    /// CREATE \[UNIQUE | FULLTEXT | SPATIAL] INDEX index_name
    ///     \[index_type]
    ///     ON tbl_name (key_part,...)
    ///     \[index_option]
    ///     \[algorithm_option | lock_option] ...
    ///
    /// key_part: {col_name \[(length)] | (expr)} \[ASC | DESC]
    ///
    /// index_option: {
    ///     KEY_BLOCK_SIZE \[=] value
    ///   | index_type
    ///   | WITH PARSER parser_name
    ///   | COMMENT 'string'
    ///   | {VISIBLE | INVISIBLE}
    ///   | ENGINE_ATTRIBUTE \[=] 'string'
    ///   | SECONDARY_ENGINE_ATTRIBUTE \[=] 'string'
    /// }
    ///
    /// index_type:
    ///     USING {BTREE | HASH}
    ///
    /// algorithm_option:
    ///     ALGORITHM \[=] {DEFAULT | INPLACE | COPY}
    ///
    /// lock_option:
    ///     LOCK \[=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
    pub fn parse(i: &str) -> IResult<&str, CreateIndexStatement, ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("CREATE"), multispace1)),
                opt(terminated(Index::parse, multispace1)),
                tuple((tag_no_case("INDEX"), multispace1)),
                map(tuple((CommonParser::sql_identifier, multispace1)), |x| {
                    String::from(x.0)
                }),
                opt(terminated(Index::parse, multispace1)),
                terminated(tag_no_case("ON"), multispace1),
                terminated(Table::without_alias, multispace1), // tbl_name
                KeyPart::key_part_list,                        // (key_part,...)
                IndexOption::opt_index_option,
                multispace0, // [index_option]
                opt(terminated(AlgorithmType::parse, multispace0)),
                opt(terminated(LockType::parse, multispace0)),
                CommonParser::statement_terminator,
            )),
            |(
                _,
                _,
                _,
                index_name,
                index_type,
                _,
                table,
                key_part,
                index_option,
                _,
                algorithm_option,
                lock_option,
                _,
            )| CreateIndexStatement {
                index_name,
                index_type,
                table,
                key_part,
                index_option,
                algorithm_option,
                lock_option,
            },
        )(i)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Index {
    Unique,
    Fulltext,
    Spatial,
}

impl Index {
    /// \[UNIQUE | FULLTEXT | SPATIAL]
    fn parse(i: &str) -> IResult<&str, Index, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("UNIQUE"), |_| Index::Unique),
            map(tag_no_case("FULLTEXT"), |_| Index::Fulltext),
            map(tag_no_case("SPATIAL"), |_| Index::Spatial),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use dds::create_index::CreateIndexStatement;

    #[test]
    fn test_create_index() {
        let sqls = [
            "create index poster_order_employee_id_index on poster_order (employee_id);",
            "create index branch_id on poster_source (branch_id);",
            "create index poster_order_employee_id_index on poster_order (employee_id)",
        ];

        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = CreateIndexStatement::parse(sqls[i]);
            println!("{:?}", res);
            assert!(res.is_ok());
        }
    }
}
