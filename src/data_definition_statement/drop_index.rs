use common::table::Table;
use common_parsers::{schema_table_name_without_alias, sql_identifier, statement_terminator};
use common_statement::{algorithm_option, lock_option, AlgorithmOption, LockType};
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropIndexStatement {
    pub index_name: String,
    pub table: Table,
    pub algorithm_option: Option<AlgorithmOption>,
    pub lock_option: Option<LockType>,
}

/// DROP INDEX index_name ON tbl_name
///     [algorithm_option | lock_option] ...
pub fn drop_index_parser(i: &[u8]) -> IResult<&[u8], DropIndexStatement> {
    map(
        tuple((
            tuple((tag_no_case("DROP"), multispace1)),
            tuple((tag_no_case("INDEX"), multispace1)),
            map(
                tuple((sql_identifier, multispace1, tag_no_case("ON"), multispace1)),
                |x| String::from_utf8(x.0.to_vec()).unwrap(),
            ),
            schema_table_name_without_alias, // tbl_name
            multispace0,
            opt(algorithm_option), // algorithm_option
            multispace0,
            opt(lock_option), // lock_option
            multispace0,
            statement_terminator,
        )),
        |(_, _, index_name, table, _, algorithm_option, _, lock_option, _, _)| DropIndexStatement {
            index_name,
            table,
            algorithm_option,
            lock_option,
        },
    )(i)
}

#[cfg(test)]
mod test {
    use common_statement::lock_option;
    use data_definition_statement::drop_index::drop_index_parser;

    #[test]
    fn test_lock_option() {
        let part = "LOCK = default";
        let res = lock_option(part.as_bytes());
        println!("{:?}", res);
        // assert!(res.is_ok());
    }

    #[test]
    fn test_drop_index() {
        let sqls = vec![
            "drop index agent_id_index on stat_agent_organ_201912;",
            "drop index agent_id_index on stat_agent_organ_201912 ALGORITHM = COPY;",
            "DROP INDEX IX_brand_id ON esta_developer_brand LOCK = default;",
            "DROP INDEX IX_brand_id ON esta_developer_brand ALGORITHM = COPY LOCK = default;",
        ];

        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = drop_index_parser(sqls[i].as_bytes());
            // res.unwrap();
            println!("{:?}", res);
            // assert!(res.is_ok());
            // println!("{:?}", res);
        }
    }
}
