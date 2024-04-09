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

/// parse `DROP INDEX index_name ON tbl_name
///     [algorithm_option | lock_option] ...`
///
/// algorithm_option: `ALGORITHM [=] {DEFAULT | INSTANT | INPLACE | COPY}`
/// lock_option: `LOCK [=] {DEFAULT | NONE | SHARED | EXCLUSIVE}`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DropIndexStatement {
    pub index_name: String,
    pub table: Table,
    pub algorithm_option: Option<AlgorithmType>,
    pub lock_option: Option<LockType>,
}

impl DropIndexStatement {
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
    use base::algorithm_type::AlgorithmType;
    use base::lock_type::LockType;
    use base::Table;
    use dds::drop_index::DropIndexStatement;

    #[test]
    fn parse_drop_index() {
        let sqls = [
            "drop index agent_id_index on tbl_name;",
            "drop index agent_id_index on db_name.tbl_name ALGORITHM = COPY;",
            "DROP INDEX IX_brand_id ON tbl_name LOCK = default;",
            "DROP INDEX IX_brand_id ON db_name.tbl_name ALGORITHM = COPY LOCK = default;",
        ];
        let exp_statements = [
            DropIndexStatement {
                index_name: "agent_id_index".to_string(),
                table: "tbl_name".into(),
                algorithm_option: None,
                lock_option: None,
            },
            DropIndexStatement {
                index_name: "agent_id_index".to_string(),
                table: ("db_name", "tbl_name").into(),
                algorithm_option: Some(AlgorithmType::Copy),
                lock_option: None,
            },
            DropIndexStatement {
                index_name: "IX_brand_id".to_string(),
                table: "tbl_name".into(),
                algorithm_option: None,
                lock_option: Some(LockType::Default),
            },
            DropIndexStatement {
                index_name: "IX_brand_id".to_string(),
                table: ("db_name", "tbl_name").into(),
                algorithm_option: Some(AlgorithmType::Copy),
                lock_option: Some(LockType::Default),
            },
        ];

        for i in 0..sqls.len() {
            let res = DropIndexStatement::parse(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp_statements[i]);
        }
    }
}
