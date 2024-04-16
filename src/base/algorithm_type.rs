use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{map, opt};
use nom::sequence::tuple;
use nom::IResult;
use std::fmt::{write, Display, Formatter};

use base::ParseSQLError;

/// parse `ALGORITHM [=] {DEFAULT | INSTANT | INPLACE | COPY}`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AlgorithmType {
    Instant, // alter table only
    Default,
    Inplace,
    Copy,
}

impl AlgorithmType {
    /// algorithm_option:
    ///     ALGORITHM [=] {DEFAULT | INSTANT | INPLACE | COPY}
    pub fn parse(i: &str) -> IResult<&str, AlgorithmType, ParseSQLError<&str>> {
        alt((
            map(
                tuple((tag_no_case("ALGORITHM"), multispace1, Self::parse_algorithm)),
                |(_, _, algorithm)| algorithm,
            ),
            map(
                tuple((
                    tag_no_case("ALGORITHM"),
                    multispace0,
                    tag("="),
                    multispace0,
                    Self::parse_algorithm,
                )),
                |(_, _, _, _, algorithm)| algorithm,
            ),
        ))(i)
    }

    fn parse_algorithm(i: &str) -> IResult<&str, AlgorithmType, ParseSQLError<&str>> {
        alt((
            map(tag_no_case("DEFAULT"), |_| AlgorithmType::Default),
            map(tag_no_case("INSTANT"), |_| AlgorithmType::Instant),
            map(tag_no_case("INPLACE"), |_| AlgorithmType::Inplace),
            map(tag_no_case("COPY"), |_| AlgorithmType::Copy),
        ))(i)
    }
}

impl Display for AlgorithmType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            AlgorithmType::Instant => write!(f, "ALGORITHM INSTANT"),
            AlgorithmType::Default => write!(f, "ALGORITHM DEFAULT"),
            AlgorithmType::Inplace => write!(f, "ALGORITHM INPLACE"),
            AlgorithmType::Copy => write!(f, "ALGORITHM COPY"),
        }
    }
}

#[cfg(test)]
mod tests {
    use base::algorithm_type::AlgorithmType;

    #[test]
    fn parse_algorithm_type() {
        let str1 = "ALGORITHM INSTANT";
        let res1 = AlgorithmType::parse(str1);
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap().1, AlgorithmType::Instant);

        let str2 = "ALGORITHM=DEFAULT";
        let res2 = AlgorithmType::parse(str2);
        assert!(res2.is_ok());
        assert_eq!(res2.unwrap().1, AlgorithmType::Default);

        let str3 = "ALGORITHM= INPLACE";
        let res3 = AlgorithmType::parse(str3);
        assert!(res3.is_ok());
        assert_eq!(res3.unwrap().1, AlgorithmType::Inplace);

        let str4 = "ALGORITHM =COPY";
        let res4 = AlgorithmType::parse(str4);
        assert!(res4.is_ok());
        assert_eq!(res4.unwrap().1, AlgorithmType::Copy);

        let str5 = "ALGORITHM = DEFAULT";
        let res5 = AlgorithmType::parse(str5);
        assert!(res5.is_ok());
        assert_eq!(res5.unwrap().1, AlgorithmType::Default);

        let str6 = "ALGORITHMDEFAULT";
        let res6 = AlgorithmType::parse(str6);
        assert!(res6.is_err());
    }
}
