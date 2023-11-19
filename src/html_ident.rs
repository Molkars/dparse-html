use std::borrow::Cow;
use dparse::{Parse, ParseError, Parser, Span};

#[derive(Debug)]
pub struct HtmlIdent<'a> {
    content: Cow<'a, str>,
    span: Span,
}

impl PartialEq<str> for HtmlIdent<'_> {
    fn eq(&self, other: &str) -> bool {
        self.content == other
    }
}

impl<'a> PartialEq<&'a str> for HtmlIdent<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        self.content == *other
    }
}

impl<'a> HtmlIdent<'a> {
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl<'a> Parse<'a> for HtmlIdent<'a> {
    fn parse<P: Parser<'a> + ?Sized>(parser: &mut P) -> Result<Self, ParseError> {
        if parser.peek().filter(|c| c.is_ascii_alphabetic()).is_none() {
            return Err(parser.mismatch("expected attribute name"));
        }

        let start = parser.location();
        let _whites = parser.preserve_whitespace();

        let name = parser.take_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            .expect("Parser::take_while failed");
        let name_span = start.span(parser.location())
            .expect("Parser::span failed");

        Ok(Self {
            content: Cow::Borrowed(name),
            span: name_span,
        })
    }
}