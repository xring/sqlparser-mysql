use core::fmt;
use std::fmt::Formatter;

use common::column::ColumnSpecification;
use common::table::Table;
use common_parsers::{sql_identifier, ws_sep_comma};
use common_statement::index_option::{index_option, IndexOption};
use common_statement::table_option::{table_option, TableOptions};
use common_statement::{
    fulltext_or_spatial_type, index_col_list, index_or_key_type, index_type, key_part,
    opt_index_name, opt_index_option, opt_index_type, single_column_definition,
    CheckConstraintDefinition, FulltextOrSpatialType, IndexOrKeyType, IndexType, KeyPart,
};
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt, rest};
use nom::multi::many1;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CreateTableStatement {
    pub table: Table,
    pub temporary: bool,
    pub if_not_exists: bool,
    pub create_type: CreateTableType,
}

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
pub fn create_table_parser(i: &[u8]) -> IResult<&[u8], CreateTableStatement> {
    alt((create_simple, create_as_query, create_like_old_table))(i)
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
    LikeOldTable(String),
}

fn create_table_options(i: &[u8]) -> IResult<&[u8], TableOptions> {
    map(many1(terminated(table_option, opt(ws_sep_comma))), |x| x)(i)
}

fn create_definition_list(i: &[u8]) -> IResult<&[u8], Vec<CreateDefinition>> {
    delimited(
        tag("("),
        many1(terminated(create_definition, opt(ws_sep_comma))),
        tag(")"),
    )(i)
}

/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     (create_definition,...)
///     [table_options]
///     [partition_options]
fn create_simple(i: &[u8]) -> IResult<&[u8], CreateTableStatement> {
    let mut parser = tuple((
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
        multispace0,
    ));

    match parser(i) {
        Ok((input, (x))) => {
            let temporary = x.0 .0;
            let if_not_exists = x.0 .1;
            let table = Table::from(x.0 .2.as_str());
            let create_type = CreateTableType::Simple(x.2, x.4, x.6);
            Ok((
                input,
                CreateTableStatement {
                    table,
                    temporary,
                    if_not_exists,
                    create_type,
                },
            ))
        }
        Err(err) => {
            println!(
                "failed to parse ---{}--- as create_simple: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     [(create_definition,...)]
///     [table_options]
///     [partition_options]
///     [IGNORE | REPLACE]
///     [AS] query_expression
fn create_as_query(i: &[u8]) -> IResult<&[u8], CreateTableStatement> {
    let mut parser = tuple((
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
        map(rest, |x: &[u8]| String::from_utf8(x.to_vec()).unwrap()),
        multispace0,
    ));

    match parser(i) {
        Ok((input, (x))) => {
            let table = Table::from(x.0 .2.as_str());
            let if_not_exists = x.0 .1;
            let temporary = x.0 .0;
            let create_type = CreateTableType::AsQuery(x.2, x.4, x.6, x.8, x.12);
            Ok((
                input,
                CreateTableStatement {
                    table,
                    temporary,
                    if_not_exists,
                    create_type,
                },
            ))
        }
        Err(err) => {
            println!(
                "failed to parse ---{}--- as create_as_query: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     { LIKE old_tbl_name | (LIKE old_tbl_name) }
fn create_like_old_table(i: &[u8]) -> IResult<&[u8], CreateTableStatement> {
    let mut parser = tuple((
        create_table_with_name,
        multispace0,
        // { LIKE old_tbl_name | (LIKE old_tbl_name) }
        map(
            alt((
                map(
                    tuple((tag_no_case("LIKE"), multispace1, sql_identifier)),
                    |x| String::from_utf8(x.2.to_vec()).unwrap(),
                ),
                map(
                    delimited(tag("("), take_until(")"), tag(")")),
                    |x: &[u8]| String::from_utf8(x.to_vec()).unwrap(),
                ),
            )),
            |x| CreateTableType::LikeOldTable(x),
        ),
        multispace0,
    ));

    match parser(i) {
        Ok((input, (x, _, create_type, _))) => {
            let table = Table::from(x.2.as_str());
            let if_not_exists = x.1;
            let temporary = x.0;
            Ok((
                input,
                CreateTableStatement {
                    table,
                    temporary,
                    if_not_exists,
                    create_type,
                },
            ))
        }
        Err(err) => {
            println!(
                "failed to parse ---{}--- as create_like_old_table: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
fn create_table_with_name(i: &[u8]) -> IResult<&[u8], (bool, bool, String)> {
    let mut parser = tuple((
        tuple((tag_no_case("CREATE"), multispace1)),
        opt(tag_no_case("TEMPORARY")),
        multispace0,
        tuple((tag_no_case("TABLE"), multispace1)),
        // [IF NOT EXISTS]
        if_not_exists,
        multispace0,
        // tbl_name
        map(sql_identifier, |x| String::from_utf8(x.to_vec()).unwrap()),
    ));
    match parser(i) {
        Ok((input, x)) => Ok((input, (x.1.is_some(), x.4, x.6))),
        Err(err) => {
            println!(
                "failed to parse ---{}--- as create_table_with_name: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// [IF NOT EXISTS]
fn if_not_exists(i: &[u8]) -> IResult<&[u8], bool> {
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
pub fn create_definition(i: &[u8]) -> IResult<&[u8], CreateDefinition> {
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
fn index_or_key(i: &[u8]) -> IResult<&[u8], CreateDefinition> {
    let mut parser = tuple((
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
    ));

    match parser(i) {
        Ok((input, (index_or_key, opt_index_name, opt_index_type, key_part, opt_index_option))) => {
            Ok((
                input,
                CreateDefinition::IndexOrKey(
                    index_or_key,
                    opt_index_name,
                    opt_index_type,
                    key_part,
                    opt_index_option,
                ),
            ))
        }
        Err(err) => {
            println!(
                "failed to parse ---{}--- as add_index_or_key: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// | {FULLTEXT | SPATIAL} [INDEX | KEY] [index_name] (key_part,...) [index_option] ...
fn fulltext_or_spatial(i: &[u8]) -> IResult<&[u8], CreateDefinition> {
    let mut parser = tuple((
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
    ));

    match parser(i) {
        Ok((
            input,
            (fulltext_or_spatial, index_or_key, index_name, key_part, opt_index_option),
        )) => Ok((
            input,
            CreateDefinition::FulltextOrSpatial(
                fulltext_or_spatial,
                index_or_key,
                index_name,
                key_part,
                opt_index_option,
            ),
        )),
        Err(err) => {
            println!(
                "failed to parse ---{}--- as add_fulltext_or_spatial: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// | [CONSTRAINT [symbol]] PRIMARY KEY [index_type] (key_part,...) [index_option] ...
fn primary_key(i: &[u8]) -> IResult<&[u8], CreateDefinition> {
    let mut parser = tuple((
        // [CONSTRAINT [symbol]]
        opt_constraint_with_opt_symbol,
        // PRIMARY KEY
        tuple((
            multispace0,
            tag_no_case("PRIMARY"),
            multispace1,
            tag_no_case("KEY"),
        )),
        // [index_type]
        opt_index_type,
        // (key_part,...)
        key_part,
        // [index_option]
        opt_index_option,
    ));

    match parser(i) {
        Ok((remaining_input, (opt_symbol, _, opt_index_type, key_part, opt_index_option))) => Ok((
            remaining_input,
            CreateDefinition::PrimaryKey(opt_symbol, opt_index_type, key_part, opt_index_option),
        )),
        Err(err) => {
            println!(
                "failed to parse ---{}--- as add_primary_key: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// [CONSTRAINT [symbol]] UNIQUE [INDEX | KEY] [index_name] [index_type] (key_part,...) [index_option] ...
fn unique(i: &[u8]) -> IResult<&[u8], CreateDefinition> {
    let mut parser = tuple((
        // [CONSTRAINT [symbol]]
        opt_constraint_with_opt_symbol,
        // UNIQUE [INDEX | KEY]
        map(
            tuple((
                multispace0,
                tag_no_case("UNIQUE"),
                multispace1,
                opt(index_or_key_type),
            )),
            |(_, _, _, value)| value,
        ),
        // [index_name]
        opt_index_name,
        // [index_type]
        opt_index_type,
        // (key_part,...)
        key_part,
        // [index_option]
        opt_index_option,
    ));

    match parser(i) {
        Ok((
            input,
            (
                opt_symbol,
                opt_index_or_key,
                opt_index_name,
                opt_index_type,
                key_part,
                opt_index_option,
            ),
        )) => Ok((
            input,
            CreateDefinition::Unique(
                opt_symbol,
                opt_index_or_key,
                opt_index_name,
                opt_index_type,
                key_part,
                opt_index_option,
            ),
        )),
        Err(err) => {
            println!(
                "failed to parse ---{}--- as ---{}---: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                "add_unique",
                err
            );
            Err(err)
        }
    }
}

/// [CONSTRAINT [symbol]] FOREIGN KEY [index_name] (col_name,...) reference_definition
fn foreign_key(i: &[u8]) -> IResult<&[u8], CreateDefinition> {
    let mut parser = tuple((
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
            )),
            |(_, value)| value.iter().map(|x| x.name.clone()).collect(),
        ),
        // reference_definition
        reference_definition,
    ));

    match parser(i) {
        Ok((input, (opt_symbol, _, opt_index_name, columns, reference_definition))) => Ok((
            input,
            CreateDefinition::ForeignKey(opt_symbol, opt_index_name, columns, reference_definition),
        )),
        Err(err) => {
            println!(
                "failed to parse ---{}--- as add_foreign_key: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// check_constraint_definition
/// | [CONSTRAINT [symbol]] CHECK (expr) [[NOT] ENFORCED]
fn check_constraint_definition(i: &[u8]) -> IResult<&[u8], CreateDefinition> {
    let mut parser = tuple((
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
    ));

    match parser(i) {
        Ok((input, (symbol, _, expr, opt_whether_enforced))) => {
            let expr = String::from_utf8(expr.to_vec()).unwrap();
            let enforced =
                opt_whether_enforced.map_or(false, |(_, opt_not, _, _, _)| opt_not.is_none());
            Ok((
                input,
                CreateDefinition::Check(CheckConstraintDefinition {
                    symbol,
                    expr,
                    enforced,
                }),
            ))
        }
        Err(err) => {
            println!(
                "failed to parse ---{}--- as add_check: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// [CONSTRAINT [symbol]]
fn opt_constraint_with_opt_symbol(i: &[u8]) -> IResult<&[u8], Option<String>> {
    map(
        opt(preceded(
            tag_no_case("CONSTRAINT"),
            opt(preceded(multispace1, sql_identifier)),
        )),
        |(x)| x.and_then(|inner| inner.map(|value| String::from_utf8(value.to_vec()).unwrap())),
    )(i)
}

/// [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    Full,
    Partial,
    Simple,
}

/// [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
fn match_type(i: &[u8]) -> IResult<&[u8], MatchType> {
    map(
        tuple((
            tag_no_case("MATCH"),
            multispace1,
            alt((
                map(tag_no_case("FULL"), |_| MatchType::Full),
                map(tag_no_case("PARTIAL"), |_| MatchType::Partial),
                map(tag_no_case("SIMPLE"), |_| MatchType::Simple),
            )),
        )),
        |x| x.2,
    )(i)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ReferenceDefinition {
    tbl_name: String,
    key_part: Vec<KeyPart>,
    match_type: Option<MatchType>,
    on_delete: Option<ReferenceOption>,
    on_update: Option<ReferenceOption>,
}

/// reference_option:
///     RESTRICT | CASCADE | SET NULL | NO ACTION | SET DEFAULT
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ReferenceOption {
    Restrict,
    Cascade,
    SetNull,
    NoAction,
    SetDefault,
}

/// reference_definition:
///     REFERENCES tbl_name (key_part,...)
///       [MATCH FULL | MATCH PARTIAL | MATCH SIMPLE]
///       [ON DELETE reference_option]
///       [ON UPDATE reference_option]
fn reference_definition(i: &[u8]) -> IResult<&[u8], ReferenceDefinition> {
    let mut parser = tuple((
        tuple((tag_no_case("REFERENCES"), multispace1)),
        // tbl_name
        map(tuple((sql_identifier, multispace1)), |x| {
            String::from_utf8(x.1.to_vec()).unwrap()
        }),
        // (key_part,...)
        key_part,
        multispace0,
        opt(match_type),
        multispace0,
        opt(map(
            tuple((
                tag_no_case("ON"),
                multispace1,
                tag_no_case("DELETE"),
                multispace1,
                reference_option,
            )),
            |x| x.4,
        )),
        multispace0,
        opt(map(
            tuple((
                tag_no_case("ON"),
                multispace1,
                tag_no_case("UPDATE"),
                multispace1,
                reference_option,
            )),
            |x| x.4,
        )),
        multispace0,
    ));

    match parser(i) {
        Ok((input, (_, tbl_name, key_part, _, match_type, _, on_delete, _, on_update, _))) => Ok((
            input,
            ReferenceDefinition {
                tbl_name,
                key_part,
                match_type,
                on_delete,
                on_update,
            },
        )),
        Err(err) => {
            println!(
                "failed to parse ---{}--- as reference_definition: {}",
                String::from(std::str::from_utf8(i).unwrap()),
                err
            );
            Err(err)
        }
    }
}

/// reference_option:
///     RESTRICT | CASCADE | SET NULL | NO ACTION | SET DEFAULT
fn reference_option(i: &[u8]) -> IResult<&[u8], ReferenceOption> {
    alt((
        map(tag_no_case("RESTRICT"), |_| ReferenceOption::Restrict),
        map(tag_no_case("CASCADE"), |_| ReferenceOption::Cascade),
        map(
            tuple((tag_no_case("SET"), multispace1, tag_no_case("NULL"))),
            |_| ReferenceOption::SetNull,
        ),
        map(
            tuple((tag_no_case("NO"), multispace1, tag_no_case("ACTION"))),
            |_| ReferenceOption::NoAction,
        ),
        map(
            tuple((tag_no_case("SET"), multispace1, tag_no_case("DEFAULT"))),
            |_| ReferenceOption::SetDefault,
        ),
    ))(i)
}

///////////////////// TODO support create partition parser

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CreatePartitionOption {
    None,
}

pub fn create_table_partition_option(i: &[u8]) -> IResult<&[u8], CreatePartitionOption> {
    map(tag_no_case(""), |_| CreatePartitionOption::None)(i)
}

///////////////////// TODO support create partition parser

#[cfg(test)]
mod test {
    use data_definition_statement::create_table::create_table_parser;

    #[test]
    fn test_create_table() {
        let create_sqls = vec![
            "CREATE TABLE order_items (order_id INT, product_id INT, quantity INT, PRIMARY KEY(order_id, product_id), FOREIGN KEY (product_id) REFERENCES product(id))",
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
            "CREATE TABLE product_structure LIKE product;",
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_table_copy LIKE my_table",
            "CREATE TABLE my_table_filtered AS SELECT * FROM my_table WHERE age < 30;",
            "CREATE TABLE employee_dept_10 AS SELECT * FROM employee WHERE department_id = 10",
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_dept_20 AS SELECT * FROM department WHERE id = 20",
            "CREATE TABLE active_products AS SELECT * FROM product WHERE price > 0",
            "CREATE TABLE IF NOT EXISTS employee_archives LIKE employee",
            "CREATE TABLE sales_by_product AS SELECT product_id, SUM(quantity) AS total_sales FROM order_items GROUP BY product_id",
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_order_summary AS SELECT order_id, SUM(quantity) AS total_items FROM order_items GROUP BY order_id",
            "CREATE TABLE employee_names AS SELECT name FROM employee",
            "CREATE TABLE product_prices AS SELECT name, price FROM product WHERE price BETWEEN 10 AND 100"
        ];

        for i in 0..create_sqls.len() {
            println!("{}/{}", i + 1, create_sqls.len());
            let res = create_table_parser(create_sqls[i].as_bytes());
            // res.unwrap();
            // println!("{:?}", res);
            assert!(res.is_ok());
            println!("{:?}", res);
        }
    }
}
