use nom::bytes::complete::tag_no_case;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;

use base::algorithm_type::AlgorithmType;
use base::error::ParseSQLError;
use base::lock_type::LockType;
use base::table::Table;
use base::CommonParser;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropIndexStatement {
    pub index_name: String,
    pub table: Table,
    pub algorithm_option: Option<AlgorithmType>,
    pub lock_option: Option<LockType>,
}

impl DropIndexStatement {
    /// DROP INDEX index_name ON tbl_name
    ///     [algorithm_option | lock_option] ...
    pub fn parse(i: &str) -> IResult<&str, DropIndexStatement, ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("DROP"), multispace1)),
                tuple((tag_no_case("INDEX"), multispace1)),
                map(
                    tuple((
                        CommonParser::sql_identifier,
                        multispace1,
                        tag_no_case("ON"),
                        multispace1,
                    )),
                    |x| String::from(x.0),
                ),
                Table::without_alias, // tbl_name
                multispace0,
                opt(AlgorithmType::parse), // algorithm_option
                multispace0,
                opt(LockType::parse), // lock_option
                multispace0,
                CommonParser::statement_terminator,
            )),
            |(_, _, index_name, table, _, algorithm_option, _, lock_option, _, _)| {
                DropIndexStatement {
                    index_name,
                    table,
                    algorithm_option,
                    lock_option,
                }
            },
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use base::lock_type::LockType;
    use dds::drop_index::DropIndexStatement;

    #[test]
    fn test_lock_option() {
        let part = "LOCK = default";
        let res = LockType::parse(part);
        println!("{:?}", res);
        // assert!(res.is_ok());
    }

    #[test]
    fn test_drop_index() {
        let sqls = [
            "drop index agent_id_index on stat_agent_organ_201912;",
            "drop index agent_id_index on stat_agent_organ_201912 ALGORITHM = COPY;",
            "DROP INDEX IX_brand_id ON esta_developer_brand LOCK = default;",
            "DROP INDEX IX_brand_id ON esta_developer_brand ALGORITHM = COPY LOCK = default;",
        ];

        for i in 0..sqls.len() {
            println!("{}/{}", i + 1, sqls.len());
            let res = DropIndexStatement::parse(sqls[i]);
            // res.unwrap();
            println!("{:?}", res);
            // assert!(res.is_ok());
            // println!("{:?}", res);
        }
    }
}
