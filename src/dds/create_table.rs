use core::fmt;
use std::fmt::Formatter;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::multi::many1;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

use base::column::{Column, ColumnSpecification};
use base::error::ParseSQLError;
use base::fulltext_or_spatial_type::FulltextOrSpatialType;
use base::index_option::IndexOption;
use base::index_or_key_type::IndexOrKeyType;
use base::index_type::IndexType;
use base::table::Table;
use base::table_option::TableOption;
use base::{CheckConstraintDefinition, CommonParser, KeyPart, ReferenceDefinition};
use dms::SelectStatement;

/// **CreateTableStatement**
/// [MySQL Doc](https://dev.mysql.com/doc/refman/8.0/en/create-table.html)
///
/// - Simple Create:
/// ```sql
/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     (create_definition,...)
///     [table_options]
///     [partition_options]
///```
/// - Create as Select:
/// ```sql
/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     [(create_definition,...)]
///     [table_options]
///     [partition_options]
///     [IGNORE | REPLACE]
///     [AS] query_expression
///```
/// - Create Like:
/// ```sql
/// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
///     { LIKE old_tbl_name | (LIKE old_tbl_name) }
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CreateTableStatement {
    /// `[TEMPORARY]` part
    pub temporary: bool,
    /// `[IF NOT EXISTS]` part
    pub if_not_exists: bool,
    /// `tbl_name` part
    pub table: Table,
    /// simple definition | as select definition | like other table definition
    pub create_type: CreateTableType,
}

impl CreateTableStatement {
    pub fn parse(i: &str) -> IResult<&str, CreateTableStatement, ParseSQLError<&str>> {
        alt((
            CreateTableType::create_simple,
            CreateTableType::create_as_query,
            CreateTableType::create_like_old_table,
        ))(i)
    }
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum IgnoreOrReplaceType {
    Ignore,
    Replace,
}

impl IgnoreOrReplaceType {
    fn parse(i: &str) -> IResult<&str, IgnoreOrReplaceType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("IGNORE"), |_| IgnoreOrReplaceType::Ignore),
            map(tag_no_case("REPLACE"), |_| IgnoreOrReplaceType::Replace),
        ))(i)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CreateTableType {
    /// Simple Create
    /// ```sql
    /// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
    ///     (create_definition,...)
    ///     [table_options]
    ///     [partition_options]
    /// ```
    Simple {
        create_definition: Vec<CreateDefinition>, // (create_definition,...)
        table_options: Option<Vec<TableOption>>,  // [table_options]
        partition_options: Option<CreatePartitionOption>, // [partition_options]
    },

    /// Select Create
    /// ```sql
    /// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
    ///     [(create_definition,...)]
    ///     [table_options]
    ///     [partition_options]
    ///     [IGNORE | REPLACE]
    ///     [AS] query_expression
    /// ```
    AsQuery {
        create_definition: Option<Vec<CreateDefinition>>, // (create_definition,...)
        table_options: Option<Vec<TableOption>>,          // [table_options]
        partition_options: Option<CreatePartitionOption>, // [partition_options]
        opt_ignore_or_replace: Option<IgnoreOrReplaceType>, // [IGNORE | REPLACE]
        query_expression: SelectStatement,                // [AS] query_expression
    },

    /// Like Create
    /// ```sql
    /// CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name
    ///     { LIKE old_tbl_name | (LIKE old_tbl_name) }
    /// ```
    LikeOldTable { table: Table },
}

impl CreateTableType {
    /// parse [CreateTableType::Simple]
    fn create_simple(i: &str) -> IResult<&str, CreateTableStatement, ParseSQLError<&str>> {
        map(
            tuple((
                Self::create_table_with_name,
                multispace0,
                // (create_definition,...)
                CreateDefinition::create_definition_list,
                multispace0,
                // [table_options]
                opt(Self::create_table_options),
                multispace0,
                // [partition_options]
                opt(CreatePartitionOption::parse),
                CommonParser::statement_terminator,
            )),
            |(x)| {
                let temporary = x.0 .0;
                let if_not_exists = x.0 .1;
                let table = x.0 .2;
                let create_type = CreateTableType::Simple {
                    create_definition: x.2,
                    table_options: x.4,
                    partition_options: x.6,
                };
                CreateTableStatement {
                    table,
                    temporary,
                    if_not_exists,
                    create_type,
                }
            },
        )(i)
    }

    /// parse [CreateTableType::AsQuery]
    fn create_as_query(i: &str) -> IResult<&str, CreateTableStatement, ParseSQLError<&str>> {
        map(
            tuple((
                Self::create_table_with_name,
                multispace0,
                // [(create_definition,...)]
                opt(CreateDefinition::create_definition_list),
                multispace0,
                // [table_options]
                opt(Self::create_table_options),
                multispace0,
                // [partition_options]
                opt(CreatePartitionOption::parse),
                multispace0,
                opt(IgnoreOrReplaceType::parse),
                multispace0,
                opt(tag_no_case("AS")),
                multispace0,
                SelectStatement::parse,
            )),
            |(x)| {
                let table = x.0 .2;
                let if_not_exists = x.0 .1;
                let temporary = x.0 .0;
                let create_type = CreateTableType::AsQuery {
                    create_definition: x.2,
                    table_options: x.4,
                    partition_options: x.6,
                    opt_ignore_or_replace: x.8,
                    query_expression: x.12,
                };
                CreateTableStatement {
                    table,
                    temporary,
                    if_not_exists,
                    create_type,
                }
            },
        )(i)
    }

    /// parse [CreateTableType::LikeOldTable]
    fn create_like_old_table(i: &str) -> IResult<&str, CreateTableStatement, ParseSQLError<&str>> {
        map(
            tuple((
                Self::create_table_with_name,
                multispace0,
                // { LIKE old_tbl_name | (LIKE old_tbl_name) }
                map(
                    alt((
                        map(
                            tuple((
                                tag_no_case("LIKE"),
                                multispace1,
                                Table::schema_table_reference,
                            )),
                            |x| x.2,
                        ),
                        map(
                            delimited(tag("("), Table::schema_table_reference, tag(")")),
                            |x| x,
                        ),
                    )),
                    |x| CreateTableType::LikeOldTable { table: x },
                ),
                CommonParser::statement_terminator,
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

    /// parse `[table_options]` part
    fn create_table_options(i: &str) -> IResult<&str, Vec<TableOption>, ParseSQLError<&str>> {
        map(
            many1(map(
                tuple((
                    TableOption::parse,
                    multispace0,
                    opt(CommonParser::ws_sep_comma),
                )),
                |x| x.0,
            )),
            |x| x,
        )(i)
    }

    /// parse `CREATE [TEMPORARY] TABLE [IF NOT EXISTS] tbl_name` part:
    fn create_table_with_name(i: &str) -> IResult<&str, (bool, bool, Table), ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("CREATE"), multispace1)),
                opt(tag_no_case("TEMPORARY")),
                multispace0,
                tuple((tag_no_case("TABLE"), multispace1)),
                // [IF NOT EXISTS]
                Self::if_not_exists,
                multispace0,
                // tbl_name
                Table::schema_table_reference,
            )),
            |x| (x.1.is_some(), x.4, x.6),
        )(i)
    }

    /// parse `[IF NOT EXISTS]` part
    fn if_not_exists(i: &str) -> IResult<&str, bool, ParseSQLError<&str>> {
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
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CreateDefinition {
    /// col_name column_definition
    ColumnDefinition {
        column_definition: ColumnSpecification,
    },

    /// `{INDEX | KEY} [index_name] [index_type] (key_part,...) [index_option] ...`
    IndexOrKey {
        index_or_key: IndexOrKeyType,          // {INDEX | KEY}
        opt_index_name: Option<String>,        // [index_name]
        opt_index_type: Option<IndexType>,     // [index_type]
        key_part: Vec<KeyPart>,                // (key_part,...)
        opt_index_option: Option<IndexOption>, // [index_option]
    },

    /// `{FULLTEXT | SPATIAL} [INDEX | KEY] [index_name] (key_part,...) [index_option] ...`
    FulltextOrSpatial {
        fulltext_or_spatial: FulltextOrSpatialType, // {FULLTEXT | SPATIAL}
        index_or_key: Option<IndexOrKeyType>,       // {INDEX | KEY}
        index_name: Option<String>,                 // [index_name]
        key_part: Vec<KeyPart>,                     // (key_part,...)
        opt_index_option: Option<IndexOption>,      // [index_option]
    },

    /// `[CONSTRAINT [symbol]] PRIMARY KEY [index_type] (key_part,...) [index_option] ...`
    PrimaryKey {
        opt_symbol: Option<String>,            // [symbol]
        opt_index_type: Option<IndexType>,     // [index_type]
        key_part: Vec<KeyPart>,                // (key_part,...)
        opt_index_option: Option<IndexOption>, // [index_option]
    },

    /// `[CONSTRAINT [symbol]] UNIQUE [INDEX | KEY] [index_name] [index_type] (key_part,...) [index_option] ...`
    Unique {
        opt_symbol: Option<String>,               // [symbol]
        opt_index_or_key: Option<IndexOrKeyType>, // [INDEX | KEY]
        opt_index_name: Option<String>,           // [index_name]
        opt_index_type: Option<IndexType>,        // [index_type]
        key_part: Vec<KeyPart>,                   // (key_part,...)
        opt_index_option: Option<IndexOption>,    // [index_option]
    },

    /// `[CONSTRAINT [symbol]] FOREIGN KEY [index_name] (col_name,...) reference_definition`
    ForeignKey {
        opt_symbol: Option<String>,                // [symbol]
        opt_index_name: Option<String>,            // [index_name]
        columns: Vec<String>,                      // (col_name,...)
        reference_definition: ReferenceDefinition, // reference_definition
    },

    /// `check_constraint_definition`
    Check {
        check_constraint_definition: CheckConstraintDefinition,
    },
}

impl CreateDefinition {
    /// `create_definition: {
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
    /// }`
    pub fn parse(i: &str) -> IResult<&str, CreateDefinition, ParseSQLError<&str>> {
        alt((
            map(ColumnSpecification::parse, |x| {
                CreateDefinition::ColumnDefinition {
                    column_definition: x,
                }
            }),
            CreateDefinition::index_or_key,
            CreateDefinition::fulltext_or_spatial,
            CreateDefinition::primary_key,
            CreateDefinition::unique,
            CreateDefinition::foreign_key,
            CreateDefinition::check_constraint_definition,
        ))(i)
    }

    fn create_definition_list(
        i: &str,
    ) -> IResult<&str, Vec<CreateDefinition>, ParseSQLError<&str>> {
        delimited(
            tag("("),
            many1(map(
                tuple((
                    multispace0,
                    CreateDefinition::parse,
                    multispace0,
                    opt(CommonParser::ws_sep_comma),
                    multispace0,
                )),
                |x| x.1,
            )),
            tag(")"),
        )(i)
    }

    /// `{INDEX | KEY} [index_name] [index_type] (key_part,...) [index_option] ...`
    fn index_or_key(i: &str) -> IResult<&str, CreateDefinition, ParseSQLError<&str>> {
        map(
            tuple((
                // {INDEX | KEY}
                IndexOrKeyType::parse,
                // [index_name]
                CommonParser::opt_index_name,
                // [index_type]
                IndexType::opt_index_type,
                // (key_part,...)
                KeyPart::parse,
                // [index_option]
                IndexOption::opt_index_option,
            )),
            |(index_or_key, opt_index_name, opt_index_type, key_part, opt_index_option)| {
                CreateDefinition::IndexOrKey {
                    index_or_key,
                    opt_index_name,
                    opt_index_type,
                    key_part,
                    opt_index_option,
                }
            },
        )(i)
    }

    /// `{FULLTEXT | SPATIAL} [INDEX | KEY] [index_name] (key_part,...) [index_option] ...`
    fn fulltext_or_spatial(i: &str) -> IResult<&str, CreateDefinition, ParseSQLError<&str>> {
        map(
            tuple((
                // {FULLTEXT | SPATIAL}
                FulltextOrSpatialType::parse,
                // [INDEX | KEY]
                preceded(multispace1, opt(IndexOrKeyType::parse)),
                // [index_name]
                CommonParser::opt_index_name,
                // (key_part,...)
                KeyPart::parse,
                // [index_option]
                IndexOption::opt_index_option,
            )),
            |(fulltext_or_spatial, index_or_key, index_name, key_part, opt_index_option)| {
                CreateDefinition::FulltextOrSpatial {
                    fulltext_or_spatial,
                    index_or_key,
                    index_name,
                    key_part,
                    opt_index_option,
                }
            },
        )(i)
    }

    /// `[CONSTRAINT [symbol]] PRIMARY KEY [index_type] (key_part,...) [index_option] ...`
    fn primary_key(i: &str) -> IResult<&str, CreateDefinition, ParseSQLError<&str>> {
        map(
            tuple((
                Self::opt_constraint_with_opt_symbol, // [CONSTRAINT [symbol]]
                tuple((
                    multispace0,
                    tag_no_case("PRIMARY"),
                    multispace1,
                    tag_no_case("KEY"),
                )), // PRIMARY KEY
                IndexType::opt_index_type,            // [index_type]
                KeyPart::parse,                       // (key_part,...)
                IndexOption::opt_index_option,        // [index_option]
            )),
            |(opt_symbol, _, opt_index_type, key_part, opt_index_option)| {
                CreateDefinition::PrimaryKey {
                    opt_symbol,
                    opt_index_type,
                    key_part,
                    opt_index_option,
                }
            },
        )(i)
    }

    /// `[CONSTRAINT [symbol]] UNIQUE [INDEX | KEY] [index_name] [index_type]
    ///  (key_part,...) [index_option] ...`
    fn unique(i: &str) -> IResult<&str, CreateDefinition, ParseSQLError<&str>> {
        map(
            tuple((
                Self::opt_constraint_with_opt_symbol, // [CONSTRAINT [symbol]]
                map(
                    tuple((
                        multispace0,
                        tag_no_case("UNIQUE"),
                        multispace1,
                        opt(IndexOrKeyType::parse),
                    )),
                    |(_, _, _, value)| value,
                ), // UNIQUE [INDEX | KEY]
                CommonParser::opt_index_name,         // [index_name]
                IndexType::opt_index_type,            // [index_type]
                KeyPart::parse,                       // (key_part,...)
                IndexOption::opt_index_option,        // [index_option]
            )),
            |(
                opt_symbol,
                opt_index_or_key,
                opt_index_name,
                opt_index_type,
                key_part,
                opt_index_option,
            )| {
                CreateDefinition::Unique {
                    opt_symbol,
                    opt_index_or_key,
                    opt_index_name,
                    opt_index_type,
                    key_part,
                    opt_index_option,
                }
            },
        )(i)
    }

    /// `[CONSTRAINT [symbol]] FOREIGN KEY [index_name] (col_name,...) reference_definition`
    fn foreign_key(i: &str) -> IResult<&str, CreateDefinition, ParseSQLError<&str>> {
        map(
            tuple((
                // [CONSTRAINT [symbol]]
                Self::opt_constraint_with_opt_symbol,
                // FOREIGN KEY
                tuple((
                    multispace0,
                    tag_no_case("FOREIGN"),
                    multispace1,
                    tag_no_case("KEY"),
                )),
                // [index_name]
                CommonParser::opt_index_name,
                // (col_name,...)
                map(
                    tuple((
                        multispace0,
                        delimited(
                            tag("("),
                            delimited(multispace0, Column::index_col_list, multispace0),
                            tag(")"),
                        ),
                        multispace0,
                    )),
                    |(_, value, _)| value.iter().map(|x| x.name.clone()).collect(),
                ),
                // reference_definition
                ReferenceDefinition::parse,
            )),
            |(opt_symbol, _, opt_index_name, columns, reference_definition)| {
                CreateDefinition::ForeignKey {
                    opt_symbol,
                    opt_index_name,
                    columns,
                    reference_definition,
                }
            },
        )(i)
    }

    /// check_constraint_definition
    /// `[CONSTRAINT [symbol]] CHECK (expr) [[NOT] ENFORCED]`
    fn check_constraint_definition(
        i: &str,
    ) -> IResult<&str, CreateDefinition, ParseSQLError<&str>> {
        map(
            tuple((
                // [CONSTRAINT [symbol]]
                Self::opt_constraint_with_opt_symbol,
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
                CreateDefinition::Check {
                    check_constraint_definition: CheckConstraintDefinition {
                        symbol,
                        expr,
                        enforced,
                    },
                }
            },
        )(i)
    }

    /// `[CONSTRAINT [symbol]]`
    fn opt_constraint_with_opt_symbol(
        i: &str,
    ) -> IResult<&str, Option<String>, ParseSQLError<&str>> {
        map(
            opt(preceded(
                tag_no_case("CONSTRAINT"),
                opt(preceded(multispace1, CommonParser::sql_identifier)),
            )),
            |(x)| x.and_then(|inner| inner.map(String::from)),
        )(i)
    }
}

///////////////////// TODO support create partition parser
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CreatePartitionOption {
    None,
}

impl CreatePartitionOption {
    fn parse(i: &str) -> IResult<&str, CreatePartitionOption, ParseSQLError<&str>> {
        map(tag_no_case(""), |_| CreatePartitionOption::None)(i)
    }
}
///////////////////// TODO support create partition parser

#[cfg(test)]
mod tests {
    use dds::create_table::{CreateDefinition, CreateTableStatement};

    #[test]
    fn test_create_table() {
        let create_sqls = vec![
            r###"CREATE TABLE `process_type` (`last_update_tm` timestamp(0))"###,
            r###"CREATE TABLE `process_type` (`last_update_tm` timestamp(0) NOT NULL DEFAULT CURRENT_TIMESTAMP(0) ON UPDATE CURRENT_TIMESTAMP(0))"###,
            "CREATE TABLE foo.order_items (order_id INT, product_id INT, quantity INT, PRIMARY KEY(order_id, product_id), FOREIGN KEY (product_id) REFERENCES product (id))",
            "CREATE TABLE employee (id INT, name VARCHAR(100), department_id INT, PRIMARY KEY(id), FOREIGN KEY (department_id) REFERENCES department(id))",
            "CREATE TABLE my_table (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), age INT)",
            "CREATE TEMPORARY TABLE temp_table (id INT, score DECIMAL(5, 2))",
            "CREATE TABLE IF NOT EXISTS my_table (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), age INT)",
            "CREATE TABLE department (id INT AUTO_INCREMENT, name VARCHAR(100), PRIMARY KEY(id))",
            "CREATE TABLE product (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), price DECIMAL(10,2), category_id INT, INDEX(category_id))",
            "CREATE TABLE my_table_copy LIKE my_table",
            "CREATE TABLE IF NOT EXISTS my_table_copy LIKE my_table",
            "CREATE TEMPORARY TABLE temp_table_copy LIKE temp_table;",
            "CREATE TABLE department_copy LIKE department",
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_table_copy LIKE my_table",
            "CREATE TABLE IF NOT EXISTS bar.employee_archives LIKE foo.employee",
            "CREATE TABLE product (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), price DECIMAL(10,2), category_id INT, INDEX(category_id))",
            "CREATE TABLE my_table_filtered AS SELECT * FROM my_table WHERE age < 30;",
            "CREATE TABLE employee_dept_10 AS SELECT * FROM employee WHERE department_id = 10",
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_dept_20 AS SELECT * FROM department WHERE id = 20",
            "CREATE TABLE active_products AS SELECT * FROM product WHERE price > 0",
            "CREATE TABLE sales_by_product AS SELECT product_id, SUM(quantity) AS total_sales FROM order_items GROUP BY product_id",
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_order_summary AS SELECT order_id, SUM(quantity) AS total_items FROM order_items GROUP BY order_id",
            "CREATE TABLE employee_names AS SELECT name FROM employee",
            "CREATE TABLE new_table AS SELECT name, age FROM my_table WHERE age > 18",
            "CREATE TABLE unique_names IGNORE AS SELECT DISTINCT name FROM my_table",
            "CREATE TABLE employee_summary AS SELECT department_id, COUNT(*) AS employee_count FROM employee GROUP BY department_id",
            "CREATE TEMPORARY TABLE temp_employee AS SELECT * FROM employee WHERE department_id = 3",
            "CREATE TABLE IF NOT EXISTS employee_backup AS SELECT * FROM employee",
            "CREATE TABLE product_prices AS SELECT name, price FROM product WHERE price BETWEEN 10 AND 100",
            "CREATE TABLE author ( a_id int not null, a_fname varchar(20), a_lname varchar(20), a_mname varchar(20), a_dob date, a_bio int, PRIMARY KEY(a_id))",
            "CREATE TABLE customer ( c_id int not null, c_uname varchar(20), c_passwd varchar(20), c_fname varchar(17), c_lname varchar(17), c_addr_id int, c_phone varchar(18), c_email varchar(50), c_since date, c_last_login date, c_login timestamp, c_expiration timestamp, c_discount real, c_balance double, c_ytd_pmt double, c_birthdate date, c_data int, PRIMARY KEY(c_id))",
            "CREATE TABLE item ( i_id int not null, i_title varchar(60), i_a_id int, i_pub_date date, i_publisher varchar(60), i_subject varchar(60), i_desc text, i_related1 int, i_related2 int, i_related3 int, i_related4 int, i_related5 int, i_thumbnail varchar(40), i_image varchar(40), i_srp double, i_cost double, i_avail date, i_stock int, i_isbn char(13), i_page int, i_backing varchar(15), i_dimensions varchar(25), PRIMARY KEY(i_id))",
            "CREATE TABLE user (user_id int(5) unsigned NOT NULL auto_increment,user_name varchar(255) binary NOT NULL default '',user_rights tinyblob NOT NULL default '',user_password tinyblob NOT NULL default '',user_newpassword tinyblob NOT NULL default '',user_email tinytext NOT NULL default '',user_options blob NOT NULL default '',user_touched char(14) binary NOT NULL default '',UNIQUE KEY user_id (user_id)) ENGINE=MyISAM PACK_KEYS=1;",
            "CREATE TABLE `admin_assert` (`assert_id` int(10) unsigned NOT NULL Auto_Increment COMMENT 'Assert ID',`assert_type` varchar(20) DEFAULT NULL COMMENT 'Assert Type',`assert_data` text COMMENT 'Assert Data',PRIMARY KEY (`assert_id`)) ENGINE=InnoDB DEFAULT CHARSET=utf8;",
            "CREATE TABLE `admin_role` (`role_id` int(10) unsigned NOT NULL Auto_Increment COMMENT 'Role ID',`parent_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT 'Parent Role ID',`tree_level` smallint(5) unsigned NOT NULL DEFAULT '0' COMMENT 'Role Tree Level',`sort_order` smallint(5) unsigned NOT NULL DEFAULT '0' COMMENT 'Role Sort Order',`role_type` varchar(1) NOT NULL DEFAULT '0' COMMENT 'Role Type',`user_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT 'User ID',`role_name` varchar(50) DEFAULT NULL COMMENT 'Role Name',PRIMARY KEY (`role_id`),KEY `IDX_ADMIN_ROLE_PARENT_ID_SORT_ORDER` (`parent_id`,`sort_order`),KEY `IDX_ADMIN_ROLE_TREE_LEVEL` (`tree_level`)) ENGINE=InnoDB DEFAULT CHARSET=utf8 COMMENT='Admin Role Table';",
            "CREATE TABLE `postcode_city` (`id` int(10) unsigned NOT NULL Auto_Increment COMMENT 'Id',`country_code` varchar(5) NOT NULL COMMENT 'Country Code',`postcode` varchar(20) NOT NULL COMMENT 'Postcode',`city` text NOT NULL COMMENT 'City',PRIMARY KEY (`id`)) ENGINE=InnoDB Auto_Increment=52142 DEFAULT CHARSET=utf8 COMMENT='Postcode -> City';",
        ];

        for i in 0..create_sqls.len() {
            println!("{}/{}", i + 1, create_sqls.len());
            let res = CreateTableStatement::parse(create_sqls[i]);
            println!("{:?}", res);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_create_definition_list() {
        let part = "(order_id INT, product_id INT, quantity INT, PRIMARY KEY(order_id, product_id), FOREIGN KEY (product_id) REFERENCES product(id))";
        let res = CreateDefinition::create_definition_list(part);
        assert!(res.is_ok());
    }
}
