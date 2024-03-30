use common::table::Table;
use common_parsers::{schema_table_name_without_alias, sql_identifier, statement_terminator};
use common_statement::index_option::IndexOption;
use common_statement::{
    algorithm_option, index_type, key_part, key_part_list, lock_option, opt_index_option,
    AlgorithmOption, KeyPart, LockType,
};
use data_definition_statement::drop_index::DropIndexStatement;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::error::VerboseError;
use nom::sequence::{terminated, tuple};
use nom::IResult;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CreateIndexStatement {
    pub index_name: String,
    pub index_type: Option<Index>,
    pub table: Table,
    pub key_part: Vec<KeyPart>,
    pub index_option: Option<IndexOption>,
    pub algorithm_option: Option<AlgorithmOption>,
    pub lock_option: Option<LockType>,
}

/// CREATE [UNIQUE | FULLTEXT | SPATIAL] INDEX index_name
///     [index_type]
///     ON tbl_name (key_part,...)
///     [index_option]
///     [algorithm_option | lock_option] ...
///
/// key_part: {col_name [(length)] | (expr)} [ASC | DESC]
///
/// index_option: {
///     KEY_BLOCK_SIZE [=] value
///   | index_type
///   | WITH PARSER parser_name
///   | COMMENT 'string'
///   | {VISIBLE | INVISIBLE}
///   | ENGINE_ATTRIBUTE [=] 'string'
///   | SECONDARY_ENGINE_ATTRIBUTE [=] 'string'
/// }
///
/// index_type:
///     USING {BTREE | HASH}
///
/// algorithm_option:
///     ALGORITHM [=] {DEFAULT | INPLACE | COPY}
///
/// lock_option:
///     LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
pub fn create_index_parser(i: &str) -> IResult<&str, CreateIndexStatement, VerboseError<&str>> {
    map(
        tuple((
            tuple((tag_no_case("CREATE"), multispace1)),
            opt(terminated(index, multispace1)),
            tuple((tag_no_case("INDEX"), multispace1)),
            map(tuple((sql_identifier, multispace1)), |x| String::from(x.0)),
            opt(terminated(index, multispace1)),
            terminated(tag_no_case("ON"), multispace1),
            terminated(schema_table_name_without_alias, multispace1), // tbl_name
            key_part,                                                 // (key_part,...)
            opt_index_option,
            multispace0, // [index_option]
            opt(terminated(algorithm_option, multispace0)),
            opt(terminated(lock_option, multispace0)),
            statement_terminator,
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Index {
    Unique,
    Fulltext,
    Spatial,
}

/// [UNIQUE | FULLTEXT | SPATIAL]
fn index(i: &str) -> IResult<&str, Index, VerboseError<&str>> {
    alt((
        map(tag_no_case("UNIQUE"), |_| Index::Unique),
        map(tag_no_case("FULLTEXT"), |_| Index::Fulltext),
        map(tag_no_case("SPATIAL"), |_| Index::Spatial),
    ))(i)
}

#[cfg(test)]
mod test {
    use data_definition_statement::drop_index::drop_index_parser;

    #[test]
    fn test_create_index() {
        let sqls = vec![
            "create index poster_order_employee_id_index on poster_order (employee_id);",
            "create index branch_id on poster_source (branch_id);",
        ];

        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = drop_index_parser(sqls[i]);
            assert!(res.is_ok());
            println!("{:?}", res);
        }
    }
}
