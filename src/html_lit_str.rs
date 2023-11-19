use std::borrow::Cow;
use dparse::{Parse, ParseError, Parser, Span};

#[derive(Debug)]
pub struct HtmlStrLit<'a> {
    content: Cow<'a, str>,
    span: Span,
}

impl HtmlStrLit<'_> {
    #[inline]
    pub fn content(&self) -> &str {
        self.content.as_ref()
    }
}

impl PartialEq<str> for HtmlStrLit<'_> {
    fn eq(&self, other: &str) -> bool {
        self.content == other
    }
}

impl<'a> PartialEq<&'a str> for HtmlStrLit<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        self.content == *other
    }
}

impl<'a> Parse<'a> for HtmlStrLit<'a> {
    fn parse<P: Parser<'a> + ?Sized>(parser: &mut P) -> Result<Self, ParseError> {
        let start = parser.location();
        if !parser.take_char('"') {
            return Err(parser.mismatch("expected '\"'"));
        }
        let _whites = parser.preserve_whitespace();

        let mut loc = parser.location();
        let mut content = Cow::Borrowed("");
        while let Some(c) = parser.peek() {
            if c == '"' {
                break;
            }

            let start = parser.location();
            parser.consume();

            if c != '&' {
                continue;
            }

            let fill = if parser.take_str("amp;") {
                '&'
            } else if parser.take_str("lt;") {
                '<'
            } else if parser.take_str("gt;") {
                '>'
            } else if parser.take_str("#39;") {
                '&'
            } else if parser.take_str("quot;") {
                '"'
            } else {
                let span = start.span(parser.location()).unwrap();
                return Err(ParseError::new(
                    span,
                    format!("unknown escape sequence: &amp; or &lt; or &gt; or &#39; or &quot;"),
                ));
            };

            let mut str = match content {
                Cow::Owned(str) => str,
                Cow::Borrowed(_) => {
                    let span = loc.span(start).unwrap();
                    loc = parser.location();
                    parser.source_for_span(span).unwrap().to_string()
                }
            };
            str.push(fill);
            content = Cow::Owned(str);
        }

        let span = loc.span(parser.location()).unwrap();
        let rest = parser.source_for_span(span).unwrap();
        let content = match content {
            Cow::Borrowed(_) => Cow::Borrowed(rest), // rest is all
            Cow::Owned(mut str) => {
                str.push_str(rest);
                Cow::Owned(str)
            }
        };

        if !parser.take_char('"') {
            return Err(parser.error("expected '\"'"));
        }

        let span = start.span(parser.location()).unwrap();

        Ok(Self { content, span })
    }
}

#[derive(Debug)]
pub struct HtmlRawStrLit<'a> {
    content: Cow<'a, str>,
    span: Span,
}

impl HtmlRawStrLit<'_> {
    #[inline]
    pub fn content(&self) -> &str {
        self.content.as_ref()
    }
}

impl PartialEq<str> for HtmlRawStrLit<'_> {
    fn eq(&self, other: &str) -> bool {
        self.content == other
    }
}

impl<'a> PartialEq<&'a str> for HtmlRawStrLit<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        self.content == *other
    }
}

impl<'a> Parse<'a> for HtmlRawStrLit<'a> {
    fn parse<P: Parser<'a> + ?Sized>(parser: &mut P) -> Result<Self, ParseError> {
        let start = parser.location();
        if !parser.take_str("\"") {
            return Err(parser.mismatch("expected 'r#\"'"));
        }
        let _whites = parser.preserve_whitespace();

        let inner_start = parser.location();
        while parser.more() && !parser.match_char('"') {
            parser.consume();
        }
        let inner_span = inner_start.span(parser.location()).unwrap();
        let content = parser.source_for_span(inner_span).unwrap();
        let content = Cow::Borrowed(content);

        if !parser.take_str("\"") {
            return Err(parser.error("expected '\"'"));
        }

        let span = start.span(parser.location()).unwrap();

        Ok(Self { content, span })
    }
}


#[cfg(test)]
mod tests {
    use dparse::BasicParser;
    use super::*;

    #[test]
    fn test() {
        let mut parser = BasicParser::new(r#" "4 &lt; 5"  "#);
        let attr = HtmlStrLit::parse(&mut parser);
        let attr = attr
            .map_err(|e| {
                eprintln!("{}", e);
            })
            .unwrap();
        assert_eq!(attr.content, "4 < 5");
    }

    #[test]
    fn test_raw() {
        let mut parser = BasicParser::new(r#" "4 < 5" "#);
        let attr = HtmlRawStrLit::parse(&mut parser);
        let attr = attr
            .map_err(|e| {
                eprintln!("{}", e);
            })
            .unwrap();
        assert_eq!(attr.content, "4 < 5");
    }
}
