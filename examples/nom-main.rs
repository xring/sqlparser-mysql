extern crate nom;
extern crate sqlparser_mysql;

use nom::character::complete::multispace0;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::terminated;
use nom::{
    bytes::complete::tag_no_case,
    error::{context, VerboseError},
    sequence::tuple,
    IResult,
};
use sqlparser_mysql::common::table::Table;
use sqlparser_mysql::common_parsers::{schema_table_reference_to_schema_table_reference, statement_terminator, ws_sep_comma};
use sqlparser_mysql::data_definition_statement::{create_table_parser, rename_table_parser};

// 使用VerboseError来获得更详细的错误信息
fn parse_add_demo(input: &str) -> IResult<&str, (&str, &str), VerboseError<&str>> {
    // 使用tuple来组合解析器，并为每个步骤提供上下文
    tuple((
        context("expect ADD", tag_no_case("ADD")),
        context("expect DEMO", tag_no_case("DEMO")),
    ))(input)
}

fn parse_add_demo2(
    input: &str,
) -> IResult<&str, (&str, &str, &str, &str, Vec<(Table, Table)>, ()), VerboseError<&str>> {
    // 使用tuple来组合解析器，并为每个步骤提供上下文
    tuple((
        tag_no_case("RENAME "),
        multispace0,
        tag_no_case("TABLE "),
        multispace0,
        many0(terminated(
            schema_table_reference_to_schema_table_reference,
            opt(ws_sep_comma),
        )),
        statement_terminator,
    ))(input)
}

fn main() {
    let input = "RENAME aTABLE db1.tbl_name1 TO db2.tbl_name2, tbl_name3 TO tbl_name4;";
    let input = "CREATE aTABLEa foo.order_items (order_id INT, product_id INT, quantity INT, PRIMARY KEY(order_id, product_id), FOREIGN KEY (product_id) REFERENCES product (id));";
    match create_table_parser(input) {
        Ok((remaining, value)) => println!(
            "Parsed successfully: {:?}, Remaining: '{}'",
            value, remaining
        ),
        Err(err) => match err {
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                //println!("{e}");
                // println!("=====");
                println!("Error: {}", nom::error::convert_error(input, e));
            }
            _ => println!("Parsing failed in an unexpected way."),
        },
    }
}
