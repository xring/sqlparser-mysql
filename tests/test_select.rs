extern crate sqlparser_mysql;

use sqlparser_mysql::base::arithmetic::{ArithmeticBase, ArithmeticExpression, ArithmeticOperator};
use sqlparser_mysql::base::column::{FunctionArgument, FunctionArguments, FunctionExpression};
use sqlparser_mysql::base::condition::ConditionBase::LiteralList;
use sqlparser_mysql::base::condition::ConditionExpression::{Base, ComparisonOp, LogicalOp};
use sqlparser_mysql::base::condition::{ConditionBase, ConditionExpression, ConditionTree};
use sqlparser_mysql::base::{
    CaseWhenExpression, Column, ColumnOrLiteral, FieldDefinitionExpression, FieldValueExpression,
    ItemPlaceholder, JoinClause, JoinConstraint, JoinOperator, JoinRightSide, Literal, Operator,
    OrderClause, OrderType, Table,
};
use sqlparser_mysql::dms::{
    BetweenAndClause, CompoundSelectOperator, CompoundSelectStatement, GroupByClause, LimitClause,
    SelectStatement,
};
use sqlparser_mysql::{ParseConfig, Parser};

#[test]
fn display_select_query() {
    let str0 = "SELECT * FROM users";
    let str1 = "SELECT * FROM users AS u";
    let str2 = "SELECT name, password FROM users AS u";
    let str3 = "SELECT name, password FROM users AS u WHERE user_id = '1'";
    let str4 = "SELECT name, password FROM users AS u WHERE user = 'aaa' AND password = 'xxx'";
    let str5 = "SELECT name * 2 AS double_name FROM users";
    let config = ParseConfig::default();

    let res0 = Parser::parse(&config, str0);
    let res1 = Parser::parse(&config, str1);
    let res2 = Parser::parse(&config, str2);
    let res3 = Parser::parse(&config, str3);
    let res4 = Parser::parse(&config, str4);
    let res5 = Parser::parse(&config, str5);

    assert!(res0.is_ok());
    assert!(res1.is_ok());
    assert!(res2.is_ok());
    assert!(res3.is_ok());
    assert!(res4.is_ok());
    assert!(res5.is_ok());

    assert_eq!(str0, format!("{}", res0.unwrap()));
    assert_eq!(str1, format!("{}", res1.unwrap()));
    assert_eq!(str2, format!("{}", res2.unwrap()));
    assert_eq!(str3, format!("{}", res3.unwrap()));
    assert_eq!(str4, format!("{}", res4.unwrap()));
    assert_eq!(str5, format!("{}", res5.unwrap()));
}

/////////////// COMPOUND SELECT
#[test]
fn simple() {
    let sql = "SELECT * FROM my_table WHERE age < 30;";
    let res = CompoundSelectStatement::parse(sql);
    println!("{:?}", res);
}

#[test]
fn union() {
    let qstr = "SELECT id, 1 FROM Vote UNION SELECT id, stars from Rating;";
    let qstr2 = "(SELECT id, 1 FROM Vote) UNION (SELECT id, stars from Rating);";
    let res = CompoundSelectStatement::parse(qstr);
    let res2 = CompoundSelectStatement::parse(qstr2);

    let first_select = SelectStatement {
        tables: vec![Table::from("Vote")],
        fields: vec![
            FieldDefinitionExpression::Col(Column::from("id")),
            FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                Literal::Integer(1).into(),
            )),
        ],
        ..Default::default()
    };
    let second_select = SelectStatement {
        tables: vec![Table::from("Rating")],
        fields: vec![
            FieldDefinitionExpression::Col(Column::from("id")),
            FieldDefinitionExpression::Col(Column::from("stars")),
        ],
        ..Default::default()
    };
    let expected = CompoundSelectStatement {
        selects: vec![
            (None, first_select),
            (Some(CompoundSelectOperator::DistinctUnion), second_select),
        ],
        order: None,
        limit: None,
    };

    assert_eq!(res.unwrap().1, expected);
    assert_eq!(res2.unwrap().1, expected);
}

#[test]
fn union_strict() {
    let qstr = "SELECT id, 1 FROM Vote);";
    let qstr2 = "(SELECT id, 1 FROM Vote;";
    let qstr3 = "SELECT id, 1 FROM Vote) UNION (SELECT id, stars from Rating;";
    let res = CompoundSelectStatement::parse(qstr);
    let res2 = CompoundSelectStatement::parse(qstr2);
    let res3 = CompoundSelectStatement::parse(qstr3);

    assert!(&res.is_err());
    // assert_eq!(
    //     res.unwrap_err(),
    //     nom::Err::Error(nom::error::Error::new(");", nom::error::ErrorKind::Tag))
    // );
    assert!(&res2.is_err());
    // assert_eq!(
    //     res2.unwrap_err(),
    //     nom::Err::Error(nom::error::Error::new(";", nom::error::ErrorKind::Tag))
    // );
    assert!(&res3.is_err());
    // assert_eq!(
    //     res3.unwrap_err(),
    //     nom::Err::Error(nom::error::Error::new(
    //         ") UNION (SELECT id, stars from Rating;",
    //         nom::error::ErrorKind::Tag,
    //     ))
    // );
}

#[test]
fn multi_union() {
    let qstr = "SELECT id, 1 FROM Vote \
                    UNION SELECT id, stars from Rating \
                    UNION DISTINCT SELECT 42, 5 FROM Vote;";
    let res = CompoundSelectStatement::parse(qstr);

    let first_select = SelectStatement {
        tables: vec![Table::from("Vote")],
        fields: vec![
            FieldDefinitionExpression::Col(Column::from("id")),
            FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                Literal::Integer(1).into(),
            )),
        ],
        ..Default::default()
    };
    let second_select = SelectStatement {
        tables: vec![Table::from("Rating")],
        fields: vec![
            FieldDefinitionExpression::Col(Column::from("id")),
            FieldDefinitionExpression::Col(Column::from("stars")),
        ],
        ..Default::default()
    };
    let third_select = SelectStatement {
        tables: vec![Table::from("Vote")],
        fields: vec![
            FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                Literal::Integer(42).into(),
            )),
            FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                Literal::Integer(5).into(),
            )),
        ],
        ..Default::default()
    };

    let expected = CompoundSelectStatement {
        selects: vec![
            (None, first_select),
            (Some(CompoundSelectOperator::DistinctUnion), second_select),
            (Some(CompoundSelectOperator::DistinctUnion), third_select),
        ],
        order: None,
        limit: None,
    };

    assert_eq!(res.unwrap().1, expected);
}

#[test]
fn union_all() {
    let qstr = "SELECT id, 1 FROM Vote UNION ALL SELECT id, stars from Rating;";
    let res = CompoundSelectStatement::parse(qstr);

    let first_select = SelectStatement {
        tables: vec![Table::from("Vote")],
        fields: vec![
            FieldDefinitionExpression::Col(Column::from("id")),
            FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                Literal::Integer(1).into(),
            )),
        ],
        ..Default::default()
    };
    let second_select = SelectStatement {
        tables: vec![Table::from("Rating")],
        fields: vec![
            FieldDefinitionExpression::Col(Column::from("id")),
            FieldDefinitionExpression::Col(Column::from("stars")),
        ],
        ..Default::default()
    };
    let expected = CompoundSelectStatement {
        selects: vec![
            (None, first_select),
            (Some(CompoundSelectOperator::Union), second_select),
        ],
        order: None,
        limit: None,
    };

    assert_eq!(res.unwrap().1, expected);
}

/////////////// SELECT
#[test]
fn between_and() {
    let str = "age between 10 and 20";
    let res = BetweenAndClause::parse(str);
    println!("{:?}", res);
}

#[test]
fn simple_select() {
    let str = "SELECT id, name FROM users;";

    let res = SelectStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("users")],
            fields: FieldDefinitionExpression::from_column_str(&["id", "name"]),
            ..Default::default()
        }
    );
}

#[test]
fn more_involved_select() {
    let str = "SELECT users.id, users.name FROM users;";

    let res = SelectStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("users")],
            fields: FieldDefinitionExpression::from_column_str(&["users.id", "users.name"]),
            ..Default::default()
        }
    );
}

#[test]
fn select_literals() {
    use sqlparser_mysql::base::Literal;

    let str = "SELECT NULL, 1, \"foo\", CURRENT_TIME FROM users;";
    // TODO: doesn't support selecting literals without a FROM clause, which is still valid SQL
    //        let str = "SELECT NULL, 1, \"foo\";";

    let res = SelectStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("users")],
            fields: vec![
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                    Literal::Null.into(),
                )),
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                    Literal::Integer(1).into(),
                )),
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                    Literal::String("foo".to_owned()).into(),
                )),
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(
                    Literal::CurrentTime.into(),
                )),
            ],
            ..Default::default()
        }
    );
}

#[test]
fn select_all() {
    let str = "SELECT * FROM users;";

    let res = SelectStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("users")],
            fields: vec![FieldDefinitionExpression::All],
            ..Default::default()
        }
    );
}

#[test]
fn select_all_in_table() {
    let str = "SELECT users.* FROM users, votes;";

    let res = SelectStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("users"), Table::from("votes")],
            fields: vec![FieldDefinitionExpression::AllInTable(String::from("users"))],
            ..Default::default()
        }
    );
}

#[test]
fn spaces_optional() {
    let str = "SELECT id,name FROM users;";

    let res = SelectStatement::parse(str);
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("users")],
            fields: FieldDefinitionExpression::from_column_str(&["id", "name"]),
            ..Default::default()
        }
    );
}

#[test]
fn case_sensitivity() {
    let str_lc = "select id, name from users;";
    let str_uc = "SELECT id, name FROM users;";

    assert_eq!(
        SelectStatement::parse(str_lc).unwrap(),
        SelectStatement::parse(str_uc).unwrap()
    );
}

#[test]
fn termination() {
    let str_sem = "select id, name from users;";
    let str_nosem = "select id, name from users";
    let str_linebreak = "select id, name from users\n";

    let r1 = SelectStatement::parse(str_sem).unwrap();
    let r2 = SelectStatement::parse(str_nosem).unwrap();
    let r3 = SelectStatement::parse(str_linebreak).unwrap();
    assert_eq!(r1, r2);
    assert_eq!(r2, r3);
}

#[test]
fn where_clause() {
    where_clause_with_variable_placeholder(
        "select * from ContactInfo where email=?;",
        Literal::Placeholder(ItemPlaceholder::QuestionMark),
    );
}

#[test]
fn where_clause_with_dollar_variable() {
    where_clause_with_variable_placeholder(
        "select * from ContactInfo where email= $3;",
        Literal::Placeholder(ItemPlaceholder::DollarNumber(3)),
    );
}

#[test]
fn where_clause_with_colon_variable() {
    where_clause_with_variable_placeholder(
        "select * from ContactInfo where email= :5;",
        Literal::Placeholder(ItemPlaceholder::ColonNumber(5)),
    );
}

fn where_clause_with_variable_placeholder(str: &str, literal: Literal) {
    let res = SelectStatement::parse(str);

    let expected_left = Base(ConditionBase::Field(Column::from("email")));
    let expected_where_cond = Some(ComparisonOp(ConditionTree {
        left: Box::new(expected_left),
        right: Box::new(Base(ConditionBase::Literal(literal))),
        operator: Operator::Equal,
    }));
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("ContactInfo")],
            fields: vec![FieldDefinitionExpression::All],
            where_clause: expected_where_cond,
            ..Default::default()
        }
    );
}

#[test]
fn limit_clause() {
    let str1 = "select * from users limit 10\n";
    let str2 = "select * from users limit 10 offset 10\n";

    let expected_lim1 = LimitClause {
        limit: 10,
        offset: 0,
    };
    let expected_lim2 = LimitClause {
        limit: 10,
        offset: 10,
    };

    let res1 = SelectStatement::parse(str1);
    let res2 = SelectStatement::parse(str2);
    assert_eq!(res1.unwrap().1.limit, Some(expected_lim1));
    assert_eq!(res2.unwrap().1.limit, Some(expected_lim2));
}

#[test]
fn table_alias() {
    let str1 = "select * from PaperTag as t;";
    // let str2 = "select * from PaperTag t;";

    let res1 = SelectStatement::parse(str1);
    assert_eq!(
        res1.unwrap().1,
        SelectStatement {
            tables: vec![Table {
                name: String::from("PaperTag"),
                alias: Some(String::from("t")),
                schema: None,
            },],
            fields: vec![FieldDefinitionExpression::All],
            ..Default::default()
        }
    );
    // let res2 = SelectStatement::parse(str2);
    // assert_eq!(res1.unwrap().1, res2.unwrap().1);
}

#[test]
fn table_schema() {
    let str1 = "select * from db1.PaperTag as t;";

    let res1 = SelectStatement::parse(str1);
    assert_eq!(
        res1.unwrap().1,
        SelectStatement {
            tables: vec![Table {
                name: String::from("PaperTag"),
                alias: Some(String::from("t")),
                schema: Some(String::from("db1")),
            },],
            fields: vec![FieldDefinitionExpression::All],
            ..Default::default()
        }
    );
    // let res2 = SelectStatement::parse(str2);
    // assert_eq!(res1.unwrap().1, res2.unwrap().1);
}

#[test]
fn column_alias() {
    let str1 = "select name as TagName from PaperTag;";
    let str2 = "select PaperTag.name as TagName from PaperTag;";

    let res1 = SelectStatement::parse(str1);
    assert_eq!(
        res1.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("PaperTag")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("name"),
                alias: Some(String::from("TagName")),
                table: None,
                function: None,
            }),],
            ..Default::default()
        }
    );
    let res2 = SelectStatement::parse(str2);
    assert_eq!(
        res2.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("PaperTag")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("name"),
                alias: Some(String::from("TagName")),
                table: Some(String::from("PaperTag")),
                function: None,
            }),],
            ..Default::default()
        }
    );
}

#[test]
fn column_alias_no_as() {
    let str1 = "select name TagName from PaperTag;";
    let str2 = "select PaperTag.name TagName from PaperTag;";

    let res1 = SelectStatement::parse(str1);
    assert_eq!(
        res1.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("PaperTag")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("name"),
                alias: Some(String::from("TagName")),
                table: None,
                function: None,
            }),],
            ..Default::default()
        }
    );
    let res2 = SelectStatement::parse(str2);
    assert_eq!(
        res2.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("PaperTag")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("name"),
                alias: Some(String::from("TagName")),
                table: Some(String::from("PaperTag")),
                function: None,
            }),],
            ..Default::default()
        }
    );
}

#[test]
fn distinct() {
    let str = "select distinct tag from PaperTag where paperId=?;";

    let res = SelectStatement::parse(str);
    let expected_left = Base(ConditionBase::Field(Column::from("paperId")));
    let expected_where_cond = Some(ComparisonOp(ConditionTree {
        left: Box::new(expected_left),
        right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
            ItemPlaceholder::QuestionMark,
        )))),
        operator: Operator::Equal,
    }));
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("PaperTag")],
            distinct: true,
            fields: FieldDefinitionExpression::from_column_str(&["tag"]),
            where_clause: expected_where_cond,
            ..Default::default()
        }
    );
}

#[test]
fn simple_condition_expr() {
    let str = "select infoJson from PaperStorage where paperId=? and paperStorageId=?;";

    let res = SelectStatement::parse(str);

    let left_ct = ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("paperId")))),
        right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
            ItemPlaceholder::QuestionMark,
        )))),
        operator: Operator::Equal,
    };
    let left_comp = Box::new(ComparisonOp(left_ct));
    let right_ct = ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("paperStorageId")))),
        right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
            ItemPlaceholder::QuestionMark,
        )))),
        operator: Operator::Equal,
    };
    let right_comp = Box::new(ComparisonOp(right_ct));
    let expected_where_cond = Some(LogicalOp(ConditionTree {
        left: left_comp,
        right: right_comp,
        operator: Operator::And,
    }));
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("PaperStorage")],
            fields: FieldDefinitionExpression::from_column_str(&["infoJson"]),
            where_clause: expected_where_cond,
            ..Default::default()
        }
    );
}

#[test]
fn where_and_limit_clauses() {
    let str = "select * from users where id = ? limit 10\n";
    let res = SelectStatement::parse(str);

    let expected_lim = Some(LimitClause {
        limit: 10,
        offset: 0,
    });
    let ct = ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("id")))),
        right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
            ItemPlaceholder::QuestionMark,
        )))),
        operator: Operator::Equal,
    };
    let expected_where_cond = Some(ComparisonOp(ct));

    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("users")],
            fields: vec![FieldDefinitionExpression::All],
            where_clause: expected_where_cond,
            limit: expected_lim,
            ..Default::default()
        }
    );
}

#[test]
fn aggregation_column() {
    let str = "SELECT max(addr_id) FROM address;";

    let res = SelectStatement::parse(str);
    let agg_expr = FunctionExpression::Max(FunctionArgument::Column(Column::from("addr_id")));
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("address")],
            fields: vec![FieldDefinitionExpression::Col(Column {
                name: String::from("max(addr_id)"),
                alias: None,
                table: None,
                function: Some(Box::new(agg_expr)),
            }),],
            ..Default::default()
        }
    );
}

#[test]
fn aggregation_column_with_alias() {
    let str = "SELECT max(addr_id) AS max_addr FROM address;";

    let res = SelectStatement::parse(str);
    let agg_expr = FunctionExpression::Max(FunctionArgument::Column(Column::from("addr_id")));
    let expected_stmt = SelectStatement {
        tables: vec![Table::from("address")],
        fields: vec![FieldDefinitionExpression::Col(Column {
            name: String::from("max_addr"),
            alias: Some(String::from("max_addr")),
            table: None,
            function: Some(Box::new(agg_expr)),
        })],
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected_stmt);
}

#[test]
fn count_all() {
    let str = "SELECT COUNT(*) FROM votes GROUP BY aid;";

    let res = SelectStatement::parse(str);
    let agg_expr = FunctionExpression::CountStar;
    let expected_stmt = SelectStatement {
        tables: vec![Table::from("votes")],
        fields: vec![FieldDefinitionExpression::Col(Column {
            name: String::from("count(*)"),
            alias: None,
            table: None,
            function: Some(Box::new(agg_expr)),
        })],
        group_by: Some(GroupByClause {
            columns: vec![Column::from("aid")],
            having: None,
        }),
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected_stmt);
}

#[test]
fn count_distinct() {
    let str = "SELECT COUNT(DISTINCT vote_id) FROM votes GROUP BY aid;";

    let res = SelectStatement::parse(str);
    let agg_expr =
        FunctionExpression::Count(FunctionArgument::Column(Column::from("vote_id")), true);
    let expected_stmt = SelectStatement {
        tables: vec![Table::from("votes")],
        fields: vec![FieldDefinitionExpression::Col(Column {
            name: String::from("count(distinct vote_id)"),
            alias: None,
            table: None,
            function: Some(Box::new(agg_expr)),
        })],
        group_by: Some(GroupByClause {
            columns: vec![Column::from("aid")],
            having: None,
        }),
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected_stmt);
}

#[test]
fn count_filter() {
    let str = "SELECT COUNT(CASE WHEN vote_id > 10 THEN vote_id END) FROM votes GROUP BY aid;";
    let res = SelectStatement::parse(str);

    let filter_cond = ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("vote_id")))),
        right: Box::new(Base(ConditionBase::Literal(Literal::Integer(10.into())))),
        operator: Operator::Greater,
    });
    let agg_expr = FunctionExpression::Count(
        FunctionArgument::Conditional(CaseWhenExpression {
            then_expr: ColumnOrLiteral::Column(Column::from("vote_id")),
            else_expr: None,
            condition: filter_cond,
        }),
        false,
    );
    let expected_stmt = SelectStatement {
        tables: vec![Table::from("votes")],
        fields: vec![FieldDefinitionExpression::Col(Column {
            name: format!("{}", agg_expr),
            alias: None,
            table: None,
            function: Some(Box::new(agg_expr)),
        })],
        group_by: Some(GroupByClause {
            columns: vec![Column::from("aid")],
            having: None,
        }),
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected_stmt);
}

#[test]
fn sum_filter() {
    let str = "SELECT SUM(CASE WHEN sign = 1 THEN vote_id END) FROM votes GROUP BY aid;";

    let res = SelectStatement::parse(str);

    let filter_cond = ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("sign")))),
        right: Box::new(Base(ConditionBase::Literal(Literal::Integer(1.into())))),
        operator: Operator::Equal,
    });
    let agg_expr = FunctionExpression::Sum(
        FunctionArgument::Conditional(CaseWhenExpression {
            then_expr: ColumnOrLiteral::Column(Column::from("vote_id")),
            else_expr: None,
            condition: filter_cond,
        }),
        false,
    );
    let expected_stmt = SelectStatement {
        tables: vec![Table::from("votes")],
        fields: vec![FieldDefinitionExpression::Col(Column {
            name: format!("{}", agg_expr),
            alias: None,
            table: None,
            function: Some(Box::new(agg_expr)),
        })],
        group_by: Some(GroupByClause {
            columns: vec![Column::from("aid")],
            having: None,
        }),
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected_stmt);
}

#[test]
fn sum_filter_else_literal() {
    let str = "SELECT SUM(CASE WHEN sign = 1 THEN vote_id ELSE 6 END) FROM votes GROUP BY aid;";

    let res = SelectStatement::parse(str);

    let filter_cond = ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("sign")))),
        right: Box::new(Base(ConditionBase::Literal(Literal::Integer(1.into())))),
        operator: Operator::Equal,
    });
    let agg_expr = FunctionExpression::Sum(
        FunctionArgument::Conditional(CaseWhenExpression {
            then_expr: ColumnOrLiteral::Column(Column::from("vote_id")),
            else_expr: Some(ColumnOrLiteral::Literal(Literal::Integer(6))),
            condition: filter_cond,
        }),
        false,
    );
    let expected_stmt = SelectStatement {
        tables: vec![Table::from("votes")],
        fields: vec![FieldDefinitionExpression::Col(Column {
            name: format!("{}", agg_expr),
            alias: None,
            table: None,
            function: Some(Box::new(agg_expr)),
        })],
        group_by: Some(GroupByClause {
            columns: vec![Column::from("aid")],
            having: None,
        }),
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected_stmt);
}

#[test]
fn count_filter_lobsters() {
    let str = "SELECT
            COUNT(CASE WHEN votes.story_id IS NULL AND votes.vote = 0 THEN votes.vote END) as votes
            FROM votes
            GROUP BY votes.comment_id;";

    let res = SelectStatement::parse(str);

    let filter_cond = LogicalOp(ConditionTree {
        left: Box::new(ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("votes.story_id")))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Null))),
            operator: Operator::Equal,
        })),
        right: Box::new(ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("votes.vote")))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Integer(0)))),
            operator: Operator::Equal,
        })),
        operator: Operator::And,
    });
    let agg_expr = FunctionExpression::Count(
        FunctionArgument::Conditional(CaseWhenExpression {
            then_expr: ColumnOrLiteral::Column(Column::from("votes.vote")),
            else_expr: None,
            condition: filter_cond,
        }),
        false,
    );
    let expected_stmt = SelectStatement {
        tables: vec![Table::from("votes")],
        fields: vec![FieldDefinitionExpression::Col(Column {
            name: String::from("votes"),
            alias: Some(String::from("votes")),
            table: None,
            function: Some(Box::new(agg_expr)),
        })],
        group_by: Some(GroupByClause {
            columns: vec![Column::from("votes.comment_id")],
            having: None,
        }),
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected_stmt);
}

#[test]
fn generic_function_query() {
    let str = "SELECT coalesce(a, b,c) as x,d FROM sometable;";

    let res = SelectStatement::parse(str);
    let agg_expr = FunctionExpression::Generic(
        String::from("coalesce"),
        FunctionArguments {
            arguments: vec![
                FunctionArgument::Column(Column {
                    name: String::from("a"),
                    alias: None,
                    table: None,
                    function: None,
                }),
                FunctionArgument::Column(Column {
                    name: String::from("b"),
                    alias: None,
                    table: None,
                    function: None,
                }),
                FunctionArgument::Column(Column {
                    name: String::from("c"),
                    alias: None,
                    table: None,
                    function: None,
                }),
            ],
        },
    );
    let expected_stmt = SelectStatement {
        tables: vec![Table::from("sometable")],
        fields: vec![
            FieldDefinitionExpression::Col(Column {
                name: String::from("x"),
                alias: Some(String::from("x")),
                table: None,
                function: Some(Box::new(agg_expr)),
            }),
            FieldDefinitionExpression::Col(Column {
                name: String::from("d"),
                alias: None,
                table: None,
                function: None,
            }),
        ],
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected_stmt);
}

#[test]
fn moderately_complex_selection() {
    let str = "SELECT * FROM item, author WHERE item.i_a_id = author.a_id AND \
                       item.i_subject = ? ORDER BY item.i_title limit 50;";

    let res = SelectStatement::parse(str);
    let expected_where_cond = Some(LogicalOp(ConditionTree {
        left: Box::new(ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("item.i_a_id")))),
            right: Box::new(Base(ConditionBase::Field(Column::from("author.a_id")))),
            operator: Operator::Equal,
        })),
        right: Box::new(ComparisonOp(ConditionTree {
            left: Box::new(Base(ConditionBase::Field(Column::from("item.i_subject")))),
            right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
                ItemPlaceholder::QuestionMark,
            )))),
            operator: Operator::Equal,
        })),
        operator: Operator::And,
    }));
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("item"), Table::from("author")],
            fields: vec![FieldDefinitionExpression::All],
            where_clause: expected_where_cond,
            order: Some(OrderClause {
                columns: vec![("item.i_title".into(), OrderType::Asc)],
            }),
            limit: Some(LimitClause {
                limit: 50,
                offset: 0,
            }),
            ..Default::default()
        }
    );
}

#[test]
fn simple_joins() {
    let str = "select paperId from PaperConflict join PCMember using (contactId);";

    let res = SelectStatement::parse(str);
    let expected_stmt = SelectStatement {
        tables: vec![Table::from("PaperConflict")],
        fields: FieldDefinitionExpression::from_column_str(&["paperId"]),
        join: vec![JoinClause {
            operator: JoinOperator::Join,
            right: JoinRightSide::Table(Table::from("PCMember")),
            constraint: JoinConstraint::Using(vec![Column::from("contactId")]),
        }],
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected_stmt);

    // slightly simplified from
    // "select PCMember.contactId, group_concat(reviewType separator '')
    // from PCMember left join PaperReview on (PCMember.contactId=PaperReview.contactId)
    // group by PCMember.contactId"
    let str = "select PCMember.contactId \
                       from PCMember \
                       join PaperReview on (PCMember.contactId=PaperReview.contactId) \
                       order by contactId;";

    let res = SelectStatement::parse(str);
    let ct = ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from(
            "PCMember.contactId",
        )))),
        right: Box::new(Base(ConditionBase::Field(Column::from(
            "PaperReview.contactId",
        )))),
        operator: Operator::Equal,
    };
    let join_cond = ConditionExpression::ComparisonOp(ct);
    let expected = SelectStatement {
        tables: vec![Table::from("PCMember")],
        fields: FieldDefinitionExpression::from_column_str(&["PCMember.contactId"]),
        join: vec![JoinClause {
            operator: JoinOperator::Join,
            right: JoinRightSide::Table(Table::from("PaperReview")),
            constraint: JoinConstraint::On(join_cond),
        }],
        order: Some(OrderClause {
            columns: vec![("contactId".into(), OrderType::Asc)],
        }),
        ..Default::default()
    };
    assert_eq!(res.unwrap().1, expected);

    // Same as above, but no brackets
    let str = "select PCMember.contactId \
                       from PCMember \
                       join PaperReview on PCMember.contactId=PaperReview.contactId \
                       order by contactId;";
    let res = SelectStatement::parse(str);
    assert_eq!(res.unwrap().1, expected);
}

#[test]
fn multi_join() {
    // simplified from
    // "select max(conflictType), PaperReview.contactId as reviewer, PCMember.contactId as
    //  pcMember, ChairAssistant.contactId as assistant, Chair.contactId as chair,
    //  max(PaperReview.reviewNeedsSubmit) as reviewNeedsSubmit from ContactInfo
    //  left join PaperReview using (contactId) left join PaperConflict using (contactId)
    //  left join PCMember using (contactId) left join ChairAssistant using (contactId)
    //  left join Chair using (contactId) where ContactInfo.contactId=?
    //  group by ContactInfo.contactId;";
    let str = "select PCMember.contactId, ChairAssistant.contactId, \
                       Chair.contactId from ContactInfo left join PaperReview using (contactId) \
                       left join PaperConflict using (contactId) left join PCMember using \
                       (contactId) left join ChairAssistant using (contactId) left join Chair \
                       using (contactId) where ContactInfo.contactId=?;";

    let res = SelectStatement::parse(str);
    let ct = ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from(
            "ContactInfo.contactId",
        )))),
        right: Box::new(Base(ConditionBase::Literal(Literal::Placeholder(
            ItemPlaceholder::QuestionMark,
        )))),
        operator: Operator::Equal,
    };
    let expected_where_cond = Some(ComparisonOp(ct));
    let mkjoin = |tbl: &str, col: &str| -> JoinClause {
        JoinClause {
            operator: JoinOperator::LeftJoin,
            right: JoinRightSide::Table(Table::from(tbl)),
            constraint: JoinConstraint::Using(vec![Column::from(col)]),
        }
    };
    assert_eq!(
        res.unwrap().1,
        SelectStatement {
            tables: vec![Table::from("ContactInfo")],
            fields: FieldDefinitionExpression::from_column_str(&[
                "PCMember.contactId",
                "ChairAssistant.contactId",
                "Chair.contactId"
            ]),
            join: vec![
                mkjoin("PaperReview", "contactId"),
                mkjoin("PaperConflict", "contactId"),
                mkjoin("PCMember", "contactId"),
                mkjoin("ChairAssistant", "contactId"),
                mkjoin("Chair", "contactId"),
            ],
            where_clause: expected_where_cond,
            ..Default::default()
        }
    );
}

#[test]
fn nested_select() {
    let qstr = "SELECT ol_i_id FROM orders, order_line \
                    WHERE orders.o_c_id IN (SELECT o_c_id FROM orders, order_line \
                    WHERE orders.o_id = order_line.ol_o_id);";

    let res = SelectStatement::parse(qstr);
    let inner_where_clause = ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_id")))),
        right: Box::new(Base(ConditionBase::Field(Column::from(
            "order_line.ol_o_id",
        )))),
        operator: Operator::Equal,
    });

    let inner_select = SelectStatement {
        tables: vec![Table::from("orders"), Table::from("order_line")],
        fields: FieldDefinitionExpression::from_column_str(&["o_c_id"]),
        where_clause: Some(inner_where_clause),
        ..Default::default()
    };

    let outer_where_clause = ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_c_id")))),
        right: Box::new(Base(ConditionBase::NestedSelect(Box::new(inner_select)))),
        operator: Operator::In,
    });

    let outer_select = SelectStatement {
        tables: vec![Table::from("orders"), Table::from("order_line")],
        fields: FieldDefinitionExpression::from_column_str(&["ol_i_id"]),
        where_clause: Some(outer_where_clause),
        ..Default::default()
    };

    assert_eq!(res.unwrap().1, outer_select);
}

#[test]
fn recursive_nested_select() {
    let qstr = "SELECT ol_i_id FROM orders, order_line WHERE orders.o_c_id \
                    IN (SELECT o_c_id FROM orders, order_line \
                    WHERE orders.o_id = order_line.ol_o_id \
                    AND orders.o_id > (SELECT MAX(o_id) FROM orders));";

    let res = SelectStatement::parse(qstr);

    let agg_expr = FunctionExpression::Max(FunctionArgument::Column(Column::from("o_id")));
    let recursive_select = SelectStatement {
        tables: vec![Table::from("orders")],
        fields: vec![FieldDefinitionExpression::Col(Column {
            name: String::from("max(o_id)"),
            alias: None,
            table: None,
            function: Some(Box::new(agg_expr)),
        })],
        ..Default::default()
    };

    let cop1 = ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_id")))),
        right: Box::new(Base(ConditionBase::Field(Column::from(
            "order_line.ol_o_id",
        )))),
        operator: Operator::Equal,
    });

    let cop2 = ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_id")))),
        right: Box::new(Base(ConditionBase::NestedSelect(Box::new(
            recursive_select,
        )))),
        operator: Operator::Greater,
    });

    let inner_where_clause = LogicalOp(ConditionTree {
        left: Box::new(cop1),
        right: Box::new(cop2),
        operator: Operator::And,
    });

    let inner_select = SelectStatement {
        tables: vec![Table::from("orders"), Table::from("order_line")],
        fields: FieldDefinitionExpression::from_column_str(&["o_c_id"]),
        where_clause: Some(inner_where_clause),
        ..Default::default()
    };

    let outer_where_clause = ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_c_id")))),
        right: Box::new(Base(ConditionBase::NestedSelect(Box::new(inner_select)))),
        operator: Operator::In,
    });

    let outer_select = SelectStatement {
        tables: vec![Table::from("orders"), Table::from("order_line")],
        fields: FieldDefinitionExpression::from_column_str(&["ol_i_id"]),
        where_clause: Some(outer_where_clause),
        ..Default::default()
    };

    assert_eq!(res.unwrap().1, outer_select);
}

#[test]
fn join_against_nested_select() {
    let t0 = "(SELECT ol_i_id FROM order_line)";
    let t1 = "(SELECT ol_i_id FROM order_line) AS ids";

    assert!(JoinRightSide::parse(t0).is_ok());
    assert!(JoinRightSide::parse(t1).is_ok());

    let t0 = "JOIN (SELECT ol_i_id FROM order_line) ON (orders.o_id = ol_i_id)";
    let t1 = "JOIN (SELECT ol_i_id FROM order_line) AS ids ON (orders.o_id = ids.ol_i_id)";

    assert!(JoinClause::parse(t0).is_ok());
    assert!(JoinClause::parse(t1).is_ok());

    let qstr_with_alias = "SELECT o_id, ol_i_id FROM orders JOIN \
                               (SELECT ol_i_id FROM order_line) AS ids \
                               ON (orders.o_id = ids.ol_i_id);";
    let res = SelectStatement::parse(qstr_with_alias);

    // N.B.: Don't alias the inner select to `inner`, which is, well, a SQL keyword!
    let inner_select = SelectStatement {
        tables: vec![Table::from("order_line")],
        fields: FieldDefinitionExpression::from_column_str(&["ol_i_id"]),
        ..Default::default()
    };

    let outer_select = SelectStatement {
        tables: vec![Table::from("orders")],
        fields: FieldDefinitionExpression::from_column_str(&["o_id", "ol_i_id"]),
        join: vec![JoinClause {
            operator: JoinOperator::Join,
            right: JoinRightSide::NestedSelect(Box::new(inner_select), Some("ids".into())),
            constraint: JoinConstraint::On(ComparisonOp(ConditionTree {
                operator: Operator::Equal,
                left: Box::new(Base(ConditionBase::Field(Column::from("orders.o_id")))),
                right: Box::new(Base(ConditionBase::Field(Column::from("ids.ol_i_id")))),
            })),
        }],
        ..Default::default()
    };

    assert_eq!(res.unwrap().1, outer_select);
}

#[test]
fn project_arithmetic_expressions() {
    let qstr = "SELECT MAX(o_id)-3333 FROM orders;";
    let res = SelectStatement::parse(qstr);

    let expected = SelectStatement {
        tables: vec![Table::from("orders")],
        fields: vec![FieldDefinitionExpression::Value(
            FieldValueExpression::Arithmetic(ArithmeticExpression::new(
                ArithmeticOperator::Subtract,
                ArithmeticBase::Column(Column {
                    name: String::from("max(o_id)"),
                    alias: None,
                    table: None,
                    function: Some(Box::new(FunctionExpression::Max(FunctionArgument::Column(
                        "o_id".into(),
                    )))),
                }),
                ArithmeticBase::Scalar(3333.into()),
                None,
            )),
        )],
        ..Default::default()
    };

    assert_eq!(res.unwrap().1, expected);
}

#[test]
fn project_arithmetic_expressions_with_aliases() {
    let qstr = "SELECT max(o_id) * 2 as double_max FROM orders;";
    let res = SelectStatement::parse(qstr);

    let expected = SelectStatement {
        tables: vec![Table::from("orders")],
        fields: vec![FieldDefinitionExpression::Value(
            FieldValueExpression::Arithmetic(ArithmeticExpression::new(
                ArithmeticOperator::Multiply,
                ArithmeticBase::Column(Column {
                    name: String::from("max(o_id)"),
                    alias: None,
                    table: None,
                    function: Some(Box::new(FunctionExpression::Max(FunctionArgument::Column(
                        "o_id".into(),
                    )))),
                }),
                ArithmeticBase::Scalar(2.into()),
                Some(String::from("double_max")),
            )),
        )],
        ..Default::default()
    };

    assert_eq!(res.unwrap().1, expected);
}

#[test]
fn where_in_clause() {
    let qstr = "SELECT `auth_permission`.`content_type_id`, `auth_permission`.`codename`
                    FROM `auth_permission`
                    JOIN `django_content_type`
                      ON ( `auth_permission`.`content_type_id` = `django_content_type`.`id` )
                    WHERE `auth_permission`.`content_type_id` IN (0);";
    let res = SelectStatement::parse(qstr);

    let expected_where_clause = Some(ComparisonOp(ConditionTree {
        left: Box::new(Base(ConditionBase::Field(Column::from(
            "auth_permission.content_type_id",
        )))),
        right: Box::new(Base(LiteralList(vec![0.into()]))),
        operator: Operator::In,
    }));

    let expected = SelectStatement {
        tables: vec![Table::from("auth_permission")],
        fields: vec![
            FieldDefinitionExpression::Col(Column::from("auth_permission.content_type_id")),
            FieldDefinitionExpression::Col(Column::from("auth_permission.codename")),
        ],
        join: vec![JoinClause {
            operator: JoinOperator::Join,
            right: JoinRightSide::Table(Table::from("django_content_type")),
            constraint: JoinConstraint::On(ComparisonOp(ConditionTree {
                operator: Operator::Equal,
                left: Box::new(Base(ConditionBase::Field(Column::from(
                    "auth_permission.content_type_id",
                )))),
                right: Box::new(Base(ConditionBase::Field(Column::from(
                    "django_content_type.id",
                )))),
            })),
        }],
        where_clause: expected_where_clause,
        ..Default::default()
    };

    assert_eq!(res.unwrap().1, expected);
}
