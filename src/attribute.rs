use dparse::{Parse, ParseError, Parser};
use crate::html_ident::HtmlIdent;
use crate::html_lit_str::{HtmlRawStrLit, HtmlStrLit};

#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: HtmlIdent<'a>,
    pub value: AttributeValue<'a>,
}

#[derive(Debug)]
pub enum AttributeValue<'a> {
    HtmlStrLit(HtmlStrLit<'a>),
    HtmlRawStrLit(HtmlRawStrLit<'a>),
}

impl AttributeValue<'_> {
    pub fn content(&self) -> &str {
        match self {
            Self::HtmlStrLit(value) => value.content(),
            Self::HtmlRawStrLit(value) => value.content(),
        }
    }
}

impl PartialEq<str> for AttributeValue<'_> {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::HtmlStrLit(value) => value == other,
            Self::HtmlRawStrLit(value) => value == other,
        }
    }
}

impl<'a> PartialEq<&'a str> for AttributeValue<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        match self {
            Self::HtmlStrLit(value) => value == other,
            Self::HtmlRawStrLit(value) => value == other,
        }
    }
}

impl<'a> Parse<'a> for Attribute<'a> {
    fn parse<P: Parser<'a> + ?Sized>(parser: &mut P) -> Result<Self, ParseError> {
        let Some(name) = parser.try_parse::<HtmlIdent>()? else {
            return Err(parser.mismatch("expected attribute name"));
        };

        if !parser.take_char('=') {
            return Err(parser.error("expected '='"));
        }

        let value = match name.content() {
            "style" => {
                let value = parser.require::<HtmlRawStrLit>()?;
                AttributeValue::HtmlRawStrLit(value)
            }
            _ => {
                let value = parser.require::<HtmlStrLit>()?;
                AttributeValue::HtmlStrLit(value)
            }
        };

        Ok(Self {
            name,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use dparse::BasicParser;
    use super::*;

    fn parse(input: &str) -> Attribute {
        let mut parser = BasicParser::new(input);
        parser.parse::<Attribute>().unwrap()
    }

    #[test]
    fn test_simple() {
        let attr = parse(r#" class="foo" "#);
        assert_eq!(attr.name, "class");
        assert_eq!(attr.value, "foo");
    }

    #[test]
    fn test_style() {
        let attr = parse(r#" style="color: red; details > p { color: blue; }" "#);
        assert_eq!(attr.name, "style");
        assert_eq!(attr.value, r#"color: red; details > p { color: blue; }"#);
    }
}