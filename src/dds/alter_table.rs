use core::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{alphanumeric1, anychar, digit1, multispace0, multispace1};
use nom::combinator::{map, opt, recognize};
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::{IResult, Parser};

use base::algorithm_type::AlgorithmType;
use base::check_constraint::CheckConstraintDefinition;
use base::column::{Column, ColumnSpecification};
use base::fulltext_or_spatial_type::FulltextOrSpatialType;
use base::index_option::IndexOption;
use base::index_or_key_type::IndexOrKeyType;
use base::index_type::IndexType;
use base::lock_type::LockType;
use base::table::Table;
use base::table_option::TableOption;
use base::visible_type::VisibleType;
use base::{CommonParser, KeyPart, ParseSQLError, PartitionDefinition, ReferenceDefinition};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AlterTableStatement {
    pub table: Table,
    pub alter_options: Option<Vec<AlterTableOption>>,
    pub partition_options: Option<Vec<AlterPartitionOption>>,
}

impl AlterTableStatement {
    /// ALTER TABLE tbl_name \[alter_option \[, alter_option] ...] \[partition_options]
    pub fn parse(i: &str) -> IResult<&str, AlterTableStatement, ParseSQLError<&str>> {
        let mut parser = tuple((
            tuple((
                tag_no_case("ALTER "),
                multispace0,
                tag_no_case("TABLE "),
                multispace0,
            )),
            // tbl_name
            Table::without_alias,
            multispace0,
            //
            opt(many0(map(
                tuple((
                    AlterTableOption::parse,
                    opt(CommonParser::ws_sep_comma),
                    multispace0,
                )),
                |x| x.0,
            ))),
            opt(many0(terminated(
                AlterPartitionOption::parse,
                opt(CommonParser::ws_sep_comma),
            ))),
            CommonParser::statement_terminator,
        ));
        let (remaining_input, (_, table, _, alter_options, partition_options, _)) = parser(i)?;
        Ok((
            remaining_input,
            AlterTableStatement {
                table,
                alter_options,
                partition_options,
            },
        ))
    }
}

impl fmt::Display for AlterTableStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let table_name = match &self.table.schema {
            Some(schema) => format!("{}.{}", schema, self.table.name),
            None => format!(" {}", self.table.name),
        };
        write!(f, "ALTER TABLE {} ", table_name)?;
        Ok(())
    }
}
/////// Alter Table Option

/// {CHECK | CONSTRAINT}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CheckOrConstraintType {
    Check,
    Constraint,
}

impl CheckOrConstraintType {
    fn parse(i: &str) -> IResult<&str, CheckOrConstraintType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("CHECK"), |_| CheckOrConstraintType::Check),
            map(tag_no_case("CONSTRAINT"), |_| {
                CheckOrConstraintType::Constraint
            }),
        ))(i)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AlterTableOption {
    /// table_options
    TableOptions { table_options: Vec<TableOption> },

    /// ADD \[COLUMN] col_name column_definition
    ///     \[FIRST | AFTER col_name]
    /// ADD \[COLUMN] (col_name column_definition,...)
    AddColumn {
        opt_column: bool, // [COLUMN]
        columns: Vec<ColumnSpecification>,
    },

    /// ADD {INDEX | KEY} \[index_name]
    ///     \[index_type] (key_part,...) \[index_option] ...
    AddIndexOrKey {
        index_or_key: IndexOrKeyType,          // {INDEX | KEY}
        opt_index_name: Option<String>,        // [index_name]
        opt_index_type: Option<IndexType>,     // [index_type]
        key_part: Vec<KeyPart>,                // (key_part,...)
        opt_index_option: Option<IndexOption>, // [index_option]
    },

    /// ADD {FULLTEXT | SPATIAL} \[INDEX | KEY] \[index_name]
    ///     (key_part,...) \[index_option] ...
    AddFulltextOrSpatial {
        fulltext_or_spatial: FulltextOrSpatialType, // {FULLTEXT | SPATIAL}
        index_or_key: Option<IndexOrKeyType>,       // {INDEX | KEY}
        opt_index_name: Option<String>,             // [index_name]
        key_part: Vec<KeyPart>,                     // (key_part,...)
        opt_index_option: Option<IndexOption>,      // [index_option]
    },

    /// ADD \[CONSTRAINT \[symbol]] PRIMARY KEY
    ///     \[index_type] (key_part,...)
    ///     \[index_option] ...
    AddPrimaryKey {
        opt_symbol: Option<String>,            // [symbol]
        opt_index_type: Option<IndexType>,     // [index_type]
        key_part: Vec<KeyPart>,                // (key_part,...)
        opt_index_option: Option<IndexOption>, // [index_option]
    },

    /// ADD \[CONSTRAINT \[symbol]] UNIQUE [INDEX | KEY]
    ///     \[index_name] \[index_type] (key_part,...)
    ///     \[index_option] ...
    AddUnique {
        opt_symbol: Option<String>,               // [symbol]
        opt_index_or_key: Option<IndexOrKeyType>, // [INDEX | KEY]
        opt_index_name: Option<String>,           // [index_name]
        opt_index_type: Option<IndexType>,        // [index_type]
        key_part: Vec<KeyPart>,                   // (key_part,...)
        opt_index_option: Option<IndexOption>,    // [index_option]
    },

    /// ADD \[CONSTRAINT \[symbol]] FOREIGN KEY
    ///     \[index_name] (col_name,...)
    ///     reference_definition
    AddForeignKey {
        opt_symbol: Option<String>,                // [symbol]
        opt_index_name: Option<String>,            // [index_name]
        columns: Vec<String>,                      // (col_name,...)
        reference_definition: ReferenceDefinition, // reference_definition
    },

    /// ADD \[CONSTRAINT \[symbol]] CHECK (expr) \[\[NOT] ENFORCED]
    AddCheck {
        check_constraint: CheckConstraintDefinition,
    },

    /// DROP {CHECK | CONSTRAINT} symbol
    DropCheckOrConstraint {
        check_or_constraint: CheckOrConstraintType,
        symbol: String,
    },

    /// ALTER {CHECK | CONSTRAINT} symbol \[NOT] ENFORCED
    AlterCheckOrConstraintEnforced {
        check_or_constraint: CheckOrConstraintType,
        symbol: String,
        enforced: bool,
    },

    /// ALGORITHM \[=] {DEFAULT | INSTANT | INPLACE | COPY}
    Algorithm { algorithm: AlgorithmType },

    /// ALTER \[COLUMN] col_name { SET DEFAULT {literal | (expr)} | SET {VISIBLE | INVISIBLE} | DROP DEFAULT }
    AlterColumn {
        col_name: String,
        alter_column_operation: AlertColumnOperation,
    },

    /// ALTER INDEX index_name {VISIBLE | INVISIBLE}
    AlterIndexVisibility {
        index_name: String,
        visible: VisibleType,
    },

    /// CHANGE \[COLUMN] old_col_name new_col_name column_definition \[FIRST | AFTER col_name]
    ChangeColumn {
        old_col_name: String,
        column_definition: ColumnSpecification,
    },

    /// \[DEFAULT] CHARACTER SET \[=] charset_name [COLLATE \[=] collation_name]
    DefaultCharacterSet {
        charset_name: String,
        collation_name: Option<String>,
    },

    /// CONVERT TO CHARACTER SET charset_name \[COLLATE collation_name]
    ConvertToCharacterSet {
        charset_name: String,
        collation_name: Option<String>,
    },

    /// {DISABLE | ENABLE} KEYS
    DisableKeys,

    /// {DISABLE | ENABLE} KEYS
    EnableKeys,

    /// {DISCARD | IMPORT} TABLESPACE
    DiscardTablespace,

    /// {DISCARD | IMPORT} TABLESPACE
    ImportTablespace,

    /// DROP \[COLUMN] col_name
    DropColumn { col_name: String },

    /// DROP {INDEX | KEY} index_name
    DropIndexOrKey {
        index_or_key: IndexOrKeyType,
        index_name: String,
    },

    /// DROP PRIMARY KEY
    DropPrimaryKey,

    /// DROP FOREIGN KEY fk_symbol
    DropForeignKey { fk_symbol: String },

    /// FORCE
    Force,

    /// LOCK \[=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
    Lock { lock_type: LockType },

    /// MODIFY \[COLUMN] col_name column_definition \[FIRST | AFTER col_name]
    ModifyColumn {
        column_definition: ColumnSpecification,
    },

    /// ORDER BY col_name \[, col_name] ...
    OrderBy { columns: Vec<String> },

    /// RENAME COLUMN old_col_name TO new_col_name
    RenameColumn {
        old_col_name: String,
        new_col_name: String,
    },

    /// RENAME {INDEX | KEY} old_index_name TO new_index_name
    RenameIndexOrKey {
        index_or_key: IndexOrKeyType,
        old_index_name: String,
        new_index_name: String,
    },

    /// RENAME \[TO | AS] new_tbl_name
    RenameTable { new_tbl_name: String },

    /// {WITHOUT | WITH} VALIDATION
    Validation { with_validation: bool },
}

impl AlterTableOption {
    fn parse(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        let mut parser = alt((
            Self::alter_table_options,
            Self::alter_option_part_1,
            Self::alter_option_part_2,
        ));
        let (remaining_input, res) = parser(i)?;
        Ok((remaining_input, res))
    }

    /// table_options:
    ///     table_option \[\[,] table_option] ...
    pub fn alter_table_options(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            many1(terminated(
                TableOption::parse,
                opt(CommonParser::ws_sep_comma),
            )),
            |table_options| AlterTableOption::TableOptions { table_options },
        )(i)
    }

    fn alter_option_part_1(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        alt((
            Self::add_column,
            Self::add_index_or_key,
            Self::add_fulltext_or_spatial,
            Self::add_primary_key,
            Self::add_unique,
            Self::add_foreign_key,
            Self::add_check,
            Self::drop_check_or_constraint,
            Self::alter_check_or_constraint_enforced,
            map(AlgorithmType::parse, |x| AlterTableOption::Algorithm {
                algorithm: x,
            }),
            Self::alter_column,
            Self::alter_index_visibility,
            Self::change_column,
            Self::default_character_set,
        ))(i)
    }

    fn alter_option_part_2(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        alt((
            Self::convert_to_character_set,
            Self::disable_or_enable_keys,
            Self::discard_or_import_tablespace,
            Self::drop_column,
            Self::drop_index_or_key,
            Self::drop_primary_key,
            Self::drop_foreign_key,
            Self::force,
            Self::lock,
            Self::modify_column,
            Self::order_by,
            Self::rename_column,
            Self::rename_index_or_key,
            Self::rename_table,
            Self::without_or_with_validation,
        ))(i)
    }

    /// \[CONSTRAINT \[symbol]]
    fn opt_constraint_with_opt_symbol_and_operation(
        i: &str,
    ) -> IResult<&str, Option<String>, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ADD"),
                opt(preceded(
                    tuple((multispace1, tag_no_case("CONSTRAINT"))),
                    opt(preceded(multispace1, CommonParser::sql_identifier)),
                )),
            )),
            |(_, x)| x.and_then(|inner| inner.map(String::from)),
        )(i)
    }

    ///  | ADD \[COLUMN] col_name column_definition
    ///        \[FIRST | AFTER col_name]
    ///  | ADD \[COLUMN] (col_name column_definition,...)
    fn add_column(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ADD"),
                alt((
                    map(
                        tuple((
                            multispace1,
                            tag_no_case("COLUMN"),
                            multispace1,
                            ColumnSpecification::parse,
                            multispace0,
                        )),
                        |x| (true, vec![x.3]),
                    ),
                    map(
                        tuple((
                            multispace1,
                            tag_no_case("COLUMN"),
                            multispace0,
                            tag("("),
                            multispace0,
                            many1(ColumnSpecification::parse),
                            multispace0,
                            tag(")"),
                        )),
                        |x| (true, x.5),
                    ),
                    map(tuple((multispace0, ColumnSpecification::parse)), |x| {
                        (false, vec![x.1])
                    }),
                    map(
                        tuple((
                            multispace0,
                            tag("("),
                            multispace0,
                            many1(ColumnSpecification::parse),
                            multispace0,
                            tag(")"),
                        )),
                        |x| (false, x.3),
                    ),
                )),
            )),
            |(_, (opt_column, columns))| AlterTableOption::AddColumn {
                opt_column,
                columns,
            },
        )(i)
    }

    /// ADD {INDEX | KEY} \[index_name] \[index_type] (key_part,...) \[index_option] ...
    fn add_index_or_key(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("ADD"), multispace1)),
                // {INDEX | KEY}
                IndexOrKeyType::parse,
                // [index_name]
                CommonParser::opt_index_name,
                // [index_type]
                IndexType::opt_index_type,
                // (key_part,...)
                KeyPart::key_part_list,
                // [index_option]
                IndexOption::opt_index_option,
            )),
            |(_, index_or_key, opt_index_name, opt_index_type, key_part, opt_index_option)| {
                AlterTableOption::AddIndexOrKey {
                    index_or_key,
                    opt_index_name,
                    opt_index_type,
                    key_part,
                    opt_index_option,
                }
            },
        )(i)
    }

    /// | ADD {FULLTEXT | SPATIAL} \[INDEX | KEY] \[index_name] (key_part,...) \[index_option] ...
    fn add_fulltext_or_spatial(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("ADD"), multispace1)),
                // {FULLTEXT | SPATIAL}
                FulltextOrSpatialType::parse,
                // [INDEX | KEY]
                preceded(multispace1, opt(IndexOrKeyType::parse)),
                // [index_name]
                CommonParser::opt_index_name,
                // (key_part,...)
                KeyPart::key_part_list,
                // [index_option]
                IndexOption::opt_index_option,
            )),
            |(_, fulltext_or_spatial, index_or_key, opt_index_name, key_part, opt_index_option)| {
                AlterTableOption::AddFulltextOrSpatial {
                    fulltext_or_spatial,
                    index_or_key,
                    opt_index_name,
                    key_part,
                    opt_index_option,
                }
            },
        )(i)
    }

    /// | ADD \[CONSTRAINT \[symbol]] PRIMARY KEY \[index_type] (key_part,...) \[index_option] ...
    fn add_primary_key(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                // [CONSTRAINT [symbol]]
                Self::opt_constraint_with_opt_symbol_and_operation,
                // PRIMARY KEY
                tuple((
                    multispace0,
                    tag_no_case("PRIMARY"),
                    multispace1,
                    tag_no_case("KEY"),
                )),
                // [index_type]
                IndexType::opt_index_type,
                // (key_part,...)
                KeyPart::key_part_list,
                // [index_option]
                IndexOption::opt_index_option,
            )),
            |(opt_symbol, _, opt_index_type, key_part, opt_index_option)| {
                AlterTableOption::AddPrimaryKey {
                    opt_symbol,
                    opt_index_type,
                    key_part,
                    opt_index_option,
                }
            },
        )(i)
    }

    /// | ADD \[CONSTRAINT \[symbol]] UNIQUE \[INDEX | KEY] \[index_name] \[index_type] (key_part,...) \[index_option] ...
    fn add_unique(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                // [CONSTRAINT [symbol]]
                Self::opt_constraint_with_opt_symbol_and_operation,
                // UNIQUE [INDEX | KEY]
                map(
                    tuple((
                        multispace0,
                        tag_no_case("UNIQUE"),
                        multispace1,
                        opt(alt((
                            map(tag_no_case("INDEX"), |_| IndexOrKeyType::Index),
                            map(tag_no_case("KEY"), |_| IndexOrKeyType::Key),
                        ))),
                    )),
                    |(_, _, _, value)| value,
                ),
                // [index_name]
                CommonParser::opt_index_name,
                // [index_type]
                IndexType::opt_index_type,
                // (key_part,...)
                KeyPart::key_part_list,
                // [index_option]
                IndexOption::opt_index_option,
            )),
            |(
                opt_symbol,
                opt_index_or_key,
                opt_index_name,
                opt_index_type,
                key_part,
                opt_index_option,
            )| {
                AlterTableOption::AddUnique {
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

    /// ADD \[CONSTRAINT \[symbol]] FOREIGN KEY \[index_name] (col_name,...) reference_definition
    fn add_foreign_key(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                // [CONSTRAINT [symbol]]
                Self::opt_constraint_with_opt_symbol_and_operation,
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
                    )),
                    |(_, value)| value.iter().map(|x| x.name.clone()).collect(),
                ),
                // reference_definition
                ReferenceDefinition::parse,
            )),
            |(opt_symbol, _, opt_index_name, columns, reference_definition)| {
                AlterTableOption::AddForeignKey {
                    opt_symbol,
                    opt_index_name,
                    columns,
                    reference_definition,
                }
            },
        )(i)
    }

    /// | ADD \[CONSTRAINT \[symbol]] CHECK (expr) \[\[NOT] ENFORCED]
    fn add_check(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                // [CONSTRAINT [symbol]]
                Self::opt_constraint_with_opt_symbol_and_operation,
                // CHECK
                tuple((multispace1, tag_no_case("CHECK"), multispace0)),
                // (expr)
                map(delimited(tag("("), take_until(")"), tag(")")), |expr| {
                    String::from(expr)
                }),
                // [[NOT] ENFORCED]
                map(
                    opt(tuple((
                        multispace0,
                        opt(tag_no_case("NOT")),
                        multispace1,
                        tag_no_case("ENFORCED"),
                        multispace0,
                    ))),
                    |x| x.map_or(false, |(_, opt_not, _, _, _)| opt_not.is_none()),
                ),
            )),
            |(symbol, _, expr, enforced)| AlterTableOption::AddCheck {
                check_constraint: CheckConstraintDefinition {
                    symbol,
                    expr,
                    enforced,
                },
            },
        )(i)
    }

    /// DROP {CHECK | CONSTRAINT} symbol
    fn drop_check_or_constraint(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("DROP"), multispace1)),
                // {CHECK | CONSTRAINT}
                CheckOrConstraintType::parse,
                // symbol
                map(
                    tuple((multispace1, CommonParser::sql_identifier, multispace0)),
                    |(_, symbol, _)| String::from(symbol),
                ),
            )),
            |(_, check_or_constraint, symbol)| AlterTableOption::DropCheckOrConstraint {
                check_or_constraint,
                symbol,
            },
        )(i)
    }

    /// ALTER {CHECK | CONSTRAINT} symbol \[NOT] ENFORCED
    fn alter_check_or_constraint_enforced(
        i: &str,
    ) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("ALTER"), multispace1)),
                // {CHECK | CONSTRAINT}
                CheckOrConstraintType::parse,
                // symbol
                map(
                    tuple((multispace1, CommonParser::sql_identifier, multispace1)),
                    |(_, symbol, _)| String::from(symbol),
                ),
                opt(tag_no_case("NOT ")),
                tuple((multispace0, tag_no_case("ENFORCED"))),
            )),
            |(_, check_or_constraint, symbol, opt_not, _)| {
                AlterTableOption::AlterCheckOrConstraintEnforced {
                    check_or_constraint,
                    symbol,
                    enforced: opt_not.is_none(),
                }
            },
        )(i)
    }

    /// ALTER \[COLUMN] col_name {
    ///   SET DEFAULT {literal | (expr)}
    ///   | SET {VISIBLE | INVISIBLE}
    ///   | DROP DEFAULT
    /// }
    fn alter_column(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ALTER "),
                multispace0,
                opt(tag_no_case("COLUMN ")),
                // col_name
                map(
                    tuple((multispace0, CommonParser::sql_identifier, multispace1)),
                    |(_, col_name, _)| String::from(col_name),
                ),
                AlertColumnOperation::parse,
                multispace0,
            )),
            |(_, _, _, col_name, alter_column_operation, _)| AlterTableOption::AlterColumn {
                col_name,
                alter_column_operation,
            },
        )(i)
    }

    /// ALTER INDEX index_name {VISIBLE | INVISIBLE}
    fn alter_index_visibility(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ALTER "),
                multispace0,
                opt(tag_no_case("INDEX ")),
                // index_name
                map(
                    tuple((multispace0, CommonParser::sql_identifier, multispace1)),
                    |(_, col_name, _)| String::from(col_name),
                ),
                VisibleType::parse,
                multispace0,
            )),
            |(_, _, _, index_name, visible, _)| AlterTableOption::AlterIndexVisibility {
                index_name,
                visible,
            },
        )(i)
    }

    /// CHANGE \[COLUMN] old_col_name new_col_name column_definition \[FIRST | AFTER col_name]
    fn change_column(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("CHANGE "),
                multispace0,
                opt(tag_no_case("COLUMN ")),
                multispace0,
                // old_col_name
                map(CommonParser::sql_identifier, String::from),
                multispace1,
                ColumnSpecification::parse,
                multispace0,
            )),
            |(_, _, _, _, old_col_name, _, column_definition, _)| AlterTableOption::ChangeColumn {
                old_col_name,
                column_definition,
            },
        )(i)
    }

    /// \[DEFAULT] CHARACTER SET \[=] charset_name [COLLATE \[=] collation_name]
    fn default_character_set(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                opt(tag_no_case("DEFAULT ")),
                multispace0,
                tuple((
                    multispace0,
                    tag_no_case("CHARACTER"),
                    multispace1,
                    tag_no_case("SET"),
                    multispace0,
                    opt(tag("=")),
                    multispace0,
                )),
                map(CommonParser::sql_identifier, String::from),
                multispace0,
                opt(map(
                    tuple((
                        multispace0,
                        tag_no_case("COLLATE"),
                        multispace1,
                        CommonParser::sql_identifier,
                    )),
                    |(_, _, _, collation_name)| String::from(collation_name),
                )),
            )),
            |(_, _, _, charset_name, _, collation_name)| AlterTableOption::DefaultCharacterSet {
                charset_name,
                collation_name,
            },
        )(i)
    }

    /// CONVERT TO CHARACTER SET charset_name \[COLLATE collation_name]
    fn convert_to_character_set(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        let prefix = tuple((
            tag_no_case("CONVERT"),
            multispace1,
            tag_no_case("TO"),
            multispace1,
            tag_no_case("CHARACTER"),
            multispace1,
            tag_no_case("SET"),
            multispace1,
        ));
        map(
            tuple((
                // CONVERT TO CHARACTER SET
                prefix,
                map(CommonParser::sql_identifier, String::from),
                multispace0,
                opt(map(
                    tuple((
                        multispace0,
                        tag_no_case("COLLATE"),
                        multispace1,
                        CommonParser::sql_identifier,
                    )),
                    |(_, _, _, collation_name)| String::from(collation_name),
                )),
            )),
            |(_, charset_name, _, collation_name)| AlterTableOption::ConvertToCharacterSet {
                charset_name,
                collation_name,
            },
        )(i)
    }

    /// {DISCARD | IMPORT} TABLESPACE
    fn disable_or_enable_keys(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                alt((
                    map(tag_no_case("DISABLE"), |_| AlterTableOption::DisableKeys),
                    map(tag_no_case("ENABLE"), |_| AlterTableOption::EnableKeys),
                )),
                multispace1,
                tag_no_case("KEYS"),
                multispace0,
            )),
            |(operation, _, _, _)| operation,
        )(i)
    }

    /// {DISCARD | IMPORT} TABLESPACE
    fn discard_or_import_tablespace(
        i: &str,
    ) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                alt((
                    map(tag_no_case("DISCARD"), |_| {
                        AlterTableOption::DiscardTablespace
                    }),
                    map(tag_no_case("IMPORT"), |_| {
                        AlterTableOption::ImportTablespace
                    }),
                )),
                multispace1,
                tag_no_case("TABLESPACE"),
                multispace0,
            )),
            |(operation, _, _, _)| operation,
        )(i)
    }

    /// DROP \[COLUMN] col_name
    fn drop_column(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("DROP "),
                multispace0,
                opt(tag_no_case("COLUMN ")),
                // col_name
                map(
                    tuple((multispace0, CommonParser::sql_identifier, multispace0)),
                    |(_, col_name, _)| String::from(col_name),
                ),
                multispace0,
            )),
            |(_, _, _, col_name, _)| AlterTableOption::DropColumn { col_name },
        )(i)
    }

    /// DROP {INDEX | KEY} index_name
    fn drop_index_or_key(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("DROP"), multispace1)),
                // {INDEX | KEY}
                IndexOrKeyType::parse,
                // [index_name]
                map(
                    tuple((multispace1, CommonParser::sql_identifier, multispace0)),
                    |(_, index_name, _)| String::from(index_name),
                ),
                multispace0,
            )),
            |(_, index_or_key, index_name, _)| AlterTableOption::DropIndexOrKey {
                index_or_key,
                index_name,
            },
        )(i)
    }

    /// DROP PRIMARY KEY
    fn drop_primary_key(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("DROP"),
                multispace1,
                tag_no_case("PRIMARY"),
                multispace1,
                tag_no_case("KEY"),
                multispace0,
            )),
            |_| AlterTableOption::DropPrimaryKey,
        )(i)
    }

    /// DROP FOREIGN KEY fk_symbol
    fn drop_foreign_key(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("DROP"),
                multispace1,
                tag_no_case("FOREIGN"),
                multispace1,
                tag_no_case("KEY"),
                multispace1,
                map(CommonParser::sql_identifier, String::from),
                multispace0,
            )),
            |x| AlterTableOption::DropForeignKey { fk_symbol: x.6 },
        )(i)
    }

    /// FORCE
    fn force(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(tuple((tag_no_case("FORCE"), multispace0)), |_| {
            AlterTableOption::Force
        })(i)
    }

    /// LOCK \[=] {DEFAULT | NONE | SHARED | EXCLUSIVE}
    fn lock(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(LockType::parse, |(lock_type)| AlterTableOption::Lock {
            lock_type,
        })(i)
    }

    /// MODIFY \[COLUMN] col_name column_definition \[FIRST | AFTER col_name]
    fn modify_column(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("MODIFY "),
                multispace0,
                opt(tag_no_case("COLUMN ")),
                multispace0,
                ColumnSpecification::parse,
                multispace0,
            )),
            |(_, _, _, _, column_definition, _)| AlterTableOption::ModifyColumn {
                column_definition,
            },
        )(i)
    }

    /// ORDER BY col_name \[, col_name] ...
    fn order_by(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("ORDER"),
                multispace1,
                tag_no_case("BY"),
                multispace1,
                many0(map(
                    terminated(Column::without_alias, opt(CommonParser::ws_sep_comma)),
                    |e| e.name,
                )),
                multispace0,
            )),
            |(_, _, _, _, columns, _)| AlterTableOption::OrderBy { columns },
        )(i)
    }

    /// RENAME COLUMN old_col_name TO new_col_name
    fn rename_column(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tag_no_case("RENAME "),
                multispace0,
                opt(tag_no_case("COLUMN ")),
                multispace0,
                // old_col_name
                map(CommonParser::sql_identifier, String::from),
                multispace1,
                tag_no_case("TO"),
                multispace1,
                // new_col_name
                map(CommonParser::sql_identifier, String::from),
                multispace0,
            )),
            |(_, _, _, _, old_col_name, _, _, _, new_col_name, _)| AlterTableOption::RenameColumn {
                old_col_name,
                new_col_name,
            },
        )(i)
    }

    /// RENAME {INDEX | KEY} old_index_name TO new_index_name
    fn rename_index_or_key(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("RENAME"), multispace1)),
                // {INDEX | KEY}
                IndexOrKeyType::parse,
                // old_index_name
                map(
                    tuple((multispace1, CommonParser::sql_identifier, multispace1)),
                    |(_, index_name, _)| String::from(index_name),
                ),
                tuple((multispace1, tag_no_case("TO"))),
                // new_index_name
                map(
                    tuple((multispace1, CommonParser::sql_identifier, multispace1)),
                    |(_, index_name, _)| String::from(index_name),
                ),
                multispace0,
            )),
            |(_, index_or_key, old_index_name, _, new_index_name, _)| {
                AlterTableOption::RenameIndexOrKey {
                    index_or_key,
                    old_index_name,
                    new_index_name,
                }
            },
        )(i)
    }

    /// RENAME \[TO | AS] new_tbl_name
    fn rename_table(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                tuple((tag_no_case("RENAME"), multispace1)),
                // {INDEX | KEY}
                alt((tag_no_case("TO"), tag_no_case("AS"))),
                // new_tbl_name
                map(
                    tuple((multispace1, CommonParser::sql_identifier, multispace0)),
                    |(_, index_name, _)| String::from(index_name),
                ),
                multispace0,
            )),
            |x| AlterTableOption::RenameTable { new_tbl_name: x.2 },
        )(i)
    }

    /// {WITHOUT | WITH} VALIDATION
    fn without_or_with_validation(i: &str) -> IResult<&str, AlterTableOption, ParseSQLError<&str>> {
        map(
            tuple((
                // {WITHOUT | WITH}
                alt((
                    map(tag_no_case("WITHOUT"), |_| false),
                    map(tag_no_case("WITH"), |_| true),
                )),
                multispace1,
                tag_no_case("VALIDATION"),
                multispace0,
            )),
            |x| AlterTableOption::Validation {
                with_validation: x.0,
            },
        )(i)
    }
}

/// { SET DEFAULT {literal | (expr)} | SET {VISIBLE | INVISIBLE} | DROP DEFAULT }
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AlertColumnOperation {
    SetDefaultLiteral(String),
    SetDefaultExpr(String),
    SetVisible(VisibleType),
    DropDefault,
}

impl AlertColumnOperation {
    fn parse(i: &str) -> IResult<&str, AlertColumnOperation, ParseSQLError<&str>> {
        alt((
            map(
                tuple((
                    tag_no_case("SET"),
                    multispace1,
                    tag_no_case("DEFAULT"),
                    multispace1,
                    alt((
                        map(
                            alt((recognize(tuple((opt(tag("-")), digit1))), alphanumeric1)),
                            |x| AlertColumnOperation::SetDefaultLiteral(String::from(x)),
                        ),
                        map(
                            delimited(tag("("), recognize(many1(anychar)), tag(")")),
                            |x| AlertColumnOperation::SetDefaultExpr(String::from(x)),
                        ),
                    )),
                    multispace0,
                )),
                |x| x.4,
            ),
            map(
                tuple((
                    tag_no_case("SET"),
                    multispace1,
                    VisibleType::parse,
                    multispace0,
                )),
                |x| AlertColumnOperation::SetVisible(x.2),
            ),
            map(
                tuple((
                    tag_no_case("DROP"),
                    multispace1,
                    tag_no_case("DEFAULT"),
                    multispace0,
                )),
                |_| AlertColumnOperation::DropDefault,
            ),
        ))(i)
    }
}

////////////// TODO support alter partition parser
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AlterPartitionOption {
    None,
    AddPartition(PartitionDefinition),
    DropPartition(String),
    DiscardPartition,
    ImportPartition,
    TruncatePartition,
    CoalescePartition,
    ReorganizePartitionInto,
    ExchangePartitionWithTable,
    AnalyzePartition,
    CheckPartition,
    OptimizePartition,
    RebuildPartition,
    RepairPartition,
    RemovePartitioning,
}

impl AlterPartitionOption {
    pub fn parse(i: &str) -> IResult<&str, AlterPartitionOption, ParseSQLError<&str>> {
        map(tag_no_case(""), |_| AlterPartitionOption::None)(i)
    }
}
////////////// TODO support alter partition parser

#[cfg(test)]
mod tests {
    use base::column::{ColumnConstraint, ColumnPosition, ColumnSpecification};
    use base::index_option::IndexOption;
    use base::Literal;
    use dds::alter_table::{AlterTableOption, AlterTableStatement};

    #[test]
    fn test_add_column() {
        let parts = [
            r###"ADD COLUMN filter   TINYINT(4) DEFAULT 0 COMMENT '统计过滤(1=启用过滤；0=禁用过滤)'"###,
            r###"ADD COLUMN filter_name VARCHAR(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '统计筛选字段名称';"###,
            r###"ADD replace_approver varchar(512) NOT NULL DEFAULT "0" COMMENT '指定代审员工'"###,
            "ADD template_admin_approver json NULL DEFAULT NULL COMMENT '模板管理员'",
            "ADD COLUMN (new_column8 INT, new_column9 VARCHAR(100));",
            "ADD COLUMN column1 VARCHAR(255)",
            "ADD column2 INT DEFAULT 10",
            "ADD COLUMN column3 DATE NOT NULL",
            "ADD COLUMN column4 TEXT UNIQUE;",
            "ADD column4 TEXT UNIQUE;",
            "ADD COLUMN column5 DECIMAL(10, 2)",
            "ADD column7 ENUM('small', 'medium', 'large')",
            "ADD COLUMN column7 ENUM('small', 'medium', 'large')",
            "ADD column8 BLOB",
            "ADD column9 VARCHAR(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
            "ADD COLUMN new_column2 VARCHAR(255) FIRST;",
            "ADD COLUMN new_column3 DATE AFTER existing_column;",
            "ADD COLUMN new_column5 TEXT COMMENT 'This is a comment';",
            "ADD new_column6 DECIMAL(10,2) NOT NULL;",
            "ADD COLUMN new_column8 INT",
            "ADD COLUMN new_column9 VARCHAR(100)",
            "ADD new_column10 TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
            "ADD COLUMN new_column11 VARCHAR(50) NOT NULL UNIQUE;",
            "ADD (new_column10 TIMESTAMP DEFAULT CURRENT_TIMESTAMP, new_column11 VARCHAR(50) NOT NULL UNIQUE);",
            "ADD column6 TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP;",
            "ADD new_column4 BOOLEAN DEFAULT FALSE;",
            "ADD new_column7 UUID UNIQUE;",
        ];
        for i in 0..parts.len() {
            println!("{}/{}", i + 1, parts.len());
            let res = AlterTableOption::add_column(parts[i]);
            // // res.unwrap();
            println!("{:?}", res);
            assert!(res.is_ok());
        }

        let sql = "ADD name VARCHAR(128) NULL DEFAULT NULL AFTER age";
        let res = AlterTableOption::add_column(sql);
        assert!(res.is_ok());
        if let (
            _,
            AlterTableOption::AddColumn {
                opt_column: bl,
                columns: cols,
            },
        ) = res.unwrap()
        {
            assert_eq!(cols.len(), 1);
            assert_eq!(
                cols[0].constraints,
                vec![
                    ColumnConstraint::Null,
                    ColumnConstraint::DefaultValue(Literal::Null)
                ]
            );
        }
    }

    #[test]
    fn test_position() {
        let parts = [
            "FIRST",
            " FIRST",
            " FIRST ",
            "AFTER foo",
            " AFTER foo ",
            "  AFTER  foo ",
        ];
        let positions = vec![
            ColumnPosition::First,
            ColumnPosition::First,
            ColumnPosition::First,
            ColumnPosition::After("foo".into()),
            ColumnPosition::After("foo".into()),
            ColumnPosition::After("foo".into()),
        ];
        for i in 0..parts.len() {
            let (r, res) = ColumnPosition::parse(parts[i]).unwrap();
            assert_eq!(res, positions[i])
        }
    }

    #[test]
    fn test_column_definition() {
        let parts = [
            "column1 VARCHAR(255)",
            "column2 INT DEFAULT 10",
            "column3 DATE NOT NULL",
            "column4 TEXT UNIQUE",
            "column5 DECIMAL(10, 2)",
            "column6 TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP",
            "column7 ENUM('small', 'medium', 'large')",
            "column7 ENUM('small', 'medium', 'large')",
            "column8 BLOB",
            "column9 VARCHAR(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
            "new_column2 VARCHAR(255) FIRST;",
            "new_column3 DATE AFTER existing_column;",
            "new_column4 BOOLEAN DEFAULT FALSE;",
            "new_column5 TEXT COMMENT 'This is a comment';",
            "new_column6 DECIMAL(10,2) NOT NULL;",
            "new_column7 UUID UNIQUE;",
            "new_column8 INT",
            "new_column9 VARCHAR(100)",
            "new_column10 TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
            "new_column11 VARCHAR(50) NOT NULL UNIQUE",
        ];
        for i in 0..parts.len() {
            println!("{} / {}", i + 1, parts.len());
            let res = ColumnSpecification::parse(parts[i]);
            println!("{:?}", res);
            assert!(res.is_ok())
        }
    }

    #[test]
    fn test_add_index_or_key() {
        let parts = [
            "ADD INDEX index_name (column_name);",
            "ADD KEY key_name (column_name);",
            "ADD INDEX index_name (column_name) USING BTREE;",
            "ADD INDEX index_name (column_name) KEY_BLOCK_SIZE=1024;",
            "ADD INDEX index_name (column_name) COMMENT 'This is an index comment';",
            "ADD INDEX index_name (column_name) INVISIBLE;",
            "ADD INDEX comp_index_name (column1, column2);",
            "ADD INDEX index_name (column_name(10));",
        ];
        for i in 0..parts.len() {
            println!("{}/{}", i + 1, parts.len());
            let res = AlterTableOption::add_index_or_key(parts[i]);
            // res.unwrap();
            assert!(res.is_ok());
            println!("{:?}", res.unwrap().1)
        }
    }

    #[test]
    fn test_add_fulltext_or_spatial() {
        let parts = [
            "ADD FULLTEXT INDEX ft_index_name (column_name);",
            "ADD FULLTEXT INDEX ft_index_name (column_name) KEY_BLOCK_SIZE=1024 COMMENT 'Fulltext index on column_name' WITH PARSER ngram VISIBLE;",
            "ADD SPATIAL INDEX sp_index_name (column_name);",
            "ADD FULLTEXT INDEX sp_index_name (column_name);",
        ];
        for i in 0..parts.len() {
            println!("{}/{}", i + 1, parts.len());
            let res = AlterTableOption::add_fulltext_or_spatial(parts[i]);
            assert!(res.is_ok());
            println!("{:?}", res.unwrap().1)
        }
    }

    #[test]
    fn test_index_option() {
        let parts = [
            "KEY_BLOCK_SIZE=1024",
            "COMMENT 'This is an index comment'",
            "INVISIBLE",
            "KEY_BLOCK_SIZE=1024 COMMENT 'Fulltext index on column_name' WITH PARSER ngram VISIBLE",
            "USING BTREE",
        ];
        for i in 0..parts.len() {
            println!("{}/{}", i + 1, parts.len());
            let res = IndexOption::parse(parts[i]);
            // res.unwrap();
            // println!("{:?}", res);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_add_unique() {
        let parts = ["ADD CONSTRAINT UNIQUE (col_19)"];
        for i in 0..parts.len() {
            println!("{}/{}", i + 1, parts.len());
            let res = AlterTableOption::add_unique(parts[i]);
            // res.unwrap();
            // println!("{:?}", res);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_convert_to_character_set() {
        let parts = ["CONVERT TO CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci"];
        for i in 0..parts.len() {
            println!("{}/{}", i + 1, parts.len());
            let res = AlterTableOption::convert_to_character_set(parts[i]);
            // res.unwrap();
            // println!("{:?}", res);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_add_primary_key() {
        let parts = ["ADD PRIMARY KEY (new_column)"];
        for i in 0..parts.len() {
            println!("{}/{}", i + 1, parts.len());
            let res = AlterTableOption::add_primary_key(parts[i]);
            // res.unwrap();
            // println!("{:?}", res);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_add_check() {
        let parts = ["ADD CONSTRAINT chk_column CHECK (new_column > 0) NOT ENFORCED;"];
        for i in 0..parts.len() {
            println!("{}/{}", i + 1, parts.len());
            let res = AlterTableOption::add_check(parts[i]);
            // res.unwrap();
            // println!("{:?}", res);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_modify_column() {
        let parts = ["MODIFY COLUMN another_column VARCHAR(255) FIRST;"];
        for i in 0..parts.len() {
            println!("{}/{}", i + 1, parts.len());
            let res = AlterTableOption::modify_column(parts[i]);
            // res.unwrap();
            assert!(res.is_ok());
            println!("{:?}", res);
        }
    }

    #[test]
    fn test_alter_table() {
        let alter_sqls = vec![
            r###"ALTER TABLE common_stats.event_event_attr_link ADD COLUMN filter   TINYINT(4) DEFAULT 0 COMMENT '统计过滤(1=启用过滤；0=禁用过滤)', ADD COLUMN filter_name VARCHAR(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '统计筛选字段名称';"###,
            "ALTER TABLE tbl_order DISABLE KEYS",
            "ALTER TABLE tbl_order ORDER BY col_3",
            "ALTER TABLE tbl_customer ENABLE KEYS",
            "ALTER TABLE tbl_order DROP COLUMN col_6",
            "ALTER TABLE tbl_order RENAME TO tbl_customer_31",
            "ALTER TABLE tbl_order ADD INDEX idx_34 (col_14)",
            "ALTER TABLE tbl_customer ADD COLUMN col_74 VARCHAR(255)",
            "ALTER TABLE tbl_customer RENAME COLUMN col_20 TO col_30",
            "ALTER TABLE tbl_product CHANGE COLUMN col_1 col_21 DATE",
            "ALTER TABLE tbl_inventory ADD CONSTRAINT UNIQUE (col_19)",
            "ALTER TABLE tbl_order ADD FULLTEXT INDEX ft_idx_87 (col_1)",
            "ALTER TABLE tbl_product CONVERT TO CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
            "ALTER TABLE test_table ADD COLUMN new_column INT;",
            "ALTER TABLE test_table ADD COLUMN another_column VARCHAR(255) AFTER new_column;",
            "ALTER TABLE test_table ADD INDEX (new_column);",
            "ALTER TABLE test_table ADD FULLTEXT INDEX (another_column);",
            "ALTER TABLE test_table ADD SPATIAL INDEX (another_column);",
            "ALTER TABLE test_table ADD CONSTRAINT fk_example FOREIGN KEY (new_column) REFERENCES other_table(other_column);",
            "ALTER TABLE test_table ADD CONSTRAINT chk_column CHECK (new_column > 0) NOT ENFORCED;",
            "ALTER TABLE test_table DROP CHECK chk_column;",
            "ALTER TABLE test_table ALTER CHECK chk_column NOT ENFORCED;",
            "ALTER TABLE test_table ENGINE = InnoDB;",
            "ALTER TABLE test_table MODIFY COLUMN new_column BIGINT NOT NULL;",
            "ALTER TABLE test_table ALTER COLUMN new_column SET DEFAULT 10;",
            "ALTER TABLE test_table ALTER COLUMN new_column DROP DEFAULT;",
            "ALTER TABLE test_table MODIFY COLUMN another_column VARCHAR(255) FIRST;",
            "ALTER TABLE test_table RENAME COLUMN another_column TO renamed_column;",
            "ALTER TABLE test_table DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
            "ALTER TABLE test_table CONVERT TO CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
            "ALTER TABLE test_table DISABLE KEYS;",
            "ALTER TABLE test_table ENABLE KEYS;",
            "ALTER TABLE test_table DISCARD TABLESPACE;",
            "ALTER TABLE test_table IMPORT TABLESPACE;",
            "ALTER TABLE test_table DROP COLUMN renamed_column;",
            "ALTER TABLE test_table DROP INDEX unique_index_name;",
            "ALTER TABLE test_table DROP PRIMARY KEY;",
            "ALTER TABLE test_table DROP FOREIGN KEY fk_example;",
            "ALTER TABLE test_table FORCE;",
            "ALTER TABLE test_table ALTER INDEX index_name VISIBLE;",
            "ALTER TABLE test_table ALTER INDEX index_name INVISIBLE;",
            "ALTER TABLE test_table ADD PRIMARY KEY (new_column);",
            "ALTER TABLE test_table ADD UNIQUE INDEX unique_index_name (another_column);",
            "ALTER TABLE tbl_product ADD COLUMN col_name160 VARCHAR(255) NOT NULL",
            "ALTER TABLE tbl_customer DROP COLUMN col_name91",
            "ALTER TABLE tbl_inventory MODIFY COLUMN col_name73 TEXT",
            "ALTER TABLE tbl_product CHANGE COLUMN col_name28 col_name217 DATETIME",
            "ALTER TABLE tbl_inventory ADD INDEX idx_name145 (col_name51)",
            "ALTER TABLE tbl_order DROP INDEX idx_name23",
            "ALTER TABLE tbl_product RENAME TO tbl_product_new",
            "ALTER TABLE tbl_order ADD PRIMARY KEY (col_name49)",
            "ALTER TABLE tbl_order DROP PRIMARY KEY",
            "ALTER TABLE tbl_customer ADD FOREIGN KEY (col_name74) REFERENCES tbl_order(order_id)",
            "ALTER TABLE tbl_inventory DROP FOREIGN KEY fk_name46",
            "ALTER TABLE demo ADD name VARCHAR(128) NULL DEFAULT NULL AFTER age",
            "ALTER TABLE `process_template_config` ADD template_admin_approver json NULL DEFAULT NULL COMMENT '模板管理员';"
        ];

        for i in 0..alter_sqls.len() {
            println!("{}/{}", i + 1, alter_sqls.len());
            let res = AlterTableStatement::parse(alter_sqls[i]);
            // res.unwrap();
            // println!("{:?}", res);
            println!("{:?}", res);
            assert!(res.is_ok());
        }
    }
}
