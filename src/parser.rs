use std::fmt;
use std::io::BufRead;
use std::str;

use das::SetStatement;
use dds::{
    AlterDatabaseStatement, AlterTableStatement, CreateIndexStatement, CreateTableStatement,
    DropDatabaseStatement, DropEventStatement, DropFunctionStatement, DropIndexStatement,
    DropLogfileGroupStatement, DropProcedureStatement, DropServerStatement,
    DropSpatialReferenceSystemStatement, DropTableStatement, DropTablespaceStatement,
    DropTriggerStatement, DropViewStatement, RenameTableStatement, TruncateTableStatement,
};
use dms::{
    CompoundSelectStatement, DeleteStatement, InsertStatement, SelectStatement, UpdateStatement,
};
use nom::branch::alt;
use nom::combinator::map;
use nom::Offset;

pub struct Parser;

impl Parser {
    pub fn parse(config: &ParseConfig, input: &str) -> Result<Statement, String> {
        let input = input.trim();

        let dds_parser = alt((
            map(AlterDatabaseStatement::parse, Statement::AlterDatabase),
            map(AlterTableStatement::parse, Statement::AlterTable),
            map(CreateIndexStatement::parse, Statement::CreateIndex),
            map(CreateTableStatement::parse, Statement::CreateTable),
            map(DropDatabaseStatement::parse, Statement::DropDatabase),
            map(DropEventStatement::parse, Statement::DropEvent),
            map(DropFunctionStatement::parse, Statement::DropFunction),
            map(DropIndexStatement::parse, Statement::DropIndex),
            map(
                DropLogfileGroupStatement::parse,
                Statement::DropLogfileGroup,
            ),
            map(DropProcedureStatement::parse, Statement::DropProcedure),
            map(DropServerStatement::parse, Statement::DropServer),
            map(
                DropSpatialReferenceSystemStatement::parse,
                Statement::DropSpatialReferenceSystem,
            ),
            map(DropTableStatement::parse, Statement::DropTable),
            map(DropTablespaceStatement::parse, Statement::DropTableSpace),
            map(DropTriggerStatement::parse, Statement::DropTrigger),
            map(DropViewStatement::parse, Statement::DropView),
            map(RenameTableStatement::parse, Statement::RenameTable),
            map(TruncateTableStatement::parse, Statement::TruncateTable),
        ));

        let das_parser = alt((map(SetStatement::parse, Statement::Set),));

        let dms_parser = alt((
            map(SelectStatement::parse, Statement::Select),
            map(CompoundSelectStatement::parse, Statement::CompoundSelect),
            map(InsertStatement::parse, Statement::Insert),
            map(DeleteStatement::parse, Statement::Delete),
            map(UpdateStatement::parse, Statement::Update),
        ));

        let mut parser = alt((dds_parser, dms_parser, das_parser));

        match parser(input) {
            Ok(result) => Ok(result.1),
            Err(nom::Err::Error(err)) => {
                if config.log_with_backtrace {
                    println!(">>>>>>>>>>>>>>>>>>>>");
                    for error in &err.errors {
                        println!("{:?} :: {:?}", error.0, error.1)
                    }
                    println!("<<<<<<<<<<<<<<<<<<<<");
                }

                let msg = err.errors[0].0;
                let err_msg = format!("failed to parse sql, error near `{}`", msg);
                Err(err_msg)
            }
            _ => Err(String::from("failed to parse sql: other error")),
        }
    }
}

#[derive(Default)]
pub struct ParseConfig {
    pub log_with_backtrace: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    // DDS
    AlterDatabase(AlterDatabaseStatement),
    AlterTable(AlterTableStatement),
    CreateIndex(CreateIndexStatement),
    CreateTable(CreateTableStatement),
    DropDatabase(DropDatabaseStatement),
    DropEvent(DropEventStatement),
    DropFunction(DropFunctionStatement),
    DropIndex(DropIndexStatement),
    DropLogfileGroup(DropLogfileGroupStatement),
    DropProcedure(DropProcedureStatement),
    DropServer(DropServerStatement),
    DropSpatialReferenceSystem(DropSpatialReferenceSystemStatement),
    DropTable(DropTableStatement),
    DropTableSpace(DropTablespaceStatement),
    DropTrigger(DropTriggerStatement),
    DropView(DropViewStatement),
    RenameTable(RenameTableStatement),
    TruncateTable(TruncateTableStatement),
    // DAS
    Set(SetStatement),
    // HISTORY
    Insert(InsertStatement),
    CompoundSelect(CompoundSelectStatement),
    Select(SelectStatement),
    Delete(DeleteStatement),
    Update(UpdateStatement),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // FIXME add all
            Statement::Select(ref select) => write!(f, "{}", select),
            Statement::Insert(ref insert) => write!(f, "{}", insert),
            Statement::CreateTable(ref create) => write!(f, "{}", create),
            Statement::Delete(ref delete) => write!(f, "{}", delete),
            Statement::DropTable(ref drop) => write!(f, "{}", drop),
            Statement::DropDatabase(ref drop) => write!(f, "{}", drop),
            Statement::TruncateTable(ref drop) => write!(f, "{}", drop),
            Statement::Update(ref update) => write!(f, "{}", update),
            Statement::Set(ref set) => write!(f, "{}", set),
            _ => unimplemented!(),
        }
    }
}
