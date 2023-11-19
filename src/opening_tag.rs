use dparse::{Parse, ParseError, ParseErrorWithContext, Parser};
use dparse::basic_tokens::{LeftAngle, RightAngle};
use dparse_derive::Parse;
use crate::attribute::Attribute;
use crate::html_ident::HtmlIdent;
use crate::token::{SlashRightAngle};

#[derive(Debug)]
pub struct OpeningTag<'a> {
    pub opening_tag: LeftAngle,
    pub name: HtmlIdent<'a>,
    pub attributes: Vec<Attribute<'a>>,
    pub terminator: OpeningTagTerminator,
}

#[derive(Parse, Debug)]
pub enum OpeningTagTerminator {
    Closed(SlashRightAngle),
    Open(RightAngle),
}

impl<'a> Parse<'a> for OpeningTag<'a> {
    fn parse<P: Parser<'a> + ?Sized>(parser: &mut P) -> Result<Self, ParseError> {
        let opening_tag = parser.parse::<LeftAngle>()
            .context(parser.span(), "expected opening tag")?;
        let name = parser.require::<HtmlIdent>()
            .context(parser.span(), "expected tag name")?;

        let mut attributes = Vec::new();
        while let Some(attr) = parser.try_parse::<Attribute>()
            .context(parser.span(), "expected attribute")? {
            attributes.push(attr);
        }

        let terminator = parser.parse::<OpeningTagTerminator>()
            .context(parser.span(), "expected tag terminator")?;

        Ok(Self {
            opening_tag,
            name,
            attributes,
            terminator,
        })
    }
}

#[cfg(test)]
mod tests {
    use dparse::{BasicParser, Parse};
    use super::*;

    #[test]
    fn test_terminator() {
        let mut parser = BasicParser::new(r#" /> "#);
        let terminator = OpeningTagTerminator::parse(&mut parser).unwrap();
        assert!(matches!(terminator, OpeningTagTerminator::Closed(_)));

        let mut parser = BasicParser::new(r#" > "#);
        let terminator = OpeningTagTerminator::parse(&mut parser).unwrap();
        assert!(matches!(terminator, OpeningTagTerminator::Open(_)));
    }

    fn parse(src: &str) -> OpeningTag {
        let mut parser = BasicParser::new(src);
        parser.parse::<OpeningTag>()
            .map_err(|e| {
                eprintln!("{}", e);
            })
            .unwrap()
    }

    #[test]
    fn test_simple() {
        let tag = parse(r#"<div>"#);
        assert_eq!(tag.name, "div");
        assert_eq!(tag.attributes.len(), 0);
        assert!(matches!(tag.terminator, OpeningTagTerminator::Open(_)));
    }

    #[test]
    fn test_attributes() {
        let tag = parse(r#"<div id="foo" class="bar">"#);
        assert_eq!(tag.name, "div");

        assert_eq!(tag.attributes.len(), 2);

        assert_eq!(tag.attributes[0].name, "id");
        assert_eq!(tag.attributes[0].value.content(), "foo");

        assert_eq!(tag.attributes[1].name, "class");
        assert_eq!(tag.attributes[1].value.content(), "bar");

        assert!(matches!(tag.terminator, OpeningTagTerminator::Open(_)));
    }
}
