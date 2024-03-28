extern crate nom;

use nom::{
    bytes::complete::tag_no_case,
    error::{context, VerboseError},
    sequence::tuple,
    IResult,
};

// 使用VerboseError来获得更详细的错误信息
fn parse_add_demo(input: &str) -> IResult<&str, (&str, &str), VerboseError<&str>> {
    // 使用tuple来组合解析器，并为每个步骤提供上下文
    tuple((
        context("expect ADD", tag_no_case("ADD")),
        context("expect DEMO", tag_no_case("DEMO")),
    ))(input)
}

fn main() {
    let input = "ADD ABC";
    match parse_add_demo(input) {
        Ok((remaining, value)) => println!(
            "Parsed successfully: {:?}, Remaining: '{}'",
            value, remaining
        ),
        Err(err) => match err {
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                //println!("{e}");
                // println!("=====");
                println!("Error: {}", nom::error::convert_error(input, e));
            }
            _ => println!("Parsing failed in an unexpected way."),
        },
    }
}
