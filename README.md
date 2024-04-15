# sqlparser-mysql [![Rust](https://github.com/xring/sqlparser-mysql/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/xring/sqlparser-mysql/actions/workflows/rust.yml)

> A SQL parser for MySQL with nom. Written in Rust.

## Project Status
Please note that this repository is currently under active development. As such, it is subject to potentially disruptive changes without prior notice. We are working hard to improve the project and make it more robust. However, during this phase, we might introduce significant modifications to the codebase, features, and functionality.

We encourage users and contributors to keep this in mind when using, forking, or contributing to this project. Your understanding and patience are greatly appreciated as we continue to evolve this project.

Stay tuned for updates, and feel free to reach out or contribute to the development process!

## Disclaimer

This project is in a pre-release state. It may contain incomplete features, bugs, and unexpected behaviors. Use it at your own risk.

## Quick Start

### Example parsing SQL
```rust
use sqlparser_mysql::parser::Parser;
use sqlparser_mysql::parser::ParseConfig;

let config = ParseConfig::default();
let sql = "SELECT a, b, 123, myfunc(b) \
            FROM table_1 \
            WHERE a > b AND b < 100 \
            ORDER BY a DESC, b";
// parse to a Statement
let ast = Parser::parse(&config, sql).unwrap();

println!("AST: {:#?}", ast);
```
The output should be:
```rust
AST: Select(
    SelectStatement {
        tables: [
            Table {
                name: "table_1",
                alias: None,
                schema: None,
            },
        ],
        distinct: false,
        fields: [
            Col(
                Column {
                    name: "a",
                    alias: None,
                    table: None,
                    function: None,
                },
            ),
            Col(
                Column {
                    name: "b",
                    alias: None,
                    table: None,
                    function: None,
                },
            ),
            Value(
                Literal(
                    LiteralExpression {
                        value: Integer(
                            123,
                        ),
                        alias: None,
                    },
                ),
            ),
            Col(
                Column {
                    name: "myfunc(b)",
                    alias: None,
                    table: None,
                    function: Some(
                        Generic(
                            "myfunc",
                            FunctionArguments {
                                arguments: [
                                    Column(
                                        Column {
                                            name: "b",
                                            alias: None,
                                            table: None,
                                            function: None,
                                        },
                                    ),
                                ],
                            },
                        ),
                    ),
                },
            ),
        ],
        join: [],
        where_clause: Some(
            LogicalOp(
                ConditionTree {
                    operator: And,
                    left: ComparisonOp(
                        ConditionTree {
                            operator: Greater,
                            left: Base(
                                Field(
                                    Column {
                                        name: "a",
                                        alias: None,
                                        table: None,
                                        function: None,
                                    },
                                ),
                            ),
                            right: Base(
                                Field(
                                    Column {
                                        name: "b",
                                        alias: None,
                                        table: None,
                                        function: None,
                                    },
                                ),
                            ),
                        },
                    ),
                    right: ComparisonOp(
                        ConditionTree {
                            operator: Less,
                            left: Base(
                                Field(
                                    Column {
                                        name: "b",
                                        alias: None,
                                        table: None,
                                        function: None,
                                    },
                                ),
                            ),
                            right: Base(
                                Literal(
                                    Integer(
                                        100,
                                    ),
                                ),
                            ),
                        },
                    ),
                },
            ),
        ),
        group_by: None,
        order: Some(
            OrderClause {
                columns: [
                    (
                        Column {
                            name: "a",
                            alias: None,
                            table: None,
                            function: None,
                        },
                        Desc,
                    ),
                    (
                        Column {
                            name: "b",
                            alias: None,
                            table: None,
                            function: None,
                        },
                        Asc,
                    ),
                ],
            },
        ),
        limit: None,
    },
)
```

### Creating SQL text from AST
```rust
use sqlparser_mysql::parser::Parser;
use sqlparser_mysql::parser::ParseConfig;

let sql = "SELECT a FROM table_1";
let config = ParseConfig::default();

// parse to a Statement
let ast = Parser::parse(&config, sql).unwrap();

// The original SQL text can be generated from the AST
assert_eq!(ast.to_string(), sql);
```

## Supported Statements

### Data Definition Statements

[MySQL Doc](https://dev.mysql.com/doc/refman/8.0/en/sql-data-definition-statements.html)

- [ ] Atomic Data Definition Statement Support
- [x] ALTER DATABASE Statement
- [ ] ALTER EVENT Statement
- [ ] ALTER FUNCTION Statement
- [ ] ALTER INSTANCE Statement
- [ ] ALTER LOGFILE GROUP Statement
- [ ] ALTER PROCEDURE Statement
- [ ] ALTER SERVER Statement
- [x] ALTER TABLE Statement
- [ ] ALTER TABLESPACE Statement
- [ ] ALTER VIEW Statement
- [ ] CREATE DATABASE Statement
- [ ] CREATE EVENT Statement
- [ ] CREATE FUNCTION Statement
- [x] CREATE INDEX Statement
- [ ] CREATE LOGFILE GROUP Statement
- [ ] CREATE PROCEDURE and CREATE FUNCTION Statements
- [ ] CREATE SERVER Statement
- [ ] CREATE SPATIAL REFERENCE SYSTEM Statement
- [x] CREATE TABLE Statement
- [ ] CREATE TABLESPACE Statement
- [ ] CREATE TRIGGER Statement
- [ ] CREATE VIEW Statement
- [x] DROP DATABASE Statement
- [x] DROP EVENT Statement
- [ ] DROP FUNCTION Statement --> "DROP PROCEDURE and DROP FUNCTION Statements" && "DROP FUNCTION Statement for Loadable Functions"
- [x] DROP INDEX Statement
- [x] DROP LOGFILE GROUP Statement
- [x] DROP PROCEDURE and DROP FUNCTION Statements
- [x] DROP SERVER Statement
- [x] DROP SPATIAL REFERENCE SYSTEM Statement
- [x] DROP TABLE Statement
- [x] DROP TABLESPACE Statement
- [x] DROP TRIGGER Statement
- [x] DROP VIEW Statement
- [x] RENAME TABLE Statement
- [x] TRUNCATE TABLE Statement

### Database Administration Statements
- [x] SET Statements