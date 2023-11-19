use dparse::token;

token! {
    #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
    pub struct SlashRightAngle("/>");
    #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
    pub struct LeftAngleSlash("</");
}

#[cfg(test)]
mod tests {
    use dparse::{Parse, Parser, BasicParser};
    use super::*;

    fn parse<'a, T>(input: &'a str) -> T
    where
        T: Parse<'a>,
    {
        let mut parser = BasicParser::new(input);
        parser.parse::<T>().unwrap()
    }

    #[test]
    fn test_slash_right_angle() {
        let _: SlashRightAngle = parse(r#"/>"#);
        let _: SlashRightAngle = parse(r#" />"#);
        let _: SlashRightAngle = parse(r#"  />"#);
    }

    #[test]
    fn test_right_angle() {
        let _: LeftAngleSlash = parse(r#"</"#);
        let _: LeftAngleSlash = parse(r#" </"#);
        let _: LeftAngleSlash = parse(r#"  </"#);
    }
}