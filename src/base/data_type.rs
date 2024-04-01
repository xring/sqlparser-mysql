use std::fmt;
use std::str::FromStr;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::error::VerboseError;
use nom::IResult;
use nom::sequence::{delimited, preceded, terminated, tuple};
use base::Literal;
use common::{delim_digit, precision};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Bool,
    Char(u16),
    Varchar(u16),
    Int(u16),
    UnsignedInt(u16),
    Bigint(u16),
    UnsignedBigint(u16),
    Tinyint(u16),
    UnsignedTinyint(u16),
    Blob,
    Longblob,
    Mediumblob,
    Tinyblob,
    Double,
    Float,
    Real,
    Tinytext,
    Mediumtext,
    Longtext,
    Text,
    Json,
    Date,
    DateTime(u16),
    Timestamp,
    Binary(u16),
    Varbinary(u16),
    Enum(Vec<Literal>),
    Decimal(u8, u8),
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DataType::Bool => write!(f, "BOOL"),
            DataType::Char(len) => write!(f, "CHAR({})", len),
            DataType::Varchar(len) => write!(f, "VARCHAR({})", len),
            DataType::Int(len) => write!(f, "INT({})", len),
            DataType::UnsignedInt(len) => write!(f, "INT({}) UNSIGNED", len),
            DataType::Bigint(len) => write!(f, "BIGINT({})", len),
            DataType::UnsignedBigint(len) => write!(f, "BIGINT({}) UNSIGNED", len),
            DataType::Tinyint(len) => write!(f, "TINYINT({})", len),
            DataType::UnsignedTinyint(len) => write!(f, "TINYINT({}) UNSIGNED", len),
            DataType::Blob => write!(f, "BLOB"),
            DataType::Longblob => write!(f, "LONGBLOB"),
            DataType::Mediumblob => write!(f, "MEDIUMBLOB"),
            DataType::Tinyblob => write!(f, "TINYBLOB"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Real => write!(f, "REAL"),
            DataType::Tinytext => write!(f, "TINYTEXT"),
            DataType::Mediumtext => write!(f, "MEDIUMTEXT"),
            DataType::Longtext => write!(f, "LONGTEXT"),
            DataType::Text => write!(f, "TEXT"),
            DataType::Json => write!(f, "JSON"),
            DataType::Date => write!(f, "DATE"),
            DataType::DateTime(len) => write!(f, "DATETIME({})", len),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::Binary(len) => write!(f, "BINARY({})", len),
            DataType::Varbinary(len) => write!(f, "VARBINARY({})", len),
            DataType::Enum(_) => write!(f, "ENUM(...)"),
            DataType::Decimal(m, d) => write!(f, "DECIMAL({}, {})", m, d),
        }
    }
}

impl DataType {
    // A SQL type specifier.
    pub fn type_identifier(i: &str) -> IResult<&str, DataType, VerboseError<&str>> {
        alt((
            Self::type_identifier_first_half,
            Self::type_identifier_second_half,
        ))(i)
    }

    fn type_identifier_first_half(i: &str) -> IResult<&str, DataType, VerboseError<&str>> {
        alt((
            Self::tiny_int,
            Self::big_int,
            Self::sql_int_type,
            map(tag_no_case("bool"), |_| DataType::Bool),
            map(
                tuple((
                    tag_no_case("char"),
                    delim_digit,
                    multispace0,
                    opt(tag_no_case("binary")),
                )),
                |t| DataType::Char(Self::len_as_u16(t.1)),
            ),
            map(preceded(tag_no_case("datetime"), opt(delim_digit)), |fsp| {
                DataType::DateTime(match fsp {
                    Some(fsp) => Self::len_as_u16(fsp),
                    None => 0 as u16,
                })
            }),
            map(tag_no_case("date"), |_| DataType::Date),
            map(
                tuple((tag_no_case("double"), multispace0, Self::opt_signed)),
                |_| DataType::Double,
            ),
            map(
                terminated(
                    preceded(
                        tag_no_case("enum"),
                        delimited(tag("("), Literal::value_list, tag(")")),
                    ),
                    multispace0,
                ),
                |v| DataType::Enum(v),
            ),
            map(
                tuple((
                    tag_no_case("float"),
                    multispace0,
                    opt(precision),
                    multispace0,
                )),
                |_| DataType::Float,
            ),
            map(
                tuple((tag_no_case("real"), multispace0, Self::opt_signed)),
                |_| DataType::Real,
            ),
            map(tag_no_case("text"), |_| DataType::Text),
            map(tag_no_case("json"), |_| DataType::Json),
            map(
                tuple((tag_no_case("timestamp"), opt(delim_digit), multispace0)),
                |_| DataType::Timestamp,
            ),
            map(
                tuple((
                    tag_no_case("varchar"),
                    delim_digit,
                    multispace0,
                    opt(tag_no_case("binary")),
                )),
                |t| DataType::Varchar(Self::len_as_u16(t.1)),
            ),
            Self::decimal_or_numeric,
        ))(i)
    }

    fn type_identifier_second_half(i: &str) -> IResult<&str, DataType, VerboseError<&str>> {
        alt((
            map(
                tuple((tag_no_case("binary"), delim_digit, multispace0)),
                |t| DataType::Binary(Self::len_as_u16(t.1)),
            ),
            map(tag_no_case("blob"), |_| DataType::Blob),
            map(tag_no_case("longblob"), |_| DataType::Longblob),
            map(tag_no_case("mediumblob"), |_| DataType::Mediumblob),
            map(tag_no_case("mediumtext"), |_| DataType::Mediumtext),
            map(tag_no_case("longtext"), |_| DataType::Longtext),
            map(tag_no_case("tinyblob"), |_| DataType::Tinyblob),
            map(tag_no_case("tinytext"), |_| DataType::Tinytext),
            map(
                tuple((tag_no_case("varbinary"), delim_digit, multispace0)),
                |t| DataType::Varbinary(Self::len_as_u16(t.1)),
            ),
        ))(i)
    }

    // TODO: rather than copy paste these functions, should create a function that returns a parser
    // based on the sql int type, just like nom does
    fn tiny_int(i: &str) -> IResult<&str, DataType, VerboseError<&str>> {
        let (remaining_input, (_, _, len, _, signed)) = tuple((
            tag_no_case("tinyint"),
            multispace0,
            opt(delim_digit),
            multispace0,
            Self::opt_signed,
        ))(i)?;

        match signed {
            Some(sign) => {
                if sign.eq_ignore_ascii_case("unsigned") {
                    Ok((
                        remaining_input,
                        DataType::UnsignedTinyint(len.map(|l| Self::len_as_u16(l)).unwrap_or(1)),
                    ))
                } else {
                    Ok((
                        remaining_input,
                        DataType::Tinyint(len.map(|l| Self::len_as_u16(l)).unwrap_or(1)),
                    ))
                }
            }
            None => Ok((
                remaining_input,
                DataType::Tinyint(len.map(|l| Self::len_as_u16(l)).unwrap_or(1)),
            )),
        }
    }

    // TODO: rather than copy paste these functions, should create a function that returns a parser
    // based on the sql int type, just like nom does
    fn big_int(i: &str) -> IResult<&str, DataType, VerboseError<&str>> {
        let (remaining_input, (_, _, len, _, signed)) = tuple((
            tag_no_case("bigint"),
            multispace0,
            opt(delim_digit),
            multispace0,
            Self::opt_signed,
        ))(i)?;

        match signed {
            Some(sign) => {
                if sign.eq_ignore_ascii_case("unsigned") {
                    Ok((
                        remaining_input,
                        DataType::UnsignedBigint(len.map(|l| Self::len_as_u16(l)).unwrap_or(1)),
                    ))
                } else {
                    Ok((
                        remaining_input,
                        DataType::Bigint(len.map(|l| Self::len_as_u16(l)).unwrap_or(1)),
                    ))
                }
            }
            None => Ok((
                remaining_input,
                DataType::Bigint(len.map(|l| Self::len_as_u16(l)).unwrap_or(1)),
            )),
        }
    }

    // TODO: rather than copy paste these functions, should create a function that returns a parser
    // based on the sql int type, just like nom does
    fn sql_int_type(i: &str) -> IResult<&str, DataType, VerboseError<&str>> {
        let (remaining_input, (_, _, len, _, signed)) = tuple((
            alt((
                tag_no_case("integer"),
                tag_no_case("int"),
                tag_no_case("smallint"),
            )),
            multispace0,
            opt(delim_digit),
            multispace0,
            Self::opt_signed,
        ))(i)?;

        match signed {
            Some(sign) => {
                if sign.eq_ignore_ascii_case("unsigned") {
                    Ok((
                        remaining_input,
                        DataType::UnsignedInt(len.map(|l| Self::len_as_u16(l)).unwrap_or(32)),
                    ))
                } else {
                    Ok((
                        remaining_input,
                        DataType::Int(len.map(|l| Self::len_as_u16(l)).unwrap_or(32)),
                    ))
                }
            }
            None => Ok((
                remaining_input,
                DataType::Int(len.map(|l| Self::len_as_u16(l)).unwrap_or(32)),
            )),
        }
    }

    // TODO(malte): not strictly ok to treat DECIMAL and NUMERIC as identical; the
    // former has "at least" M precision, the latter "exactly".
    // See https://dev.mysql.com/doc/refman/5.7/en/precision-math-decimal-characteristics.html
    fn decimal_or_numeric(i: &str) -> IResult<&str, DataType, VerboseError<&str>> {
        let (remaining_input, precision) = delimited(
            alt((tag_no_case("decimal"), tag_no_case("numeric"))),
            opt(precision),
            multispace0,
        )(i)?;

        match precision {
            None => Ok((remaining_input, DataType::Decimal(32, 0))),
            Some((m, None)) => Ok((remaining_input, DataType::Decimal(m, 0))),
            Some((m, Some(d))) => Ok((remaining_input, DataType::Decimal(m, d))),
        }
    }

    fn opt_signed(i: &str) -> IResult<&str, Option<&str>, VerboseError<&str>> {
        opt(alt((tag_no_case("unsigned"), tag_no_case("signed"))))(i)
    }

    #[inline]
    fn len_as_u16(len: &str) -> u16 {
        match u16::from_str(len) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

#[cfg(test)]
mod test {
    use base::DataType;

    #[test]
    fn sql_types() {
        let ok = ["bool", "integer(16)", "datetime(16)"];
        let not_ok = ["varchar"];

        let res_ok: Vec<_> = ok
            .iter()
            .map(|t| DataType::type_identifier(t).unwrap().1)
            .collect();
        let res_not_ok: Vec<_> = not_ok
            .iter()
            .map(|t| DataType::type_identifier(t).is_ok())
            .collect();

        assert_eq!(
            res_ok,
            vec![DataType::Bool, DataType::Int(16), DataType::DateTime(16)]
        );

        assert!(res_not_ok.into_iter().all(|r| r == false));
    }
}