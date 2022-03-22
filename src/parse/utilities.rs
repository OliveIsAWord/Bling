//! Useful parsing methods that are not specific to Bling grammar.

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::{cut, map, recognize},
    error::ParseError,
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

pub fn trim_ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn trim_left_ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    preceded(multispace0, inner)
}

pub fn trim_right_ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    terminated(inner, multispace0)
}

pub fn ident(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            cut(many0(alt((alphanumeric1, tag("_"))))),
        )),
        |out: &str| out.to_owned(),
    )(input)
}

pub fn paren_args<'a, F: 'a, I: 'a>(inner: F) -> impl FnMut(&'a str) -> IResult<&str, Vec<I>>
where
    F: FnMut(&str) -> IResult<&str, I>,
{
    delimited(
        char('('),
        cut(many0(trim_left_ws(inner))),
        cut(trim_left_ws(char(')'))),
    )
}

// // This one is comma delimited
// pub fn paren_args<'a, F: 'a, I: 'a>(inner: F) -> impl FnMut(&'a str) -> IResult<&str, Vec<I>>
// where
//     F: FnMut(&str) -> IResult<&str, I>,
// {
//     delimited(
//         char('('),
//         opt_or_default(trim_left_ws(terminated(
//             separated_list1(trim_ws(char(',')), inner),
//             opt(trim_left_ws(char(','))),
//         ))),
//         trim_left_ws(char(')')),
//     )
// }
