use core::fmt;
use std::fmt::Formatter;

use common::column::ColumnSpecification;
use common::table::Table;
use common_parsers::{schema_table_reference, sql_identifier, statement_terminator, ws_sep_comma};
use common_statement::index_option::{index_option, IndexOption};
use common_statement::table_option::{table_option, TableOptions};
use common_statement::{
    fulltext_or_spatial_type, index_col_list, index_or_key_type, index_type, key_part,
    opt_index_name, opt_index_option, opt_index_type, reference_definition,
    single_column_definition, CheckConstraintDefinition, FulltextOrSpatialType, IndexOrKeyType,
    IndexType, KeyPart, ReferenceDefinition,
};
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt, rest};
use nom::error::VerboseError;
use nom::multi::many1;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;
use common::Statement;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CreateTableStatement {
    pub table: Table,
    pub temporary: bool,
    pub if_not_exists: bool,
    pub create_type: CreateTableType,
}

impl Statement for CreateTableStatement {}

impl fmt::Display for CreateTableStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let table_name = match &self.table.schema {
            Some(schema) => format!("{}.{}", schema, self.table.name),
            None => format!(" {}", self.table.name),
        };
        write!(f, "CREATE TABLE {} ", table_name)?;
        Ok(())
    }
}

/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     (create_definition,...)
///     [table_options]
///     [partition_options]
///
/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     [(create_definition,...)]
///     [table_options]
///     [partition_options]
///     [IGNORE | REPLACE]
///     [AS] query_expression
///
/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     { LIKE old_tbl_name | (LIKE old_tbl_name) }
pub fn create_table_parser(i: &str) -> IResult<&str, CreateTableStatement, VerboseError<&str>> {
    //alt((create_simple, create_like_old_table, create_as_query))(i)
    create_simple(i)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IgnoreOrReplaceType {
    Ignore,
    Replace,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CreateTableType {
    /// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
    ///     (create_definition,...)
    ///     [table_options]
    ///     [partition_options]
    Simple(
        Vec<CreateDefinition>,         // (create_definition,...)
        Option<TableOptions>,          // [table_options]
        Option<CreatePartitionOption>, // [partition_options]
    ),

    /// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
    ///     [(create_definition,...)]
    ///     [table_options]
    ///     [partition_options]
    ///     [IGNORE | REPLACE]
    ///     [AS] query_expression
    AsQuery(
        Option<Vec<CreateDefinition>>, // (create_definition,...)
        Option<TableOptions>,          // [table_options]
        Option<CreatePartitionOption>, // [partition_options]
        Option<IgnoreOrReplaceType>,   // [IGNORE | REPLACE]
        String,                        // [AS] query_expression
    ),

    /// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
    ///     { LIKE old_tbl_name | (LIKE old_tbl_name) }
    LikeOldTable(Table),
}

fn create_table_options(i: &str) -> IResult<&str, TableOptions, VerboseError<&str>> {
    map(many1(terminated(table_option, opt(ws_sep_comma))), |x| x)(i)
}

fn create_definition_list(i: &str) -> IResult<&str, Vec<CreateDefinition>, VerboseError<&str>> {
    delimited(
        tag("("),
        many1(map(
            tuple((
                multispace0,
                create_definition,
                multispace0,
                opt(ws_sep_comma),
                multispace0,
            )),
            |x| x.1,
        )),
        tag(")"),
    )(i)
}

/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     (create_definition,...)
///     [table_options]
///     [partition_options]
fn create_simple(i: &str) -> IResult<&str, CreateTableStatement, VerboseError<&str>> {
    map(
        tuple((
            create_table_with_name,
            multispace0,
            // (create_definition,...)
            create_definition_list,
            multispace0,
            // [table_options]
            opt(create_table_options),
            multispace0,
            // [partition_options]
            opt(create_table_partition_option),
            statement_terminator,
        )),
        |(x)| {
            let temporary = x.0 .0;
            let if_not_exists = x.0 .1;
            let table = x.0 .2;
            let create_type = CreateTableType::Simple(x.2, x.4, x.6);
            CreateTableStatement {
                table,
                temporary,
                if_not_exists,
                create_type,
            }
        },
    )(i)
}

/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     [(create_definition,...)]
///     [table_options]
///     [partition_options]
///     [IGNORE | REPLACE]
///     [AS] query_expression
fn create_as_query(i: &str) -> IResult<&str, CreateTableStatement, VerboseError<&str>> {
    map(
        tuple((
            create_table_with_name,
            multispace0,
            // [(create_definition,...)]
            opt(create_definition_list),
            multispace0,
            // [table_options]
            opt(create_table_options),
            multispace0,
            // [partition_options]
            opt(create_table_partition_option),
            multispace0,
            opt(alt((
                map(tag_no_case("IGNORE"), |_| IgnoreOrReplaceType::Ignore),
                map(tag_no_case("REPLACE"), |_| IgnoreOrReplaceType::Replace),
            ))),
            multispace0,
            opt(tag_no_case("AS")),
            multispace0,
            map(rest, |x| String::from(x)),
            statement_terminator,
        )),
        |(x)| {
            let table = x.0 .2;
            let if_not_exists = x.0 .1;
            let temporary = x.0 .0;
            let create_type = CreateTableType::AsQuery(x.2, x.4, x.6, x.8, x.12);
            CreateTableStatement {
                table,
                temporary,
                if_not_exists,
                create_type,
            }
        },
    )(i)
}

/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     { LIKE old_tbl_name | (LIKE old_tbl_name) }
fn create_like_old_table(i: &str) -> IResult<&str, CreateTableStatement, VerboseError<&str>> {
    map(
        tuple((
            create_table_with_name,
            multispace0,
            // { LIKE old_tbl_name | (LIKE old_tbl_name) }
            map(
                alt((
                    map(
                        tuple((tag_no_case("LIKE"), multispace1, schema_table_reference)),
                        |x| x.2,
                    ),
                    map(delimited(tag("("), schema_table_reference, tag(")")), |x| x),
                )),
                |x| CreateTableType::LikeOldTable(x),
            ),
            statement_terminator,
        )),
        |(x, _, create_type, _)| {
            let table = x.2;
            let if_not_exists = x.1;
            let temporary = x.0;
            CreateTableStatement {
                table,
                temporary,
                if_not_exists,
                create_type,
            }
        },
    )(i)
}

/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
fn create_table_with_name(i: &str) -> IResult<&str, (bool, bool, Table), VerboseError<&str>> {
    map(
        tuple((
            tuple((tag_no_case("CREATE"), multispace1)),
            opt(tag_no_case("TEMPORARY")),
            multispace0,
            tuple((tag_no_case("TABLE"), multispace1)),
            // [IF NOT EXISTS]
            if_not_exists,
            multispace0,
            // tbl_name
            schema_table_reference,
        )),
        |x| (x.1.is_some(), x.4, x.6),
    )(i)
}

/// [IF NOT EXISTS]
fn if_not_exists(i: &str) -> IResult<&str, bool, VerboseError<&str>> {
    map(
        opt(tuple((
            tag_no_case("IF"),
            multispace1,
            tag_no_case("NOT"),
            multispace1,
            tag_no_case("EXISTS"),
        ))),
        |x| x.is_some(),
    )(i)
}

/// create_definition: {
///     col_name column_definition
///   | {INDEX | KEY} [index_name] [index_type] (key_part,...)
///       [index_option] ...
///   | {FULLTEXT | SPATIAL} [INDEX | KEY] [index_name] (key_part,...)
///       [index_option] ...
///   | [CONSTRAINT [symbol]] PRIMARY KEY
///       [index_type] (key_part,...)
///       [index_option] ...
///   | [CONSTRAINT [symbol]] UNIQUE [INDEX | KEY]
///       [index_name] [index_type] (key_part,...)
///       [index_option] ...
///   | [CONSTRAINT [symbol]] FOREIGN KEY
///       [index_name] (col_name,...)
///       reference_definition
///   | check_constraint_definition
/// }
pub fn create_definition(i: &str) -> IResult<&str, CreateDefinition, VerboseError<&str>> {
    alt((
        map(single_column_definition, |x| {
            CreateDefinition::ColumnDefinition(x)
        }),
        index_or_key,
        fulltext_or_spatial,
        primary_key,
        unique,
        foreign_key,
        check_constraint_definition,
    ))(i)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CreateDefinition {
    /// col_name column_definition
    ColumnDefinition(ColumnSpecification),

    /// {INDEX | KEY} [index_name] [index_type] (key_part,...) [index_option] ...
    IndexOrKey(
        IndexOrKeyType,      // {INDEX | KEY}
        Option<String>,      // [index_name]
        Option<IndexType>,   // [index_type]
        Vec<KeyPart>,        // (key_part,...)
        Option<IndexOption>, // [index_option]
    ),

    /// {FULLTEXT | SPATIAL} [INDEX | KEY] [index_name] (key_part,...) [index_option] ...
    FulltextOrSpatial(
        FulltextOrSpatialType,  // {FULLTEXT | SPATIAL}
        Option<IndexOrKeyType>, // {INDEX | KEY}
        Option<String>,         // [index_name]
        Vec<KeyPart>,           // (key_part,...)
        Option<IndexOption>,    // [index_option]
    ),

    /// [CONSTRAINT [symbol]] PRIMARY KEY [index_type] (key_part,...) [index_option] ...
    PrimaryKey(
        Option<String>,      // [symbol]
        Option<IndexType>,   // [index_type]
        Vec<KeyPart>,        // (key_part,...)
        Option<IndexOption>, // [index_option]
    ),

    /// [CONSTRAINT [symbol]] UNIQUE [INDEX | KEY] [index_name] [index_type] (key_part,...) [index_option] ...
    Unique(
        Option<String>,         // [symbol]
        Option<IndexOrKeyType>, // [INDEX | KEY]
        Option<String>,         // [index_name]
        Option<IndexType>,      // [index_type]
        Vec<KeyPart>,           // (key_part,...)
        Option<IndexOption>,    // [index_option]
    ),

    /// [CONSTRAINT [symbol]] FOREIGN KEY [index_name] (col_name,...) reference_definition
    ForeignKey(
        Option<String>,      // [symbol]
        Option<String>,      // [index_name]
        Vec<String>,         // (col_name,...)
        ReferenceDefinition, // reference_definition
    ),

    /// check_constraint_definition
    Check(CheckConstraintDefinition),
}

/// {INDEX | KEY} [index_name] [index_type] (key_part,...) [index_option] ...
fn index_or_key(i: &str) -> IResult<&str, CreateDefinition, VerboseError<&str>> {
    map(
        tuple((
            // {INDEX | KEY}
            index_or_key_type,
            // [index_name]
            opt_index_name,
            // [index_type]
            opt_index_type,
            // (key_part,...)
            key_part,
            // [index_option]
            opt_index_option,
        )),
        |(index_or_key, opt_index_name, opt_index_type, key_part, opt_index_option)| {
            CreateDefinition::IndexOrKey(
                index_or_key,
                opt_index_name,
                opt_index_type,
                key_part,
                opt_index_option,
            )
        },
    )(i)
}

/// | {FULLTEXT | SPATIAL} [INDEX | KEY] [index_name] (key_part,...) [index_option] ...
fn fulltext_or_spatial(i: &str) -> IResult<&str, CreateDefinition, VerboseError<&str>> {
    map(
        tuple((
            // {FULLTEXT | SPATIAL}
            fulltext_or_spatial_type,
            // [INDEX | KEY]
            preceded(multispace1, opt(index_or_key_type)),
            // [index_name]
            opt_index_name,
            // (key_part,...)
            key_part,
            // [index_option]
            opt_index_option,
        )),
        |(fulltext_or_spatial, index_or_key, index_name, key_part, opt_index_option)| {
            CreateDefinition::FulltextOrSpatial(
                fulltext_or_spatial,
                index_or_key,
                index_name,
                key_part,
                opt_index_option,
            )
        },
    )(i)
}

/// | [CONSTRAINT [symbol]] PRIMARY KEY [index_type] (key_part,...) [index_option] ...
fn primary_key(i: &str) -> IResult<&str, CreateDefinition, VerboseError<&str>> {
    map(
        tuple((
            opt_constraint_with_opt_symbol, // [CONSTRAINT [symbol]]
            tuple((
                multispace0,
                tag_no_case("PRIMARY"),
                multispace1,
                tag_no_case("KEY"),
            )), // PRIMARY KEY
            opt_index_type,                 // [index_type]
            key_part,                       // (key_part,...)
            opt_index_option,               // [index_option]
        )),
        |(opt_symbol, _, opt_index_type, key_part, opt_index_option)| {
            CreateDefinition::PrimaryKey(opt_symbol, opt_index_type, key_part, opt_index_option)
        },
    )(i)
}

/// [CONSTRAINT [symbol]] UNIQUE [INDEX | KEY] [index_name] [index_type] (key_part,...) [index_option] ...
fn unique(i: &str) -> IResult<&str, CreateDefinition, VerboseError<&str>> {
    map(
        tuple((
            opt_constraint_with_opt_symbol, // [CONSTRAINT [symbol]]
            map(
                tuple((
                    multispace0,
                    tag_no_case("UNIQUE"),
                    multispace1,
                    opt(index_or_key_type),
                )),
                |(_, _, _, value)| value,
            ), // UNIQUE [INDEX | KEY]
            opt_index_name,                 // [index_name]
            opt_index_type,                 // [index_type]
            key_part,                       // (key_part,...)
            opt_index_option,               // [index_option]
        )),
        |(
            opt_symbol,
            opt_index_or_key,
            opt_index_name,
            opt_index_type,
            key_part,
            opt_index_option,
        )| {
            CreateDefinition::Unique(
                opt_symbol,
                opt_index_or_key,
                opt_index_name,
                opt_index_type,
                key_part,
                opt_index_option,
            )
        },
    )(i)
}

/// [CONSTRAINT [symbol]] FOREIGN KEY [index_name] (col_name,...) reference_definition
fn foreign_key(i: &str) -> IResult<&str, CreateDefinition, VerboseError<&str>> {
    map(
        tuple((
            // [CONSTRAINT [symbol]]
            opt_constraint_with_opt_symbol,
            // FOREIGN KEY
            tuple((
                multispace0,
                tag_no_case("FOREIGN"),
                multispace1,
                tag_no_case("KEY"),
            )),
            // [index_name]
            opt_index_name,
            // (col_name,...)
            map(
                tuple((
                    multispace0,
                    delimited(
                        tag("("),
                        delimited(multispace0, index_col_list, multispace0),
                        tag(")"),
                    ),
                    multispace0,
                )),
                |(_, value, _)| value.iter().map(|x| x.name.clone()).collect(),
            ),
            // reference_definition
            reference_definition,
        )),
        |(opt_symbol, _, opt_index_name, columns, reference_definition)| {
            CreateDefinition::ForeignKey(opt_symbol, opt_index_name, columns, reference_definition)
        },
    )(i)
}

/// check_constraint_definition
/// | [CONSTRAINT [symbol]] CHECK (expr) [[NOT] ENFORCED]
fn check_constraint_definition(i: &str) -> IResult<&str, CreateDefinition, VerboseError<&str>> {
    map(
        tuple((
            // [CONSTRAINT [symbol]]
            opt_constraint_with_opt_symbol,
            // CHECK
            tuple((multispace1, tag_no_case("CHECK"), multispace0)),
            // (expr)
            delimited(tag("("), take_until(")"), tag(")")),
            // [[NOT] ENFORCED]
            opt(tuple((
                multispace0,
                opt(tag_no_case("NOT")),
                multispace1,
                tag_no_case("ENFORCED"),
                multispace0,
            ))),
        )),
        |(symbol, _, expr, opt_whether_enforced)| {
            let expr = String::from(expr);
            let enforced =
                opt_whether_enforced.map_or(false, |(_, opt_not, _, _, _)| opt_not.is_none());
            CreateDefinition::Check(CheckConstraintDefinition {
                symbol,
                expr,
                enforced,
            })
        },
    )(i)
}

/// [CONSTRAINT [symbol]]
fn opt_constraint_with_opt_symbol(i: &str) -> IResult<&str, Option<String>, VerboseError<&str>> {
    map(
        opt(preceded(
            tag_no_case("CONSTRAINT"),
            opt(preceded(multispace1, sql_identifier)),
        )),
        |(x)| x.and_then(|inner| inner.map(|value| String::from(value))),
    )(i)
}

///////////////////// TODO support create partition parser

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CreatePartitionOption {
    None,
}

pub fn create_table_partition_option(
    i: &str,
) -> IResult<&str, CreatePartitionOption, VerboseError<&str>> {
    map(tag_no_case(""), |_| CreatePartitionOption::None)(i)
}

///////////////////// TODO support create partition parser

#[cfg(test)]
mod test {
    use data_definition_statement::create_table::{create_definition_list, create_table_parser};

    #[test]
    fn test_create_table() {
        let create_sqls = vec![
            "CREATE TABLE foo.order_items (order_id INT, product_id INT, quantity INT, PRIMARY KEY(order_id, product_id), FOREIGN KEY (product_id) REFERENCES product (id))",
            "CREATE TABLE employee (id INT, name VARCHAR(100), department_id INT, PRIMARY KEY(id), FOREIGN KEY (department_id) REFERENCES department(id))",
            "CREATE TABLE my_table (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), age INT)",
            "CREATE TEMPORARY TABLE temp_table (id INT, score DECIMAL(5, 2))",
            "CREATE TABLE IF NOT EXISTS my_table (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), age INT)",
            "CREATE TABLE department (id INT AUTO_INCREMENT, name VARCHAR(100), PRIMARY KEY(id))",
            "CREATE TABLE product (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), price DECIMAL(10,2), category_id INT, INDEX(category_id))",
            "CREATE TABLE new_table AS SELECT name, age FROM my_table WHERE age > 18",
            "CREATE TABLE unique_names IGNORE AS SELECT DISTINCT name FROM my_table",
            "CREATE TABLE employee_summary AS SELECT department_id, COUNT(*) AS employee_count FROM employee GROUP BY department_id",
            "CREATE TEMPORARY TABLE temp_employee AS SELECT * FROM employee WHERE department_id = 3",
            "CREATE TABLE IF NOT EXISTS employee_backup AS SELECT * FROM employee",
            "CREATE TABLE my_table_copy LIKE my_table",
            "CREATE TABLE IF NOT EXISTS my_table_copy LIKE my_table",
            "CREATE TEMPORARY TABLE temp_table_copy LIKE temp_table;",
            "CREATE TABLE department_copy LIKE department",
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_table_copy LIKE my_table",
            "CREATE TABLE my_table_filtered AS SELECT * FROM my_table WHERE age < 30;",
            "CREATE TABLE employee_dept_10 AS SELECT * FROM employee WHERE department_id = 10",
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_dept_20 AS SELECT * FROM department WHERE id = 20",
            "CREATE TABLE active_products AS SELECT * FROM product WHERE price > 0",
            "CREATE TABLE sales_by_product AS SELECT product_id, SUM(quantity) AS total_sales FROM order_items GROUP BY product_id",
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_order_summary AS SELECT order_id, SUM(quantity) AS total_items FROM order_items GROUP BY order_id",
            "CREATE TABLE employee_names AS SELECT name FROM employee",
            "CREATE TABLE product_prices AS SELECT name, price FROM product WHERE price BETWEEN 10 AND 100",
            "CREATE TABLE IF NOT EXISTS bar.employee_archives LIKE foo.employee",
        ];

        for i in 0..create_sqls.len() {
            println!("{}/{}", i + 1, create_sqls.len());
            let res = create_table_parser(create_sqls[i]);
            // res.unwrap();
            // println!("{:?}", res);
            assert!(res.is_ok());
            println!("{:#?}", res.unwrap().1);
        }
    }

    #[test]
    fn test_create_definition_list() {
        let part = "(order_id INT, product_id INT, quantity INT, PRIMARY KEY(order_id, product_id), FOREIGN KEY (product_id) REFERENCES product(id))";
        let res = create_definition_list(part);
        assert!(res.is_ok());
    }
}
