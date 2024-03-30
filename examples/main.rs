extern crate sqlparser_mysql;

use sqlparser_mysql::data_definition_statement::create_index_parser;
use sqlparser_mysql::parse_sql;

fn main() {
    //let sql = "drop database `abc`";
    //let sql = "drop table `abc`,  fuck a";
    //let sql = "DROP TABLE tbl_name;";
    // let sql = "DROP TABLE IF EXISTS tbl_name;";
    // let sql = "DROP TEMPORARY TABLE IF EXISTS tbl_name1, tbl_name2 RESTRICT;";
    //let sql = "DROP DATABASE if exists tbl_name";
    // let sql = "TRUNCATE tbl_name";
    //let sql = "TRUNCATE IF EXISTS tbl_name";
    //let sql = "rename table a to  b";
    //let sql = "RENAME TABLE tbl_name1 TO tbl_name2, db1.tbl_name3 TO db2.tbl_name4;";
    let sql = "RENAME aTABLE db1.tbl_name1 TO db2.tbl_name2, tbl_name3 TO tbl_name4;";
    //let sql = "drop table `table`";
    // let sql = "drop database ab";
    // let sql = "drop database a";
    // let sql = "drop database _@";
    let sql = "CREATE aTABLE foo.order_items (order_id INT, product_id INT, quantity INT, PRIMARY KEY(order_id, product_id), FOREIGN KEY (product_id) REFERENCES product (id))";
    let a: u64;
    let res = parse_sql(sql);
    println!("{:?}", res);
}
