use std::borrow::Cow;
use dparse::basic_tokens::RightAngle;
use dparse::{Parse, ParseError, ParseErrorWithContext, Parser};
use crate::html_ident::HtmlIdent;
use crate::opening_tag::OpeningTag;
use crate::token::LeftAngleSlash;

#[derive(Debug)]
pub struct Tag<'a> {
    pub opening_tag: OpeningTag<'a>,
    pub content: Vec<TagContent<'a>>,
    pub closing_tag: Option<ClosingTag<'a>>,
}

impl<'src> Parse<'src> for Tag<'src> {
    fn parse<P: Parser<'src> + ?Sized>(parser: &mut P) -> Result<Self, ParseError> {
        let opening_tag = parser.parse::<OpeningTag>()
            .context(parser.span(), "expected opening tag")?;

        let mut closing_tag = None;
        let mut content = Vec::new();
        loop {
            let start = {
                let _ws = parser.preserve_whitespace();
                parser.location() // this should be the start of the content
            };
            let location = parser.location();
            if let Some(tag) = parser.try_parse::<ClosingTag>()? {
                if opening_tag.name.content() == tag.name.content() {
                    closing_tag = Some(tag);
                } else {
                    parser.set_location(location)
                        .expect("Parser::set_location: failed");
                }

                break;
            }

            parser.set_location(start).expect("Parser::set_location: failed");
            let _ws = parser.preserve_whitespace();
            if parser.match_char('<') {
                let tag = parser.parse::<Tag>()
                    .context(parser.span(), "expected tag")?;
                content.push(TagContent::Tag(tag));
            } else {
                let Some(text) = parser.take_while(|c| c != '<') else {
                    break;
                };
                content.push(TagContent::Text(Cow::Borrowed(text)));
            }
        }

        Ok(Self {
            opening_tag,
            content,
            closing_tag,
        })
    }
}

#[derive(Debug)]
pub enum TagContent<'a> {
    Text(Cow<'a, str>),
    Tag(Tag<'a>),
}

impl<'src> Parse<'src> for TagContent<'src> {
    fn parse<P: Parser<'src> + ?Sized>(parser: &mut P) -> Result<Self, ParseError> {
        let _ws = parser.preserve_whitespace();

        if parser.match_char('<') {
            let tag = parser.parse::<Tag>()
                .context(parser.span(), "expected tag")?;
            Ok(Self::Tag(tag))
        } else {
            let text = parser.take_while(|c| c != '<')
                .expect("Parser::consume_while: failed");
            Ok(Self::Text(Cow::Borrowed(text)))
        }
    }
}

#[derive(Debug)]
pub struct ClosingTag<'a> {
    pub opening_tag: LeftAngleSlash,
    pub name: HtmlIdent<'a>,
    pub closing_tag: RightAngle,
}

impl<'a> Parse<'a> for ClosingTag<'a> {
    fn parse<P: Parser<'a> + ?Sized>(parser: &mut P) -> Result<Self, ParseError> {
        let opening_tag = parser.parse::<LeftAngleSlash>()
            .context(parser.span(), "expected </ for closing tag")?;
        let name = parser.require::<HtmlIdent>()
            .context(parser.span(), "expected tag name")?;
        let closing_tag = parser.require::<RightAngle>()
            .context(parser.span(), "expected > for closing tag")?;

        Ok(Self {
            opening_tag,
            name,
            closing_tag,
        })
    }
}

#[cfg(test)]
mod test {
    use dparse::{BasicParser, Parse};
    use super::*;

    #[test]
    fn closing_tag() {
        let mut parser = BasicParser::new("</div>");
        let closing_tag = ClosingTag::parse(&mut parser).unwrap();
        assert_eq!(closing_tag.name, "div");

        let mut parser = BasicParser::new("</div >");
        let closing_tag = ClosingTag::parse(&mut parser).unwrap();
        assert_eq!(closing_tag.name, "div");

        let mut parser = BasicParser::new("</div>");
        let closing_tag = ClosingTag::parse(&mut parser).unwrap();
        assert_eq!(closing_tag.name, "div");
    }

    #[test]
    fn test_simple() {
        let mut parser = BasicParser::new("<div></div>");
        let tag = Tag::parse(&mut parser).unwrap();
        assert_eq!(tag.opening_tag.name, "div");
        assert_eq!(tag.closing_tag.unwrap().name, "div");
        assert_eq!(tag.content.len(), 0);
    }

    #[test]
    fn test_simple_content() {
        let mut parser = BasicParser::new("<div>Molkars wuz here</div>");
        let tag = Tag::parse(&mut parser).unwrap();
        assert_eq!(tag.opening_tag.name, "div");
        assert_eq!(tag.closing_tag.unwrap().name, "div");
        assert_eq!(tag.content.len(), 1);
        let TagContent::Text(text) = &tag.content[0] else {
            panic!("expected text");
        };

        assert_eq!(text, "Molkars wuz here");
    }

    #[test]
    fn test_implicit() {
        let mut parser = BasicParser::new("<div><p>Hi There!</div>");
        let tag = Tag::parse(&mut parser).unwrap();
        println!("{:#?}", tag);
        assert_eq!(tag.opening_tag.name, "div");
        assert_eq!(tag.closing_tag.unwrap().name, "div");
        assert_eq!(tag.content.len(), 1);
        let TagContent::Tag(tag) = &tag.content[0] else {
            panic!("expected tag");
        };
        assert_eq!(tag.opening_tag.name, "p");
        assert!(tag.closing_tag.is_none());
        assert_eq!(tag.content.len(), 1);
        let TagContent::Text(text) = &tag.content[0] else {
            panic!("expected text");
        };
        assert_eq!(text, "Hi There!");
    }
}