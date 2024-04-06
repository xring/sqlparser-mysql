//! # SQL Parser for MySQL with Rust
//!
//! This crate provides parser that can parse SQL into an Abstract Syntax Tree.
//!
//! # Example parsing SQL
//!
//! ```
//! use sqlparser_mysql::parser::Parser;
//! use sqlparser_mysql::parser::ParseConfig;
//!
//! let config = ParseConfig::default();
//! let sql = "SELECT a, b, 123, myfunc(b) \
//!            FROM table_1 \
//!            WHERE a > b AND b < 100 \
//!            ORDER BY a DESC, b";
//!
//! // parse to a Statement
//! let ast = Parser::parse(&config, sql).unwrap();
//!
//! println!("AST: {:?}", ast);
//! ```
//!
//! # Creating SQL text from AST
//!
//! This crate allows users to recover the original SQL text (with comments
//! removed, normalized whitespace and identifier capitalization), which is
//! useful for tools that analyze and manipulate SQL.
//!
//! ```
//! use sqlparser_mysql::parser::Parser;
//! use sqlparser_mysql::parser::ParseConfig;
//!
//! let sql = "SELECT a FROM table_1";
//! let config = ParseConfig::default();
//!
//! // parse to a Statement
//! let ast = Parser::parse(&config, sql).unwrap();
//!
//! // The original SQL text can be generated from the AST
//! assert_eq!(ast.to_string(), sql);
//! ```
//!
//! [sqlparser-mysql crates.io page]: https://crates.io/crates/sqlparser-mysql

#![allow(unused)]
extern crate core;
extern crate nom;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub use self::parser::*;

pub mod base;
pub mod das;
pub mod dds;
pub mod dms;
pub mod parser;
