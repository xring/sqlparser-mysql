extern crate sqlparser_mysql;

use sqlparser_mysql::parser::Parser;
use sqlparser_mysql::ParseConfig;

fn main() {
    let config = ParseConfig::default();
    let sql = "SELECT a, b, 123, myfunc(b) \
            FROM table_1 \
            WHERE a > b AND b < 100 \
            ORDER BY a DESC, b";
    // parse to a Statement
    let ast = Parser::parse(&config, sql).unwrap();

    println!("AST: {:#?}", ast);
}
