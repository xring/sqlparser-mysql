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
        index_or_key: IndexOrKeyType,               // {INDEX | KEY}
        opt_index_name: Option<String>,             // [index_name]
        opt_index_type: Option<IndexType>,          // [index_type]
        key_part: Vec<KeyPart>,                     // (key_part,...)
        opt_index_option: Option<Vec<IndexOption>>, // [index_option]
    },

    /// `{FULLTEXT | SPATIAL} [INDEX | KEY] [index_name] (key_part,...) [index_option] ...`
    FulltextOrSpatial {
        fulltext_or_spatial: FulltextOrSpatialType, // {FULLTEXT | SPATIAL}
        index_or_key: Option<IndexOrKeyType>,       // {INDEX | KEY}
        index_name: Option<String>,                 // [index_name]
        key_part: Vec<KeyPart>,                     // (key_part,...)
        opt_index_option: Option<Vec<IndexOption>>, // [index_option]
    },

    /// `[CONSTRAINT [symbol]] PRIMARY KEY [index_type] (key_part,...) [index_option] ...`
    PrimaryKey {
        opt_symbol: Option<String>,                 // [symbol]
        opt_index_type: Option<IndexType>,          // [index_type]
        key_part: Vec<KeyPart>,                     // (key_part,...)
        opt_index_option: Option<Vec<IndexOption>>, // [index_option]
    },

    /// `[CONSTRAINT [symbol]] UNIQUE [INDEX | KEY] [index_name] [index_type] (key_part,...) [index_option] ...`
    Unique {
        opt_symbol: Option<String>,                 // [symbol]
        opt_index_or_key: Option<IndexOrKeyType>,   // [INDEX | KEY]
        opt_index_name: Option<String>,             // [index_name]
        opt_index_type: Option<IndexType>,          // [index_type]
        key_part: Vec<KeyPart>,                     // (key_part,...)
        opt_index_option: Option<Vec<IndexOption>>, // [index_option]
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
    use base::column::{ColumnConstraint, ColumnSpecification};
    use base::table_option::TableOption;
    use base::{
        Column, DataType, FieldDefinitionExpression, KeyPart, KeyPartType, Literal,
        ReferenceDefinition,
    };
    use dds::create_table::{
        CreateDefinition, CreatePartitionOption, CreateTableStatement, CreateTableType,
    };
    use dms::SelectStatement;

    #[test]
    fn parse_create_simple() {
        let sqls = [
            "create table admin_role \
            (`role_id` int(10) unsigned NOT NULL Auto_Increment COMMENT 'Role ID',\
            `role_type` varchar(1) NOT NULL DEFAULT '0' COMMENT 'Role Type',\
            PRIMARY KEY (`role_id`))\
            ENGINE=InnoDB DEFAULT CHARSET=utf8 COMMENT='Admin Role Table';",
        ];
        let exp = [
            CreateTableStatement {
                temporary: false,
                if_not_exists: false,
                table: "admin_role".into(),
                create_type: CreateTableType::Simple {
                    create_definition: vec![
                        CreateDefinition::ColumnDefinition {
                            column_definition: ColumnSpecification {
                                column: "role_id".into(),
                                data_type: DataType::UnsignedInt(10),
                                constraints: vec![
                                    ColumnConstraint::NotNull,
                                    ColumnConstraint::AutoIncrement,
                                ],
                                comment: Some("Role ID".to_string()),
                                position: None,
                            },
                        },
                        CreateDefinition::ColumnDefinition {
                            column_definition: ColumnSpecification {
                                column: "role_type".into(),
                                data_type: DataType::Varchar(1),
                                constraints: vec![
                                    ColumnConstraint::NotNull,
                                    ColumnConstraint::DefaultValue(Literal::String(
                                        "0".to_string(),
                                    )),
                                ],
                                comment: Some("Role Type".to_string()),
                                position: None,
                            },
                        },
                        CreateDefinition::PrimaryKey {
                            opt_symbol: None,
                            opt_index_type: None,
                            key_part: vec![KeyPart {
                                r#type: KeyPartType::ColumnNameWithLength {
                                    col_name: "role_id".to_string(),
                                    length: None,
                                },
                                order: None,
                            }],
                            opt_index_option: None,
                        },
                    ],
                    table_options: Some(vec![
                        TableOption::Engine("InnoDB".to_string()),
                        TableOption::DefaultCharset("utf8".to_string()),
                        TableOption::Comment("Admin Role Table".to_string()),
                    ]),
                    partition_options: Some(CreatePartitionOption::None),
                },
            },
            CreateTableStatement {
                temporary: false,
                if_not_exists: false,
                table: "tbl_name".into(),
                create_type: CreateTableType::LikeOldTable {
                    table: "old_tbl_name".into(),
                },
            },
        ];

        for i in 0..sqls.len() {
            let res = CreateTableType::create_simple(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp[i]);
        }
    }

    #[test]
    fn parse_create_as_query() {
        let sqls = ["CREATE TABLE tbl_name AS SELECT * from other_tbl_name"];
        let exp = [CreateTableStatement {
            temporary: false,
            if_not_exists: false,
            table: "tbl_name".into(),
            create_type: CreateTableType::AsQuery {
                create_definition: None,
                table_options: None,
                partition_options: Some(CreatePartitionOption::None),
                opt_ignore_or_replace: None,
                query_expression: SelectStatement {
                    tables: vec!["other_tbl_name".into()],
                    distinct: false,
                    fields: vec![FieldDefinitionExpression::All],
                    join: vec![],
                    where_clause: None,
                    group_by: None,
                    order: None,
                    limit: None,
                },
            },
        }];
        for i in 0..sqls.len() {
            let res = CreateTableType::create_as_query(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp[i]);
        }
    }

    #[test]
    fn parse_create_like_old() {
        let sqls = ["CREATE TABLE tbl_name LIKE old_tbl_name"];
        let exp = [CreateTableStatement {
            temporary: false,
            if_not_exists: false,
            table: "tbl_name".into(),
            create_type: CreateTableType::LikeOldTable {
                table: "old_tbl_name".into(),
            },
        }];
        for i in 0..sqls.len() {
            let res = CreateTableType::create_like_old_table(sqls[i]);
            assert!(res.is_ok());
            assert_eq!(res.unwrap().1, exp[i]);
        }
    }

    #[test]
    fn parse_create_definition_list() {
        let part = "(order_id INT not null, product_id INT DEFAULT 10,\
         PRIMARY KEY(order_id, product_id), FOREIGN KEY (product_id) REFERENCES product(id))";
        let exp = vec![
            CreateDefinition::ColumnDefinition {
                column_definition: ColumnSpecification {
                    column: "order_id".into(),
                    data_type: DataType::Int(32),
                    constraints: vec![ColumnConstraint::NotNull],
                    comment: None,
                    position: None,
                },
            },
            CreateDefinition::ColumnDefinition {
                column_definition: ColumnSpecification {
                    column: "product_id".into(),
                    data_type: DataType::Int(32),
                    constraints: vec![ColumnConstraint::DefaultValue(Literal::Integer(10))],
                    comment: None,
                    position: None,
                },
            },
            CreateDefinition::PrimaryKey {
                opt_symbol: None,
                opt_index_type: None,
                key_part: vec![
                    KeyPart {
                        r#type: KeyPartType::ColumnNameWithLength {
                            col_name: "order_id".to_string(),
                            length: None,
                        },
                        order: None,
                    },
                    KeyPart {
                        r#type: KeyPartType::ColumnNameWithLength {
                            col_name: "product_id".to_string(),
                            length: None,
                        },
                        order: None,
                    },
                ],
                opt_index_option: None,
            },
            CreateDefinition::ForeignKey {
                opt_symbol: None,
                opt_index_name: None,
                columns: vec!["product_id".to_string()],
                reference_definition: ReferenceDefinition {
                    tbl_name: "product".to_string(),
                    key_part: vec![KeyPart {
                        r#type: KeyPartType::ColumnNameWithLength {
                            col_name: "id".to_string(),
                            length: None,
                        },
                        order: None,
                    }],
                    match_type: None,
                    on_delete: None,
                    on_update: None,
                },
            },
        ];
        let res = CreateDefinition::create_definition_list(part);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().1, exp);
    }
}
