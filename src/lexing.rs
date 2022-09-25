use nom::IResult;
use nom::bytes::complete::tag;
use nom_locate::{position, LocatedSpan};

type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub position: Span<'a>,
}

#[derive(Debug)]
pub enum TokenKind {
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
}

pub fn lex(source: &str) -> Option<Vec<Token>> {
    let tokens = vec![];
    let mut buffer = source;
    //while let Some(text) = buffer.trim_
    Some(tokens)
}

// pub fn one_token(src: &str) -> IResult<&str, Token> {
//
// }

pub fn open_paren(s: &str) -> IResult<&str, Token> {
    tag("(")(s).map(|_| todo!())
}
