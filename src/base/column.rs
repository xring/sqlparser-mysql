use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::str;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until};
use nom::character::complete::{alphanumeric1, digit1, multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use base::error::ParseSQLErrorKind;
use base::keywords::escape_if_keyword;
use base::{CaseWhenExpression, CommonParser, DataType, Literal, ParseSQLError, Real};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FunctionExpression {
    Avg(FunctionArgument, bool),
    Count(FunctionArgument, bool),
    CountStar,
    Sum(FunctionArgument, bool),
    Max(FunctionArgument),
    Min(FunctionArgument),
    GroupConcat(FunctionArgument, String),
    Generic(String, FunctionArguments),
}

impl FunctionExpression {
    pub fn parse(i: &str) -> IResult<&str, FunctionExpression, ParseSQLError<&str>> {
        let delim_group_concat_fx = delimited(tag("("), Self::group_concat_fx, tag(")"));
        alt((
            map(tag_no_case("COUNT(*)"), |_| FunctionExpression::CountStar),
            map(
                preceded(tag_no_case("COUNT"), FunctionArgument::delim_fx_args),
                |args| FunctionExpression::Count(args.0.clone(), args.1),
            ),
            map(
                preceded(tag_no_case("SUM"), FunctionArgument::delim_fx_args),
                |args| FunctionExpression::Sum(args.0.clone(), args.1),
            ),
            map(
                preceded(tag_no_case("AVG"), FunctionArgument::delim_fx_args),
                |args| FunctionExpression::Avg(args.0.clone(), args.1),
            ),
            map(
                preceded(tag_no_case("MAX"), FunctionArgument::delim_fx_args),
                |args| FunctionExpression::Max(args.0.clone()),
            ),
            map(
                preceded(tag_no_case("MIN"), FunctionArgument::delim_fx_args),
                |args| FunctionExpression::Min(args.0.clone()),
            ),
            map(
                preceded(tag_no_case("GROUP_CONCAT"), delim_group_concat_fx),
                |spec| {
                    let (ref col, ref sep) = spec;
                    let sep = match *sep {
                        // default separator is a comma, see MySQL manual §5.7
                        None => String::from(","),
                        Some(s) => String::from(s),
                    };
                    FunctionExpression::GroupConcat(FunctionArgument::Column(col.clone()), sep)
                },
            ),
            map(
                tuple((
                    CommonParser::sql_identifier,
                    multispace0,
                    tag("("),
                    separated_list0(
                        tag(","),
                        delimited(multispace0, FunctionArgument::parse, multispace0),
                    ),
                    tag(")"),
                )),
                |tuple| {
                    let (name, _, _, arguments, _) = tuple;
                    FunctionExpression::Generic(
                        name.to_string(),
                        FunctionArguments::from(arguments),
                    )
                },
            ),
        ))(i)
    }

    fn group_concat_fx_helper(i: &str) -> IResult<&str, &str, ParseSQLError<&str>> {
        let ws_sep = preceded(multispace0, tag_no_case("separator"));
        let (remaining_input, sep) = delimited(
            ws_sep,
            delimited(tag("'"), opt(alphanumeric1), tag("'")),
            multispace0,
        )(i)?;

        Ok((remaining_input, sep.unwrap_or("")))
    }

    fn group_concat_fx(i: &str) -> IResult<&str, (Column, Option<&str>), ParseSQLError<&str>> {
        pair(Column::without_alias, opt(Self::group_concat_fx_helper))(i)
    }
}

impl Display for FunctionExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FunctionExpression::Avg(ref col, d) if d => write!(f, "avg(distinct {})", col),
            FunctionExpression::Count(ref col, d) if d => write!(f, "count(distinct {})", col),
            FunctionExpression::Sum(ref col, d) if d => write!(f, "sum(distinct {})", col),
            FunctionExpression::Avg(ref col, _) => write!(f, "avg({})", col),
            FunctionExpression::Count(ref col, _) => write!(f, "count({})", col),
            FunctionExpression::CountStar => write!(f, "count(*)"),
            FunctionExpression::Sum(ref col, _) => write!(f, "sum({})", col),
            FunctionExpression::Max(ref col) => write!(f, "max({})", col),
            FunctionExpression::Min(ref col) => write!(f, "min({})", col),
            FunctionExpression::GroupConcat(ref col, ref s) => {
                write!(f, "group_concat({}, {})", col, s)
            }
            FunctionExpression::Generic(ref name, ref args) => write!(f, "{}({})", name, args),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FunctionArguments {
    pub arguments: Vec<FunctionArgument>,
}

impl Display for FunctionArguments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.arguments
                .iter()
                .map(|arg| format!("{}", arg))
                .collect::<Vec<String>>()
                .join(",")
        )?;
        Ok(())
    }
}

impl From<Vec<FunctionArgument>> for FunctionArguments {
    fn from(args: Vec<FunctionArgument>) -> FunctionArguments {
        FunctionArguments { arguments: args }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum FunctionArgument {
    Column(Column),
    Conditional(CaseWhenExpression),
}

impl FunctionArgument {
    // Parses the argument for an aggregation function
    pub fn parse(i: &str) -> IResult<&str, FunctionArgument, ParseSQLError<&str>> {
        alt((
            map(CaseWhenExpression::parse, |cw| {
                FunctionArgument::Conditional(cw)
            }),
            map(Column::without_alias, FunctionArgument::Column),
        ))(i)
    }

    // Parses the arguments for an aggregation function, and also returns whether the distinct flag is
    // present.
    fn function_arguments(i: &str) -> IResult<&str, (FunctionArgument, bool), ParseSQLError<&str>> {
        let distinct_parser = opt(tuple((tag_no_case("distinct"), multispace1)));
        let (remaining_input, (distinct, args)) =
            tuple((distinct_parser, FunctionArgument::parse))(i)?;
        Ok((remaining_input, (args, distinct.is_some())))
    }

    pub fn delim_fx_args(i: &str) -> IResult<&str, (FunctionArgument, bool), ParseSQLError<&str>> {
        delimited(tag("("), Self::function_arguments, tag(")"))(i)
    }
}

impl Display for FunctionArgument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FunctionArgument::Column(ref col) => write!(f, "{}", col)?,
            FunctionArgument::Conditional(ref e) => {
                write!(f, "{}", e)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub alias: Option<String>,
    pub table: Option<String>,
    pub function: Option<Box<FunctionExpression>>,
}

impl Column {
    pub fn index_col_list(i: &str) -> IResult<&str, Vec<Column>, ParseSQLError<&str>> {
        many0(map(
            terminated(
                CommonParser::index_col_name,
                opt(CommonParser::ws_sep_comma),
            ),
            // XXX(malte): ignores length and order
            |e| e.0,
        ))(i)
    }

    // Parse rule for a comma-separated list of fields without aliases.
    pub fn field_list(i: &str) -> IResult<&str, Vec<Column>, ParseSQLError<&str>> {
        many0(terminated(
            Column::without_alias,
            opt(CommonParser::ws_sep_comma),
        ))(i)
    }

    // Parses a SQL column identifier in the column format
    pub fn without_alias(i: &str) -> IResult<&str, Column, ParseSQLError<&str>> {
        let table_parser = pair(
            opt(terminated(CommonParser::sql_identifier, tag("."))),
            CommonParser::sql_identifier,
        );
        alt((
            map(FunctionExpression::parse, |f| Column {
                name: format!("{}", f),
                alias: None,
                table: None,
                function: Some(Box::new(f)),
            }),
            map(table_parser, |tup| Column {
                name: tup.1.to_string(),
                alias: None,
                table: tup.0.map(|t| t.to_string()),
                function: None,
            }),
        ))(i)
    }

    // Parses a SQL column identifier in the table.column format
    pub fn parse(i: &str) -> IResult<&str, Column, ParseSQLError<&str>> {
        let col_func_no_table = map(
            pair(FunctionExpression::parse, opt(CommonParser::as_alias)),
            |tup| Column {
                name: match tup.1 {
                    None => format!("{}", tup.0),
                    Some(a) => String::from(a),
                },
                alias: tup.1.map(String::from),
                table: None,
                function: Some(Box::new(tup.0)),
            },
        );
        let col_w_table = map(
            tuple((
                opt(terminated(CommonParser::sql_identifier, tag("."))),
                CommonParser::sql_identifier,
                opt(CommonParser::as_alias),
            )),
            |tup| Column {
                name: tup.1.to_string(),
                alias: tup.2.map(String::from),
                table: tup.0.map(|t| t.to_string()),
                function: None,
            },
        );
        alt((col_func_no_table, col_w_table))(i)
    }
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref table) = self.table {
            write!(
                f,
                "{}.{}",
                escape_if_keyword(table),
                escape_if_keyword(&self.name)
            )?;
        } else if let Some(ref function) = self.function {
            write!(f, "{}", *function)?;
        } else {
            write!(f, "{}", escape_if_keyword(&self.name))?;
        }
        if let Some(ref alias) = self.alias {
            write!(f, " AS {}", escape_if_keyword(alias))?;
        }
        Ok(())
    }
}

impl From<String> for Column {
    fn from(value: String) -> Self {
        match value.find('.') {
            None => Column {
                name: value,
                alias: None,
                table: None,
                function: None,
            },
            Some(i) => Column {
                name: String::from(&value[i + 1..]),
                alias: None,
                table: Some(String::from(&value[0..i])),
                function: None,
            },
        }
    }
}

impl<'a> From<&'a str> for Column {
    fn from(c: &str) -> Column {
        match c.find('.') {
            None => Column {
                name: String::from(c),
                alias: None,
                table: None,
                function: None,
            },
            Some(i) => Column {
                name: String::from(&c[i + 1..]),
                alias: None,
                table: Some(String::from(&c[0..i])),
                function: None,
            },
        }
    }
}

impl Ord for Column {
    fn cmp(&self, other: &Column) -> Ordering {
        if self.table.is_some() && other.table.is_some() {
            match self.table.cmp(&other.table) {
                Ordering::Equal => self.name.cmp(&other.name),
                x => x,
            }
        } else {
            self.name.cmp(&other.name)
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Column {
    fn partial_cmp(&self, other: &Column) -> Option<Ordering> {
        if self.table.is_some() && other.table.is_some() {
            match self.table.cmp(&other.table) {
                Ordering::Equal => Some(self.name.cmp(&other.name)),
                x => Some(x),
            }
        } else if self.table.is_none() && other.table.is_none() {
            Some(self.name.cmp(&other.name))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ColumnConstraint {
    NotNull,
    Null,
    CharacterSet(String),
    Collation(String),
    DefaultValue(Literal),
    AutoIncrement,
    PrimaryKey,
    Unique,
    OnUpdate(Literal),
}

impl ColumnConstraint {
    pub fn parse(i: &str) -> IResult<&str, Option<ColumnConstraint>, ParseSQLError<&str>> {
        let not_null = map(
            delimited(multispace0, tag_no_case("NOT NULL"), multispace0),
            |_| Some(ColumnConstraint::NotNull),
        );
        let null = map(
            delimited(multispace0, tag_no_case("NULL"), multispace0),
            |_| Some(ColumnConstraint::Null),
        );
        let auto_increment = map(
            delimited(multispace0, tag_no_case("AUTO_INCREMENT"), multispace0),
            |_| Some(ColumnConstraint::AutoIncrement),
        );
        let primary_key = map(
            delimited(multispace0, tag_no_case("PRIMARY KEY"), multispace0),
            |_| Some(ColumnConstraint::PrimaryKey),
        );
        let unique = map(
            delimited(multispace0, tag_no_case("UNIQUE"), multispace0),
            |_| Some(ColumnConstraint::Unique),
        );
        let character_set = map(
            preceded(
                delimited(multispace0, tag_no_case("CHARACTER SET"), multispace1),
                alt((
                    CommonParser::sql_identifier,
                    delimited(tag("'"), CommonParser::sql_identifier, tag("'")),
                    delimited(tag("\""), CommonParser::sql_identifier, tag("\"")),
                )),
            ),
            |cs| {
                let char_set = cs.to_owned();
                Some(ColumnConstraint::CharacterSet(char_set))
            },
        );
        let collate = map(
            preceded(
                delimited(multispace0, tag_no_case("COLLATE"), multispace1),
                alt((
                    CommonParser::sql_identifier,
                    delimited(tag("'"), CommonParser::sql_identifier, tag("'")),
                    delimited(tag("\""), CommonParser::sql_identifier, tag("\"")),
                )),
            ),
            |c| {
                let collation = c.to_owned();
                Some(ColumnConstraint::Collation(collation))
            },
        );
        // https://dev.mysql.com/doc/refman/5.7/en/timestamp-initialization.html
        // for timestamp only, part of constraint
        let on_update = map(
            tuple((
                tag_no_case("ON"),
                multispace1,
                tag_no_case("UPDATE"),
                multispace1,
                tag_no_case("CURRENT_TIMESTAMP"),
                opt(CommonParser::delim_digit),
            )),
            |_| Some(ColumnConstraint::OnUpdate(Literal::CurrentTimestamp)),
        );

        alt((
            not_null,
            null,
            auto_increment,
            Self::default,
            primary_key,
            unique,
            character_set,
            collate,
            on_update,
        ))(i)
    }

    fn default(i: &str) -> IResult<&str, Option<ColumnConstraint>, ParseSQLError<&str>> {
        let (remaining_input, (_, _, _, def, _)) = tuple((
            multispace0,
            tag_no_case("DEFAULT"),
            multispace1,
            alt((
                map(delimited(tag("'"), take_until("'"), tag("'")), |s| {
                    Literal::String(String::from(s))
                }),
                map(delimited(tag("\""), take_until("\""), tag("\"")), |s| {
                    Literal::String(String::from(s))
                }),
                map(tuple((digit1, tag("."), digit1)), |(i, _, f)| {
                    Literal::FixedPoint(Real {
                        integral: i32::from_str(i).unwrap(),
                        fractional: i32::from_str(f).unwrap(),
                    })
                }),
                map(tuple((opt(tag("-")), digit1)), |d: (Option<&str>, &str)| {
                    let d_i64: i64 = d.1.parse().unwrap();
                    if d.0.is_some() {
                        Literal::Integer(-d_i64)
                    } else {
                        Literal::Integer(d_i64)
                    }
                }),
                map(tag("''"), |_| Literal::String(String::from(""))),
                map(tag_no_case("NULL"), |_| Literal::Null),
                map(tag_no_case("FALSE"), |_| Literal::Bool(false)),
                map(tag_no_case("TRUE"), |_| Literal::Bool(true)),
                map(
                    tuple((
                        tag_no_case("CURRENT_TIMESTAMP"),
                        opt(CommonParser::delim_digit),
                    )),
                    |_| Literal::CurrentTimestamp,
                ),
            )),
            multispace0,
        ))(i)?;

        Ok((remaining_input, Some(ColumnConstraint::DefaultValue(def))))
    }
}

impl fmt::Display for ColumnConstraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ColumnConstraint::NotNull => write!(f, "NOT NULL"),
            ColumnConstraint::Null => write!(f, "NULL"),
            ColumnConstraint::CharacterSet(ref charset) => write!(f, "CHARACTER SET {}", charset),
            ColumnConstraint::Collation(ref collation) => write!(f, "COLLATE {}", collation),
            ColumnConstraint::DefaultValue(ref literal) => {
                write!(f, "DEFAULT {}", literal.to_string())
            }
            ColumnConstraint::AutoIncrement => write!(f, "AutoIncrement"),
            ColumnConstraint::PrimaryKey => write!(f, "PRIMARY KEY"),
            ColumnConstraint::Unique => write!(f, "UNIQUE"),
            ColumnConstraint::OnUpdate(ref ts) => write!(f, "ON UPDATE CURRENT_TIMESTAMP"),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ColumnPosition {
    First,
    After(Column),
}

impl ColumnPosition {
    pub fn parse(i: &str) -> IResult<&str, ColumnPosition, ParseSQLError<&str>> {
        alt((
            map(
                tuple((multispace0, tag_no_case("FIRST"), multispace0)),
                |_| ColumnPosition::First,
            ),
            map(
                tuple((
                    multispace0,
                    tag_no_case("AFTER"),
                    multispace1,
                    CommonParser::sql_identifier,
                )),
                |(_, _, _, identifier)| ColumnPosition::After(String::from(identifier).into()),
            ),
        ))(i)
    }
}

impl Display for ColumnPosition {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ColumnPosition::First => Ok(write!(f, "FIRST")?),
            ColumnPosition::After(column) => {
                let column_name = match &column.table {
                    Some(table) => format!("{}.{}", table, &column.name),
                    None => column.name.to_string(),
                };
                Ok(write!(f, "AFTER {column_name}")?)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ColumnSpecification {
    pub column: Column,
    pub sql_type: DataType,
    pub constraints: Vec<ColumnConstraint>,
    pub comment: Option<String>,
    pub position: Option<ColumnPosition>,
}

impl ColumnSpecification {
    pub fn parse(i: &str) -> IResult<&str, ColumnSpecification, ParseSQLError<&str>> {
        let mut parser = tuple((
            Column::without_alias,
            opt(delimited(
                multispace1,
                DataType::type_identifier,
                multispace0,
            )),
            many0(ColumnConstraint::parse),
            opt(CommonParser::parse_comment),
            opt(ColumnPosition::parse),
            opt(CommonParser::ws_sep_comma),
        ));

        match parser(i) {
            Ok((input, (column, field_type, constraints, comment, position, _))) => {
                if field_type.is_none() {
                    let error = ParseSQLError {
                        errors: vec![(i, ParseSQLErrorKind::Context("data type is empty"))],
                    };
                    return Err(nom::Err::Error(error));
                }

                let sql_type = field_type.unwrap();
                Ok((
                    input,
                    ColumnSpecification {
                        column,
                        sql_type,
                        constraints: constraints.into_iter().flatten().collect(),
                        comment,
                        position,
                    },
                ))
            }
            Err(err) => Err(err),
        }
    }
}

impl fmt::Display for ColumnSpecification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            escape_if_keyword(&self.column.name),
            self.sql_type
        )?;
        for constraint in self.constraints.iter() {
            write!(f, " {}", constraint)?;
        }
        if let Some(ref comment) = self.comment {
            write!(f, " COMMENT '{}'", comment)?;
        }
        Ok(())
    }
}

impl ColumnSpecification {
    pub fn new(column: Column, sql_type: DataType) -> ColumnSpecification {
        ColumnSpecification {
            column,
            sql_type,
            constraints: vec![],
            comment: None,
            position: None,
        }
    }

    pub fn with_constraints(
        column: Column,
        sql_type: DataType,
        constraints: Vec<ColumnConstraint>,
    ) -> ColumnSpecification {
        ColumnSpecification {
            column,
            sql_type,
            constraints,
            comment: None,
            position: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_from_str() {
        let s = "table.col";
        let c = Column::from(s);

        assert_eq!(
            c,
            Column {
                name: String::from("col"),
                alias: None,
                table: Some(String::from("table")),
                function: None,
            }
        );
    }

    #[test]
    fn print_function_column() {
        let c1 = Column {
            name: "".into(), // must be present, but will be ignored
            alias: Some("foo".into()),
            table: None,
            function: Some(Box::new(FunctionExpression::CountStar)),
        };
        let c2 = Column {
            name: "".into(), // must be present, but will be ignored
            alias: None,
            table: None,
            function: Some(Box::new(FunctionExpression::CountStar)),
        };
        let c3 = Column {
            name: "".into(), // must be present, but will be ignored
            alias: None,
            table: None,
            function: Some(Box::new(FunctionExpression::Sum(
                FunctionArgument::Column(Column::from("mytab.foo")),
                false,
            ))),
        };

        assert_eq!(format!("{}", c1), "count(*) AS foo");
        assert_eq!(format!("{}", c2), "count(*)");
        assert_eq!(format!("{}", c3), "sum(mytab.foo)");
    }

    #[test]
    fn simple_column_function() {
        let qs = "max(addr_id)";

        let res = Column::parse(qs);
        let expected = Column {
            name: String::from("max(addr_id)"),
            alias: None,
            table: None,
            function: Some(Box::new(FunctionExpression::Max(FunctionArgument::Column(
                Column::from("addr_id"),
            )))),
        };
        assert_eq!(res.unwrap().1, expected);
    }

    #[test]
    fn simple_generic_function() {
        let qlist = [
            "coalesce(a,b,c)",
            "coalesce (a,b,c)",
            "coalesce(a ,b,c)",
            "coalesce(a, b,c)",
        ];
        for q in qlist.iter() {
            let res = FunctionExpression::parse(q);
            let expected = FunctionExpression::Generic(
                "coalesce".to_string(),
                FunctionArguments::from(vec![
                    FunctionArgument::Column(Column::from("a")),
                    FunctionArgument::Column(Column::from("b")),
                    FunctionArgument::Column(Column::from("c")),
                ]),
            );
            assert_eq!(res, Ok(("", expected)));
        }
    }
}
