use std::str::FromStr;
use tracing::{instrument, trace};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Pattern {
    ExactChar(char),
    AnyChar,
    AlphaNumeric,
    Sequence(Vec<Pattern>),
    Repeated {
        min: usize,
        max: Option<usize>,
        pattern: Box<Pattern>,
    },
    OneOf(Vec<Pattern>),
    CharacterSet {
        chars: String,
        negated: bool,
    },
    StartOfLine,
    EndOfLine,
    OneOrMore(Box<Pattern>),
    ZeroOrOne(Box<Pattern>),
    Alternation(Vec<Pattern>),
    Backreference(usize),
}

impl FromStr for Pattern {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_group(s: &str, chars: &mut std::str::Chars) -> Result<Pattern, String> {
            let mut alternatives = vec![];
            let mut current = vec![];

            while let Some(c) = chars.next() {
                match c {
                    '(' => {
                        current.push(parse_group(s, chars)?);
                    }
                    ')' => {
                        if !current.is_empty() {
                            alternatives.push(Pattern::Sequence(current));
                        }
                        return if alternatives.len() == 1 {
                            Ok(alternatives.pop().unwrap())
                        } else {
                            Ok(Pattern::Alternation(alternatives))
                        };
                    }
                    '|' => {
                        if !current.is_empty() {
                            alternatives.push(Pattern::Sequence(current));
                            current = vec![];
                        }
                    }
                    '\\' => match chars.next() {
                        Some('w') => current.push(Pattern::AlphaNumeric),
                        Some('d') => current.push(Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false }),
                        Some(c) if c.is_digit(10) => {
                            let backreference = c.to_digit(10).unwrap() as usize;
                            current.push(Pattern::Backreference(backreference));
                        },
                        Some(c) => current.push(Pattern::ExactChar(c)),
                        None => return Err(format!("Unterminated escape in {:?}", s)),
                    },
                    '.' => current.push(Pattern::AnyChar),
                    '*' => {
                        match current.pop() {
                            Some(p) => current.push(Pattern::Repeated {
                                min: 0,
                                max: None,
                                pattern: Box::new(p),
                            }),
                            None => return Err("Invalid repeat".into()),
                        }
                    }
                    '[' => {
                        let mut chars_set = String::new();
                        let mut found_end = false;
                        let mut negated = false;
                        while let Some(c2) = chars.next() {
                            match c2 {
                                '^' if chars_set.is_empty() => negated = true,
                                ']' => {
                                    found_end = true;
                                    break;
                                }
                                other => chars_set.push(other),
                            }
                        }
                        if !found_end {
                            return Err("Unterminated '[' pattern".into());
                        }
                        current.push(Pattern::CharacterSet { chars: chars_set, negated });
                    }
                    '^' if current.is_empty() => current.push(Pattern::StartOfLine),
                    '$' if chars.clone().next().is_none() => current.push(Pattern::EndOfLine),
                    '+' => {
                        match current.pop() {
                            Some(p) => current.push(Pattern::OneOrMore(Box::new(p))),
                            None => return Err("Invalid '+' quantifier".into()),
                        }
                    }
                    '?' => {
                        match current.pop() {
                            Some(p) => current.push(Pattern::ZeroOrOne(Box::new(p))),
                            None => return Err("Invalid '?' quantifier".into()),
                        }
                    }
                    e => current.push(Pattern::ExactChar(e)),
                }
            }

            if !current.is_empty() {
                alternatives.push(Pattern::Sequence(current));
            }

            if alternatives.len() == 1 {
                Ok(alternatives.pop().unwrap())
            } else {
                Ok(Pattern::Alternation(alternatives))
            }
        }

        parse_group(s, &mut s.chars())
    }
}

impl Pattern {
    #[instrument]
    pub fn match_str(&self, data: &str) -> bool {
        trace!("Matching starts");
        let mut captured_groups = Vec::new();
        self.match_str_with_captures(data, &mut captured_groups)
    }

    fn match_str_with_captures(&self, data: &str, captured_groups: &mut Vec<String>) -> bool {
        match self {
            Pattern::StartOfLine => self.match_from_start(data, captured_groups),
            Pattern::EndOfLine => data.is_empty(),
            Pattern::Sequence(patterns) => {
                if patterns.first() == Some(&Pattern::StartOfLine) && patterns.last() == Some(&Pattern::EndOfLine) {
                    // Both start and end anchors
                    let without_anchors = Pattern::Sequence(patterns[1..patterns.len()-1].to_vec());
                    without_anchors.match_from_start(data, captured_groups) && data.len() == without_anchors.match_length(data, captured_groups)
                } else if patterns.first() == Some(&Pattern::StartOfLine) {
                    // Only start anchor
                    self.match_from_start(data, captured_groups)
                } else if patterns.last() == Some(&Pattern::EndOfLine) {
                    // Only end anchor
                    let without_end = Pattern::Sequence(patterns[..patterns.len() - 1].to_vec());
                    without_end.match_from_start(data, captured_groups) && data.len() == without_end.match_length(data, captured_groups)
                } else {
                    // No anchors
                    (0..=data.len()).any(|i| self.match_from_start(&data[i..], captured_groups))
                }
            }
            _ => (0..=data.len()).any(|i| self.match_from_start(&data[i..], captured_groups))
        }
    }

    fn match_from_start(&self, data: &str, captured_groups: &mut Vec<String>) -> bool {
        match self {
            Pattern::ExactChar(c) => data.starts_with(*c),
            Pattern::AnyChar => !data.is_empty(),
            Pattern::AlphaNumeric => data.chars().next().map_or(false, |c| c.is_alphanumeric() || c == '_'),
            Pattern::Sequence(sub_patterns) => {
                let mut remaining = data;
                let start_len = captured_groups.len();
                for sub_pattern in sub_patterns {
                    if let Some(new_remaining) = sub_pattern.consume_match(remaining, captured_groups) {
                        remaining = new_remaining;
                    } else {
                        captured_groups.truncate(start_len);
                        return false;
                    }
                }
                if !remaining.is_empty() {
                    captured_groups.push(data[..data.len() - remaining.len()].to_string());
                }
                true
            },
            Pattern::CharacterSet { chars, negated } => {
                data.chars().next().map_or(false, |c| chars.contains(c) != *negated)
            },
            Pattern::StartOfLine => true,
            Pattern::OneOf(sub_patterns) => sub_patterns.iter().any(|p| p.match_from_start(data, captured_groups)),
            Pattern::Repeated { min, max, pattern } => {
                let mut count = 0;
                let mut remaining = data;
                while max.map_or(true, |m| count < m) {
                    if let Some(new_remaining) = pattern.consume_match(remaining, captured_groups) {
                        remaining = new_remaining;
                        count += 1;
                    } else {
                        break;
                    }
                }
                count >= *min
            },
            Pattern::EndOfLine => data.is_empty(),
            Pattern::OneOrMore(pattern) => {
                let mut remaining = data;
                let mut matched = false;
                while let Some(new_remaining) = pattern.consume_match(remaining, captured_groups) {
                    remaining = new_remaining;
                    matched = true;
                }
                matched
            },
            Pattern::ZeroOrOne(pattern) => {
                pattern.consume_match(data, captured_groups).is_some() || true
            },
            Pattern::Alternation(patterns) => patterns.iter().any(|p| p.match_from_start(data, captured_groups)),
            Pattern::Backreference(n) => {
                if let Some(group) = captured_groups.get(*n - 1) {
                    data.starts_with(group)
                } else {
                    false
                }
            },
        }
    }

    fn consume_match<'a>(&self, data: &'a str, captured_groups: &mut Vec<String>) -> Option<&'a str> {
        if self.match_from_start(data, captured_groups) {
            Some(&data[self.match_length(data, captured_groups)..])
        } else {
            None
        }
    }

    fn match_length(&self, data: &str, captured_groups: &mut Vec<String>) -> usize {
        match self {
            Pattern::ExactChar(_) => 1,
            Pattern::AnyChar => 1,
            Pattern::AlphaNumeric => 1,
            Pattern::Sequence(sub_patterns) => {
                let mut length = 0;
                let mut remaining = data;
                for sub_pattern in sub_patterns {
                    if let Some(new_remaining) = sub_pattern.consume_match(remaining, captured_groups) {
                        length += remaining.len() - new_remaining.len();
                        remaining = new_remaining;
                    } else {
                        break;
                    }
                }
                length
            },
            Pattern::CharacterSet { .. } => 1,
            Pattern::StartOfLine => 0,
            Pattern::OneOf(sub_patterns) => sub_patterns
                .iter()
                .filter_map(|p| p.consume_match(data, captured_groups).map(|r| data.len() - r.len()))
                .next()
                .unwrap_or(0),
            Pattern::Repeated { min: _, max, pattern } => {
                let mut count = 0;
                let mut length = 0;
                let mut remaining = data;
                while max.map_or(true, |m| count < m) {
                    if let Some(new_remaining) = pattern.consume_match(remaining, captured_groups) {
                        length += remaining.len() - new_remaining.len();
                        remaining = new_remaining;
                        count += 1;
                    } else {
                        break;
                    }
                }
                length
            },
            Pattern::EndOfLine => 0,
            Pattern::OneOrMore(pattern) => {
                let mut length = 0;
                let mut remaining = data;
                while let Some(new_remaining) = pattern.consume_match(remaining, captured_groups) {
                    length += remaining.len() - new_remaining.len();
                    remaining = new_remaining;
                }
                length
            },
            Pattern::ZeroOrOne(pattern) => {
                pattern.consume_match(data, captured_groups).map_or(0, |r| data.len() - r.len())
            },
            Pattern::Alternation(patterns) => patterns
                .iter()
                .filter_map(|p| p.consume_match(data, captured_groups).map(|r| data.len() - r.len()))
                .max()
                .unwrap_or(0),
            Pattern::Backreference(n) => {
                if let Some(group) = captured_groups.get(*n - 1) {
                    if data.starts_with(group) {
                        group.len()
                    } else {
                        0
                    }
                } else {
                    0
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_match_str_exact_char() {
        assert_eq!(Pattern::ExactChar('A').match_str("ABC"), true);
        assert_eq!(Pattern::ExactChar('X').match_str("ABC"), false);
        assert_eq!(Pattern::ExactChar('C').match_str("C"), true);
    }
    #[test]
    fn test_match_str_digit() {
        assert_eq!(Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false }.match_str("123"), true);
        assert_eq!(Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false }.match_str("ABC"), false);
        assert_eq!(Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false }.match_str("9"), true);
    }
    #[test]
    fn test_match_repeated() {
        assert_eq!(
            Pattern::Repeated {
                min: 0,
                max: Some(2),
                pattern: Box::new(Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false })
            }
            .match_str("123"),
            true
        );
        assert_eq!(
            Pattern::Repeated {
                min: 2,
                max: Some(3),
                pattern: Box::new(Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false })
            }
            .match_str("12345"),
            true
        );
        assert_eq!(
            Pattern::Repeated {
                min: 2,
                max: None,
                pattern: Box::new(Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false })
            }
            .match_str("12345"),
            true
        );
        assert_eq!(
            Pattern::Repeated {
                min: 2,
                max: None,
                pattern: Box::new(Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false })
            }
            .match_str("123ABC"),
            true
        );
    }
    #[test]
    fn test_match_str_sequence() {
        assert_eq!(
            Pattern::Sequence(vec![
                Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false },
                Pattern::ExactChar('Z'),
                Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false },
            ])
            .match_str("1Z2XY"),
            true
        );
    }
    #[test_log::test]
    fn test_matches() {
        assert_eq!(
            Pattern::from_str("AB\\d\\dZZ")
                .expect("valid")
                .match_str("AB12ZZCD"),
            true
        );
        assert_eq!(
            Pattern::from_str("..\\dA")
                .expect("valid")
                .match_str("A12A"),
            true
        );
        assert_eq!(
            Pattern::from_str(".*foo")
                .expect("valid")
                .match_str("foobar"),
            true
        );
        assert_eq!(
            Pattern::from_str(".*foo")
                .expect("valid")
                .match_str("somefoobar"),
            true
        );
        assert_eq!(
            Pattern::from_str(".*ZZ.*X")
                .expect("valid")
                .match_str("ABCZZZ12XX"),
            true
        );
        assert_eq!(
            Pattern::from_str("[abc]*test")
                .expect("valid")
                .match_str("aabbcatest12"),
            true
        );
        assert_eq!(
            Pattern::from_str("[^xyz]*xtest")
                .expect("valid")
                .match_str("aabbcaxtest12"),
            true
        );
        assert_eq!(
            Pattern::from_str("[^xyz]*test")
                .expect("valid")
                .match_str("aabbcatest12"),
            true
        );
        assert_eq!(
            Pattern::from_str("\\d apple")
                .expect("valid")
                .match_str("1 apple"),
            true
        );
    }

    #[test]
    fn test_any_char() {
        assert_eq!(Pattern::AnyChar.match_str("ABC"), true);
        assert_eq!(Pattern::AnyChar.match_str("A"), true);
        assert_eq!(Pattern::AnyChar.match_str(""), false);
    }

    #[test]
    fn test_alpha_numeric() {
        assert_eq!(Pattern::AlphaNumeric.match_str("a123"), true);
        assert_eq!(Pattern::AlphaNumeric.match_str("_abc"), true);
        assert_eq!(Pattern::AlphaNumeric.match_str("9xyz"), true);
        assert_eq!(Pattern::AlphaNumeric.match_str("!abc"), false);
    }

    #[test]
    fn test_one_of() {
        let pattern = Pattern::OneOf(vec![
            Pattern::ExactChar('a'),
            Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false },
            Pattern::ExactChar('x'),
        ]);
        assert_eq!(pattern.match_str("abc"), true);
        assert_eq!(pattern.match_str("123"), true);
        assert_eq!(pattern.match_str("xyz"), true);
        assert_eq!(pattern.match_str("bcd"), false);
    }

    #[test]
    fn test_character_set() {
        let pattern = Pattern::CharacterSet {
            chars: "aeiou".to_string(),
            negated: false,
        };
        assert_eq!(pattern.match_str("apple"), true);
        assert_eq!(pattern.match_str("xyz"), false);

        let negated_pattern = Pattern::CharacterSet {
            chars: "aeiou".to_string(),
            negated: true,
        };
        assert_eq!(negated_pattern.match_str("xyz"), true);
        assert_eq!(negated_pattern.match_str("apple"), false);
    }

    #[test]
    fn test_from_str_errors() {
        assert!(Pattern::from_str("a[bc").is_err());
        assert!(Pattern::from_str("a\\").is_err());
        assert!(Pattern::from_str("*").is_err());
    }

    #[test]
    fn test_start_of_line() {
        assert_eq!(
            Pattern::from_str("^log").expect("valid").match_str("log"),
            true
        );
        assert_eq!(Pattern::from_str("^log").expect("valid").match_str("slog"), false);
        assert_eq!(
            Pattern::from_str("^\\d\\d").expect("valid").match_str("12abc"),
            true
        );
        assert_eq!(Pattern::from_str("^\\d\\d").expect("valid").match_str("a12bc"), false);
    }

    #[test]
    fn test_end_of_line() {
        assert_eq!(Pattern::from_str("cat$").expect("valid").match_str("cat"), true);
        assert_eq!(Pattern::from_str("cat$").expect("valid").match_str("cats"), false);
        assert_eq!(Pattern::from_str("cat$").expect("valid").match_str("a cat"), true);
        assert_eq!(Pattern::from_str("^cat$").expect("valid").match_str("cat"), true);
        assert_eq!(Pattern::from_str("^cat$").expect("valid").match_str("a cat"), false);
    }

    #[test]
    fn test_one_or_more() {
        assert_eq!(Pattern::from_str("a+").expect("valid").match_str("a"), true);
        assert_eq!(Pattern::from_str("a+").expect("valid").match_str("aa"), true);
        assert_eq!(Pattern::from_str("a+").expect("valid").match_str(""), false);
        assert_eq!(Pattern::from_str("a+").expect("valid").match_str("b"), false);
        assert_eq!(Pattern::from_str("ca+ts").expect("valid").match_str("caats"), true);
        assert_eq!(Pattern::from_str("ca+ts").expect("valid").match_str("cats"), true);
        assert_eq!(Pattern::from_str("ca+ts").expect("valid").match_str("cts"), false);
    }

    #[test]
    fn test_zero_or_one() {
        assert_eq!(Pattern::from_str("dogs?").expect("valid").match_str("dogs"), true);
        assert_eq!(Pattern::from_str("dogs?").expect("valid").match_str("dog"), true);
        assert_eq!(Pattern::from_str("dogs?").expect("valid").match_str("dogss"), false);
        assert_eq!(Pattern::from_str("dogs?").expect("valid").match_str("cat"), false);
        assert_eq!(Pattern::from_str("colou?r").expect("valid").match_str("color"), true);
        assert_eq!(Pattern::from_str("colou?r").expect("valid").match_str("colour"), true);
    }

    #[test]
    fn test_dot_pattern() {
        assert_eq!(Pattern::from_str("d.g").expect("valid").match_str("dog"), true);
        assert_eq!(Pattern::from_str("d.g").expect("valid").match_str("dig"), true);
        assert_eq!(Pattern::from_str("d.g").expect("valid").match_str("cog"), false);
        assert_eq!(Pattern::from_str("d.g").expect("valid").match_str("dg"), false);
    }

    #[test]
    fn test_alternation() {
        assert_eq!(Pattern::from_str("(cat|dog)").expect("valid").match_str("cat"), true);
        assert_eq!(Pattern::from_str("(cat|dog)").expect("valid").match_str("dog"), true);
        assert_eq!(Pattern::from_str("(cat|dog)").expect("valid").match_str("apple"), false);
        assert_eq!(Pattern::from_str("a(b|c)d").expect("valid").match_str("abd"), true);
        assert_eq!(Pattern::from_str("a(b|c)d").expect("valid").match_str("acd"), true);
        assert_eq!(Pattern::from_str("a(b|c)d").expect("valid").match_str("ad"), false);
    }

    #[test]
    fn test_backreferences() {
        assert_eq!(Pattern::from_str("(cat) and \\1").expect("valid").match_str("cat and cat"), true);
        assert_eq!(Pattern::from_str("(cat) and \\1").expect("valid").match_str("cat and dog"), false);
        assert_eq!(Pattern::from_str("(\\w+) and \\1").expect("valid").match_str("cat and cat"), true);
        assert_eq!(Pattern::from_str("(\\w+) and \\1").expect("valid").match_str("dog and dog"), true);
        assert_eq!(Pattern::from_str("(\\w+) and \\1").expect("valid").match_str("cat and dog"), false);
    }
}