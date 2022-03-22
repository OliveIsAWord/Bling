//! Creates an AST from Bling source code.

mod utilities;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{all_consuming, cut, map, map_res, not, opt, recognize},
    multi::{many0, many1},
    sequence::{delimited, pair, separated_pair, terminated},
    Finish, IResult,
};

use utilities::{ident, paren_args, trim_left_ws, trim_right_ws, trim_ws};

/// Internal representation of a variable identifier.
pub type Ident = String;

/// Every type of node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// An integer literal.
    Number(i64),
    /// A variable name of the form `[a-zA-Z_][a-zA-Z_0-9]*`.
    Identifier(Ident),
    /// An expression being assigned to a variable.
    Assignment(Ident, Box<Expr>),
    /// A variable being declared and initialized with the value of an expression.
    Declaration(Ident, Box<Expr>),
    /// A list of expressions within curly brackets `{}`.
    Block(Vec<Expr>),
    /// A function call.
    Application(Box<Expr>, Vec<Expr>),
    /// A lambda function definition.
    Lambda(Vec<Ident>, Box<Expr>),
}

fn number(input: &str) -> IResult<&str, Expr> {
    map_res(
        recognize(pair(
            opt(char('-')),
            many1(terminated(digit1, many0(char('_')))),
        )),
        |out: &str| out.replace('_', "").parse().map(Expr::Number),
    )(input)
}

// fn string(input: &str) -> IResult<&str, Expr> {
//     delimited(char('"'), , char('"'))
// }

fn identifier(input: &str) -> IResult<&str, Expr> {
    map(ident, Expr::Identifier)(input)
}

#[doc(hidden)]
macro_rules! assign_parse {
    ($name:ident, $variant:ident, $symbol:expr) => {
        fn $name(input: &str) -> IResult<&str, Expr> {
            map(
                separated_pair(ident, trim_ws(tag($symbol)), expr),
                |(lhs, rhs)| Expr::$variant(lhs, Box::new(rhs)),
            )(input)
        }
    };
}

assign_parse! {assignment, Assignment, "="}
assign_parse! {declaration, Declaration, ":="}

fn block(input: &str) -> IResult<&str, Expr> {
    map(
        delimited(
            char('{'),
            cut(many0(trim_left_ws(expr))),
            cut(trim_left_ws(char('}'))),
        ),
        Expr::Block,
    )(input)
}

fn application(input: &str) -> IResult<&str, Expr> {
    map(
        pair(
            alt((identifier, block)),
            many1(terminated(
                trim_left_ws(paren_args(expr)),
                not(trim_left_ws(tag("=>"))), // Prevent lambda params from being parsed as calls
            )),
        ),
        |(ident, calls)| {
            calls
                .into_iter()
                .fold(ident, |func, args| Expr::Application(Box::new(func), args))
        },
    )(input)
}

fn lambda(input: &str) -> IResult<&str, Expr> {
    map(
        separated_pair(paren_args(ident), cut(trim_left_ws(tag("=>"))), cut(expr)),
        |(params, body)| Expr::Lambda(params, Box::new(body)),
    )(input)
}

fn expr(input: &str) -> IResult<&str, Expr> {
    trim_left_ws(alt((
        number,
        lambda,
        application,
        block,
        assignment,
        declaration,
        identifier,
    )))(input)
}

pub fn parse(input: &str) -> Result<Vec<Expr>, nom::error::Error<&str>> {
    all_consuming(trim_right_ws(many0(expr)))(input)
        .finish()
        .map(|x| x.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_application() {
        use Expr::*;
        let source = "func(42)";
        assert_eq!(
            expr(source).unwrap().1,
            Application(Box::new(Identifier("func".to_owned())), vec![Number(42)])
        );
    }

    #[test]
    fn multiple_application() {
        use Expr::*;
        let source = "func(42)(555)";
        assert_eq!(
            expr(source).unwrap().1,
            Application(
                Box::new(Application(
                    Box::new(Identifier("func".to_owned())),
                    vec![Number(42)]
                )),
                vec![Number(555)]
            )
        );
    }
}
