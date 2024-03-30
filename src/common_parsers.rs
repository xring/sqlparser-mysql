use std::fmt::Display;
use std::str;
use std::str::FromStr;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case, take, take_until, take_while, take_while1};
use nom::character::complete::{
    alpha1, alphanumeric1, char, digit1, line_ending, multispace0, multispace1,
};
use nom::character::is_alphanumeric;
use nom::combinator::opt;
use nom::combinator::{map, not, peek, recognize};
use nom::error::{ErrorKind, ParseError, VerboseError};
use nom::multi::{fold_many0, many0, many1, separated_list0};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::{IResult, InputLength, Parser};

use common::column::{Column, FunctionArgument, FunctionArguments, FunctionExpression};
use common::table::Table;
use common::trigger::Trigger;
use common::{
    FieldDefinitionExpression, FieldValueExpression, ItemPlaceholder, Literal, LiteralExpression,
    Operator, Real, SqlDataType,
};
use keywords::sql_keyword;
use zz_arithmetic::arithmetic_expression;
use zz_case::case_when_column;

#[inline]
pub fn is_sql_identifier(chr: char) -> bool {
    is_alphanumeric(chr as u8) || chr == '_' || chr == '@'
}

#[inline]
fn len_as_u16(len: &str) -> u16 {
    match u16::from_str(len) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    }
}

pub(crate) fn opt_delimited<I: Clone, O1, O2, O3, E: ParseError<I>, F, G, H>(
    mut first: F,
    mut second: G,
    mut third: H,
) -> impl FnMut(I) -> IResult<I, O2, E>
where
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    H: Parser<I, O3, E>,
{
    move |input: I| {
        let inp = input.clone();
        match second.parse(input) {
            Ok((i, o)) => Ok((i, o)),
            _ => {
                let (inp, _) = first.parse(inp)?;
                let (inp, o2) = second.parse(inp)?;
                third.parse(inp).map(|(i, _)| (i, o2))
            }
        }
    }
}

fn precision_helper(i: &str) -> IResult<&str, (u8, Option<u8>), VerboseError<&str>> {
    let (remaining_input, (m, d)) = tuple((
        digit1,
        opt(preceded(tag(","), preceded(multispace0, digit1))),
    ))(i)?;

    Ok((
        remaining_input,
        (m.parse().unwrap(), d.map(|r| r.parse().unwrap())),
    ))
}

pub fn precision(i: &str) -> IResult<&str, (u8, Option<u8>), VerboseError<&str>> {
    delimited(tag("("), precision_helper, tag(")"))(i)
}

fn opt_signed(i: &str) -> IResult<&str, Option<&str>, VerboseError<&str>> {
    opt(alt((tag_no_case("unsigned"), tag_no_case("signed"))))(i)
}

fn delim_digit(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(tag("("), digit1, tag(")"))(i)
}

// TODO: rather than copy paste these functions, should create a function that returns a parser
// based on the sql int type, just like nom does
fn tiny_int(i: &str) -> IResult<&str, SqlDataType, VerboseError<&str>> {
    let (remaining_input, (_, len, _, signed)) = tuple((
        tag_no_case("tinyint"),
        opt(delim_digit),
        multispace0,
        opt_signed,
    ))(i)?;

    match signed {
        Some(sign) => {
            if sign.eq_ignore_ascii_case("unsigned") {
                Ok((
                    remaining_input,
                    SqlDataType::UnsignedTinyint(len.map(|l| len_as_u16(l)).unwrap_or(1)),
                ))
            } else {
                Ok((
                    remaining_input,
                    SqlDataType::Tinyint(len.map(|l| len_as_u16(l)).unwrap_or(1)),
                ))
            }
        }
        None => Ok((
            remaining_input,
            SqlDataType::Tinyint(len.map(|l| len_as_u16(l)).unwrap_or(1)),
        )),
    }
}

// TODO: rather than copy paste these functions, should create a function that returns a parser
// based on the sql int type, just like nom does
fn big_int(i: &str) -> IResult<&str, SqlDataType, VerboseError<&str>> {
    let (remaining_input, (_, len, _, signed)) = tuple((
        tag_no_case("bigint"),
        opt(delim_digit),
        multispace0,
        opt_signed,
    ))(i)?;

    match signed {
        Some(sign) => {
            if sign.eq_ignore_ascii_case("unsigned") {
                Ok((
                    remaining_input,
                    SqlDataType::UnsignedBigint(len.map(|l| len_as_u16(l)).unwrap_or(1)),
                ))
            } else {
                Ok((
                    remaining_input,
                    SqlDataType::Bigint(len.map(|l| len_as_u16(l)).unwrap_or(1)),
                ))
            }
        }
        None => Ok((
            remaining_input,
            SqlDataType::Bigint(len.map(|l| len_as_u16(l)).unwrap_or(1)),
        )),
    }
}

// TODO: rather than copy paste these functions, should create a function that returns a parser
// based on the sql int type, just like nom does
fn sql_int_type(i: &str) -> IResult<&str, SqlDataType, VerboseError<&str>> {
    let (remaining_input, (_, len, _, signed)) = tuple((
        alt((
            tag_no_case("integer"),
            tag_no_case("int"),
            tag_no_case("smallint"),
        )),
        opt(delim_digit),
        multispace0,
        opt_signed,
    ))(i)?;

    match signed {
        Some(sign) => {
            if sign.eq_ignore_ascii_case("unsigned") {
                Ok((
                    remaining_input,
                    SqlDataType::UnsignedInt(len.map(|l| len_as_u16(l)).unwrap_or(32)),
                ))
            } else {
                Ok((
                    remaining_input,
                    SqlDataType::Int(len.map(|l| len_as_u16(l)).unwrap_or(32)),
                ))
            }
        }
        None => Ok((
            remaining_input,
            SqlDataType::Int(len.map(|l| len_as_u16(l)).unwrap_or(32)),
        )),
    }
}

// TODO(malte): not strictly ok to treat DECIMAL and NUMERIC as identical; the
// former has "at least" M precision, the latter "exactly".
// See https://dev.mysql.com/doc/refman/5.7/en/precision-math-decimal-characteristics.html
fn decimal_or_numeric(i: &str) -> IResult<&str, SqlDataType, VerboseError<&str>> {
    let (remaining_input, precision) = delimited(
        alt((tag_no_case("decimal"), tag_no_case("numeric"))),
        opt(precision),
        multispace0,
    )(i)?;

    match precision {
        None => Ok((remaining_input, SqlDataType::Decimal(32, 0))),
        Some((m, None)) => Ok((remaining_input, SqlDataType::Decimal(m, 0))),
        Some((m, Some(d))) => Ok((remaining_input, SqlDataType::Decimal(m, d))),
    }
}

fn type_identifier_first_half(i: &str) -> IResult<&str, SqlDataType, VerboseError<&str>> {
    alt((
        tiny_int,
        big_int,
        sql_int_type,
        map(tag_no_case("bool"), |_| SqlDataType::Bool),
        map(
            tuple((
                tag_no_case("char"),
                delim_digit,
                multispace0,
                opt(tag_no_case("binary")),
            )),
            |t| SqlDataType::Char(len_as_u16(t.1)),
        ),
        map(preceded(tag_no_case("datetime"), opt(delim_digit)), |fsp| {
            SqlDataType::DateTime(match fsp {
                Some(fsp) => len_as_u16(fsp),
                None => 0 as u16,
            })
        }),
        map(tag_no_case("date"), |_| SqlDataType::Date),
        map(
            tuple((tag_no_case("double"), multispace0, opt_signed)),
            |_| SqlDataType::Double,
        ),
        map(
            terminated(
                preceded(
                    tag_no_case("enum"),
                    delimited(tag("("), value_list, tag(")")),
                ),
                multispace0,
            ),
            |v| SqlDataType::Enum(v),
        ),
        map(
            tuple((
                tag_no_case("float"),
                multispace0,
                opt(precision),
                multispace0,
            )),
            |_| SqlDataType::Float,
        ),
        map(
            tuple((tag_no_case("real"), multispace0, opt_signed)),
            |_| SqlDataType::Real,
        ),
        map(tag_no_case("text"), |_| SqlDataType::Text),
        map(
            tuple((tag_no_case("timestamp"), opt(delim_digit), multispace0)),
            |_| SqlDataType::Timestamp,
        ),
        map(
            tuple((
                tag_no_case("varchar"),
                delim_digit,
                multispace0,
                opt(tag_no_case("binary")),
            )),
            |t| SqlDataType::Varchar(len_as_u16(t.1)),
        ),
        decimal_or_numeric,
    ))(i)
}

fn type_identifier_second_half(i: &str) -> IResult<&str, SqlDataType, VerboseError<&str>> {
    alt((
        map(
            tuple((tag_no_case("binary"), delim_digit, multispace0)),
            |t| SqlDataType::Binary(len_as_u16(t.1)),
        ),
        map(tag_no_case("blob"), |_| SqlDataType::Blob),
        map(tag_no_case("longblob"), |_| SqlDataType::Longblob),
        map(tag_no_case("mediumblob"), |_| SqlDataType::Mediumblob),
        map(tag_no_case("mediumtext"), |_| SqlDataType::Mediumtext),
        map(tag_no_case("longtext"), |_| SqlDataType::Longtext),
        map(tag_no_case("tinyblob"), |_| SqlDataType::Tinyblob),
        map(tag_no_case("tinytext"), |_| SqlDataType::Tinytext),
        map(
            tuple((tag_no_case("varbinary"), delim_digit, multispace0)),
            |t| SqlDataType::Varbinary(len_as_u16(t.1)),
        ),
    ))(i)
}

// A SQL type specifier.
pub fn type_identifier(i: &str) -> IResult<&str, SqlDataType, VerboseError<&str>> {
    alt((type_identifier_first_half, type_identifier_second_half))(i)
}

// Parses the argument for an aggregation function
pub fn function_argument_parser(i: &str) -> IResult<&str, FunctionArgument, VerboseError<&str>> {
    alt((
        map(case_when_column, |cw| FunctionArgument::Conditional(cw)),
        map(column_identifier_without_alias, |c| {
            FunctionArgument::Column(c)
        }),
    ))(i)
}

// Parses the arguments for an aggregation function, and also returns whether the distinct flag is
// present.
pub fn function_arguments(i: &str) -> IResult<&str, (FunctionArgument, bool), VerboseError<&str>> {
    let distinct_parser = opt(tuple((tag_no_case("distinct"), multispace1)));
    let (remaining_input, (distinct, args)) =
        tuple((distinct_parser, function_argument_parser))(i)?;
    Ok((remaining_input, (args, distinct.is_some())))
}

fn group_concat_fx_helper(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    let ws_sep = preceded(multispace0, tag_no_case("separator"));
    let (remaining_input, sep) = delimited(
        ws_sep,
        delimited(tag("'"), opt(alphanumeric1), tag("'")),
        multispace0,
    )(i)?;

    Ok((remaining_input, sep.unwrap_or("")))
}

fn group_concat_fx(i: &str) -> IResult<&str, (Column, Option<&str>), VerboseError<&str>> {
    pair(column_identifier_without_alias, opt(group_concat_fx_helper))(i)
}

fn delim_fx_args(i: &str) -> IResult<&str, (FunctionArgument, bool), VerboseError<&str>> {
    delimited(tag("("), function_arguments, tag(")"))(i)
}

pub fn column_function(i: &str) -> IResult<&str, FunctionExpression, VerboseError<&str>> {
    let delim_group_concat_fx = delimited(tag("("), group_concat_fx, tag(")"));
    alt((
        map(tag_no_case("count(*)"), |_| FunctionExpression::CountStar),
        map(preceded(tag_no_case("count"), delim_fx_args), |args| {
            FunctionExpression::Count(args.0.clone(), args.1)
        }),
        map(preceded(tag_no_case("sum"), delim_fx_args), |args| {
            FunctionExpression::Sum(args.0.clone(), args.1)
        }),
        map(preceded(tag_no_case("avg"), delim_fx_args), |args| {
            FunctionExpression::Avg(args.0.clone(), args.1)
        }),
        map(preceded(tag_no_case("max"), delim_fx_args), |args| {
            FunctionExpression::Max(args.0.clone())
        }),
        map(preceded(tag_no_case("min"), delim_fx_args), |args| {
            FunctionExpression::Min(args.0.clone())
        }),
        map(
            preceded(tag_no_case("group_concat"), delim_group_concat_fx),
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
                sql_identifier,
                multispace0,
                tag("("),
                separated_list0(
                    tag(","),
                    delimited(multispace0, function_argument_parser, multispace0),
                ),
                tag(")"),
            )),
            |tuple| {
                let (name, _, _, arguments, _) = tuple;
                FunctionExpression::Generic(name.to_string(), FunctionArguments::from(arguments))
            },
        ),
    ))(i)
}

// Parses a SQL column identifier in the table.column format
pub fn column_identifier_without_alias(i: &str) -> IResult<&str, Column, VerboseError<&str>> {
    let table_parser = pair(opt(terminated(sql_identifier, tag("."))), sql_identifier);
    alt((
        map(column_function, |f| Column {
            name: format!("{}", f),
            alias: None,
            table: None,
            function: Some(Box::new(f)),
        }),
        map(table_parser, |tup| Column {
            name: tup.1.to_string(),
            alias: None,
            table: match tup.0 {
                None => None,
                Some(t) => Some(t.to_string()),
            },
            function: None,
        }),
    ))(i)
}

// Parses a SQL column identifier in the table.column format
pub fn column_identifier(i: &str) -> IResult<&str, Column, VerboseError<&str>> {
    let col_func_no_table = map(pair(column_function, opt(as_alias)), |tup| Column {
        name: match tup.1 {
            None => format!("{}", tup.0),
            Some(a) => String::from(a),
        },
        alias: match tup.1 {
            None => None,
            Some(a) => Some(String::from(a)),
        },
        table: None,
        function: Some(Box::new(tup.0)),
    });
    let col_w_table = map(
        tuple((
            opt(terminated(sql_identifier, tag("."))),
            sql_identifier,
            opt(as_alias),
        )),
        |tup| Column {
            name: tup.1.to_string(),
            alias: match tup.2 {
                None => None,
                Some(a) => Some(String::from(a)),
            },
            table: match tup.0 {
                None => None,
                Some(t) => Some(t.to_string()),
            },
            function: None,
        },
    );
    alt((col_func_no_table, col_w_table))(i)
}

pub fn sql_identifier(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    alt((
        alt((
            preceded(
                not(peek(sql_keyword)),
                recognize(pair(
                    // start with letter
                    alpha1,
                    // 后面可以跟随任意数量的字母、数字、下划线或@
                    take_while(is_sql_identifier),
                )),
            ),
            recognize(pair(
                // 必须以字母开头
                tag("_"),
                // 后面可以跟随任意数量的字母、数字、下划线或@
                take_while1(is_sql_identifier),
            )),
        )),
        delimited(tag("`"), take_while1(is_sql_identifier), tag("`")),
        delimited(tag("["), take_while1(is_sql_identifier), tag("]")),
    ))(i)
}

// Parse an unsigned integer.
pub fn unsigned_number(i: &str) -> IResult<&str, u64, VerboseError<&str>> {
    map(digit1, |d| FromStr::from_str(d).unwrap())(i)
}

pub(crate) fn eof<I: Copy + InputLength, E: ParseError<I>>(input: I) -> IResult<I, I, E> {
    if input.input_len() == 0 {
        Ok((input, input))
    } else {
        Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Eof)))
    }
}

// Parse a terminator that ends a SQL statement.
pub fn statement_terminator(i: &str) -> IResult<&str, (), VerboseError<&str>> {
    let (remaining_input, _) =
        delimited(multispace0, alt((tag(";"), line_ending, eof)), multispace0)(i)?;

    Ok((remaining_input, ()))
}

// Parse binary comparison operators
pub fn binary_comparison_operator(i: &str) -> IResult<&str, Operator, VerboseError<&str>> {
    alt((
        map(tag_no_case("not_like"), |_| Operator::NotLike),
        map(tag_no_case("like"), |_| Operator::Like),
        map(tag_no_case("!="), |_| Operator::NotEqual),
        map(tag_no_case("<>"), |_| Operator::NotEqual),
        map(tag_no_case(">="), |_| Operator::GreaterOrEqual),
        map(tag_no_case("<="), |_| Operator::LessOrEqual),
        map(tag_no_case("="), |_| Operator::Equal),
        map(tag_no_case("<"), |_| Operator::Less),
        map(tag_no_case(">"), |_| Operator::Greater),
        map(tag_no_case("in"), |_| Operator::In),
    ))(i)
}

// Parse rule for AS-based aliases for SQL entities.
pub fn as_alias(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    map(
        tuple((
            multispace1,
            opt(pair(tag_no_case("as"), multispace1)),
            sql_identifier,
        )),
        |a| a.2,
    )(i)
}

fn field_value_expr(i: &str) -> IResult<&str, FieldValueExpression, VerboseError<&str>> {
    alt((
        map(literal, |l| {
            FieldValueExpression::Literal(LiteralExpression {
                value: l.into(),
                alias: None,
            })
        }),
        map(arithmetic_expression, |ae| {
            FieldValueExpression::Arithmetic(ae)
        }),
    ))(i)
}

fn assignment_expr(i: &str) -> IResult<&str, (Column, FieldValueExpression), VerboseError<&str>> {
    separated_pair(
        column_identifier_without_alias,
        delimited(multispace0, tag("="), multispace0),
        field_value_expr,
    )(i)
}

pub fn ws_sep_comma(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(multispace0, tag(","), multispace0)(i)
}

pub(crate) fn ws_sep_equals(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(multispace0, tag("="), multispace0)(i)
}

pub fn assignment_expr_list(
    i: &str,
) -> IResult<&str, Vec<(Column, FieldValueExpression)>, VerboseError<&str>> {
    many1(terminated(assignment_expr, opt(ws_sep_comma)))(i)
}

// Parse rule for a comma-separated list of fields without aliases.
pub fn field_list(i: &str) -> IResult<&str, Vec<Column>, VerboseError<&str>> {
    many0(terminated(
        column_identifier_without_alias,
        opt(ws_sep_comma),
    ))(i)
}

// Parse list of column/field definitions.
pub fn field_definition_expr(
    i: &str,
) -> IResult<&str, Vec<FieldDefinitionExpression>, VerboseError<&str>> {
    many0(terminated(
        alt((
            map(tag("*"), |_| FieldDefinitionExpression::All),
            map(terminated(table_reference, tag(".*")), |t| {
                FieldDefinitionExpression::AllInTable(t.name.clone())
            }),
            map(arithmetic_expression, |expr| {
                FieldDefinitionExpression::Value(FieldValueExpression::Arithmetic(expr))
            }),
            map(literal_expression, |lit| {
                FieldDefinitionExpression::Value(FieldValueExpression::Literal(lit))
            }),
            map(column_identifier, |col| FieldDefinitionExpression::Col(col)),
        )),
        opt(ws_sep_comma),
    ))(i)
}

// Parse list of table names.
// XXX(malte): add support for aliases
pub fn table_list(i: &str) -> IResult<&str, Vec<Table>, VerboseError<&str>> {
    many0(terminated(schema_table_reference, opt(ws_sep_comma)))(i)
}

// Integer literal value
pub fn integer_literal(i: &str) -> IResult<&str, Literal, VerboseError<&str>> {
    map(pair(opt(tag("-")), digit1), |tup| {
        let mut intval = i64::from_str(tup.1).unwrap();
        if (tup.0).is_some() {
            intval *= -1;
        }
        Literal::Integer(intval)
    })(i)
}

fn unpack(v: &str) -> i32 {
    i32::from_str(v).unwrap()
}

// Floating point literal value
pub fn float_literal(i: &str) -> IResult<&str, Literal, VerboseError<&str>> {
    map(tuple((opt(tag("-")), digit1, tag("."), digit1)), |tup| {
        Literal::FixedPoint(Real {
            integral: if (tup.0).is_some() {
                -1 * unpack(tup.1)
            } else {
                unpack(tup.1)
            },
            fractional: unpack(tup.3) as i32,
        })
    })(i)
}

/// String literal value
fn raw_string_quoted(
    input: &str,
    is_single_quote: bool,
) -> IResult<&str, String, VerboseError<&str>> {
    // Adjusted to work with &str
    let quote_char = if is_single_quote { '\'' } else { '"' };
    let quote_str = if is_single_quote { "\'" } else { "\"" };
    let double_quote_str = if is_single_quote { "\'\'" } else { "\"\"" };
    let backslash_quote = if is_single_quote { "\\\'" } else { "\\\"" };

    delimited(
        tag(quote_str),
        fold_many0(
            alt((
                is_not(backslash_quote),
                map(tag(double_quote_str), |_| {
                    if is_single_quote {
                        "\'"
                    } else {
                        "\""
                    }
                }),
                map(tag("\\\\"), |_| "\\"),
                map(tag("\\b"), |_| "\x08"), // 注意：\x7f 是 DEL，\x08 是退格
                map(tag("\\r"), |_| "\r"),
                map(tag("\\n"), |_| "\n"),
                map(tag("\\t"), |_| "\t"),
                map(tag("\\0"), |_| "\0"),
                map(tag("\\Z"), |_| "\x1A"),
                preceded(tag("\\"), take(1usize)),
            )),
            || String::new(),
            |mut acc: String, bytes: &str| {
                acc.push_str(bytes);
                acc
            },
        ),
        tag(quote_str),
    )(input)
}

fn raw_string_single_quoted(i: &str) -> IResult<&str, String, VerboseError<&str>> {
    raw_string_quoted(i, true)
}

fn raw_string_double_quoted(i: &str) -> IResult<&str, String, VerboseError<&str>> {
    raw_string_quoted(i, false)
}

pub fn string_literal(i: &str) -> IResult<&str, Literal, VerboseError<&str>> {
    map(
        alt((raw_string_single_quoted, raw_string_double_quoted)),
        |str| Literal::String(str),
    )(i)
}

// Any literal value.
pub fn literal(i: &str) -> IResult<&str, Literal, VerboseError<&str>> {
    alt((
        float_literal,
        integer_literal,
        string_literal,
        map(tag_no_case("null"), |_| Literal::Null),
        map(tag_no_case("current_timestamp"), |_| {
            Literal::CurrentTimestamp
        }),
        map(tag_no_case("current_date"), |_| Literal::CurrentDate),
        map(tag_no_case("current_time"), |_| Literal::CurrentTime),
        map(tag("?"), |_| {
            Literal::Placeholder(ItemPlaceholder::QuestionMark)
        }),
        map(preceded(tag(":"), digit1), |num| {
            let value = i32::from_str(num).unwrap();
            Literal::Placeholder(ItemPlaceholder::ColonNumber(value))
        }),
        map(preceded(tag("$"), digit1), |num| {
            let value = i32::from_str(num).unwrap();
            Literal::Placeholder(ItemPlaceholder::DollarNumber(value))
        }),
    ))(i)
}

pub fn literal_expression(i: &str) -> IResult<&str, LiteralExpression, VerboseError<&str>> {
    map(
        pair(opt_delimited(tag("("), literal, tag(")")), opt(as_alias)),
        |p| LiteralExpression {
            value: p.0,
            alias: (p.1).map(|a| a.to_string()),
        },
    )(i)
}

// Parse a list of values (e.g., for INSERT syntax).
pub fn value_list(i: &str) -> IResult<&str, Vec<Literal>, VerboseError<&str>> {
    many0(delimited(multispace0, literal, opt(ws_sep_comma)))(i)
}

// Parse a reference to a named schema.table, with an optional alias
pub fn schema_table_reference(i: &str) -> IResult<&str, Table, VerboseError<&str>> {
    map(
        tuple((
            opt(pair(sql_identifier, tag("."))),
            sql_identifier,
            opt(as_alias),
        )),
        |tup| Table {
            name: String::from(tup.1),
            alias: match tup.2 {
                Some(a) => Some(String::from(a)),
                None => None,
            },
            schema: match tup.0 {
                Some((schema, _)) => Some(String::from(schema)),
                None => None,
            },
        },
    )(i)
}

/// table alias not allowed in DROP/TRUNCATE/RENAME TABLE statement
pub fn schema_table_name_without_alias(i: &str) -> IResult<&str, Table, VerboseError<&str>> {
    map(
        tuple((opt(pair(sql_identifier, tag("."))), sql_identifier)),
        |tup| Table {
            name: String::from(tup.1),
            alias: None,
            schema: match tup.0 {
                Some((schema, _)) => Some(String::from(schema)),
                None => None,
            },
        },
    )(i)
}

pub fn schema_trigger_name(i: &str) -> IResult<&str, Trigger, VerboseError<&str>> {
    map(
        tuple((opt(pair(sql_identifier, tag("."))), sql_identifier)),
        |tup| Trigger {
            name: String::from(tup.1),
            schema: match tup.0 {
                Some((schema, _)) => Some(String::from(schema)),
                None => None,
            },
        },
    )(i)
}

/// db_name.tb_name TO db_name.tb_name
pub fn schema_table_reference_to_schema_table_reference(
    i: &str,
) -> IResult<&str, (Table, Table), VerboseError<&str>> {
    map(
        tuple((
            schema_table_name_without_alias, // 解析起始表名
            multispace0,
            tag_no_case("TO "),
            multispace0,
            schema_table_name_without_alias,
        )),
        |(from, _, _, _, to)| (from, to),
    )(i)
}

// Parse a reference to a named table, with an optional alias
pub fn table_reference(i: &str) -> IResult<&str, Table, VerboseError<&str>> {
    map(pair(sql_identifier, opt(as_alias)), |tup| Table {
        name: String::from(tup.0),
        alias: match tup.1 {
            Some(a) => Some(String::from(a)),
            None => None,
        },
        schema: None,
    })(i)
}

// Parse rule for a comment part.
pub fn parse_comment(i: &str) -> IResult<&str, String, VerboseError<&str>> {
    map(
        preceded(
            delimited(multispace0, tag_no_case("comment"), multispace1),
            delimited(tag("'"), take_until("'"), tag("'")),
        ),
        |comment| String::from(comment),
    )(i)
}

pub fn parse_if_exists(i: &str) -> IResult<&str, Option<&str>, VerboseError<&str>> {
    opt(delimited(
        multispace0,
        delimited(tag_no_case("IF"), multispace1, tag_no_case("EXISTS ")),
        multispace0,
    ))(i)
}

#[cfg(test)]
mod tests {
    use common::column::{Column, FunctionArgument, FunctionArguments, FunctionExpression};
    use common::SqlDataType;

    use super::*;

    #[test]
    fn sql_identifiers() {
        let id1 = "foo";
        let id2 = "f_o_o";
        let id3 = "foo12";
        let id4 = ":fo oo";
        let id5 = "primary ";
        let id6 = "`primary`";

        assert!(sql_identifier(id1).is_ok());
        assert!(sql_identifier(id2).is_ok());
        assert!(sql_identifier(id3).is_ok());
        assert!(sql_identifier(id4).is_err());
        assert!(sql_identifier(id5).is_err());
        assert!(sql_identifier(id6).is_ok());
    }

    fn test_opt_delimited_fn_call(i: &str) -> IResult<&str, &str> {
        opt_delimited(tag("("), tag("abc"), tag(")"))(i)
    }

    #[test]
    fn opt_delimited_tests() {
        // let ok1 = IResult::Ok(("".as_bytes(), "abc".as_bytes()));
        assert_eq!(test_opt_delimited_fn_call("abc"), IResult::Ok(("", "abc")));
        assert_eq!(
            test_opt_delimited_fn_call("(abc)"),
            IResult::Ok(("", "abc"))
        );
        assert!(test_opt_delimited_fn_call("(abc").is_err());
        assert_eq!(
            test_opt_delimited_fn_call("abc)"),
            IResult::Ok((")", "abc"))
        );
        assert!(test_opt_delimited_fn_call("ab").is_err());
    }

    #[test]
    fn sql_types() {
        let ok = ["bool", "integer(16)", "datetime(16)"];
        let not_ok = ["varchar"];

        let res_ok: Vec<_> = ok.iter().map(|t| type_identifier(t).unwrap().1).collect();
        let res_not_ok: Vec<_> = not_ok.iter().map(|t| type_identifier(t).is_ok()).collect();

        assert_eq!(
            res_ok,
            vec![
                SqlDataType::Bool,
                SqlDataType::Int(16),
                SqlDataType::DateTime(16)
            ]
        );

        assert!(res_not_ok.into_iter().all(|r| r == false));
    }

    #[test]
    fn simple_column_function() {
        let qs = "max(addr_id)";

        let res = column_identifier(qs);
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
            let res = column_function(q);
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

    #[test]
    fn comment_data() {
        let res = parse_comment(" COMMENT 'test'");
        assert_eq!(res.unwrap().1, "test");
    }

    #[test]
    fn literal_string_single_backslash_escape() {
        let all_escaped = r#"\0\'\"\b\n\r\t\Z\\\%\_"#;
        for quote in [&"'"[..], &"\""[..]].iter() {
            let quoted = &[quote, &all_escaped[..], quote].concat();
            let res = string_literal(quoted);
            let expected = Literal::String("\0\'\"\x7F\n\r\t\x1a\\%_".to_string());
            assert_eq!(res, Ok(("", expected)));
        }
    }

    #[test]
    fn literal_string_single_quote() {
        let res = string_literal("'a''b'");
        let expected = Literal::String("a'b".to_string());
        assert_eq!(res, Ok(("", expected)));
    }

    #[test]
    fn literal_string_double_quote() {
        let res = string_literal(r#""a""b""#);
        let expected = Literal::String(r#"a"b"#.to_string());
        assert_eq!(res, Ok(("", expected)));
    }

    #[test]
    fn terminated_by_semicolon() {
        let res = statement_terminator("   ;  ");
        assert_eq!(res, Ok(("", ())));
    }
}
