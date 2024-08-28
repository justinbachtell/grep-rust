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
}

impl FromStr for Pattern {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut char_iterator = s.chars();
        let mut items = Vec::new();
        while let Some(c) = char_iterator.next() {
            let el = match c {
                '\\' => match char_iterator.next() {
                    Some('w') => Pattern::AlphaNumeric,
                    Some('d') => Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false },
                    Some(c) => Pattern::ExactChar(c),
                    None => return Err(format!("Unterminated escape in {:?}", s)),
                },
                '.' => Pattern::AnyChar,
                '*' => {
                    match items.pop() {
                        Some(p) => Pattern::Repeated {
                            min: 0,
                            max: None,
                            pattern: Box::new(p),
                        },
                        None => return Err("Invalid repeat".into()),
                    }
                }
                '[' => {
                    let mut chars = String::new();
                    let mut found_end = false;
                    let mut negated = false;
                    for c2 in char_iterator.by_ref() {
                        match c2 {
                            '^' if chars.is_empty() => negated = true,
                            ']' => {
                                found_end = true;
                                break;
                            }
                            other => chars.push(other),
                        }
                    }
                    if !found_end {
                        return Err("Unterminated '[' pattern".into());
                    }
                    Pattern::CharacterSet { chars, negated }
                }
                '^' if items.is_empty() => Pattern::StartOfLine,
                '$' if char_iterator.clone().next().is_none() => Pattern::EndOfLine,
                e => Pattern::ExactChar(e),
            };
            items.push(el);
        }
        if items.len() == 1 {
            return Ok(items.pop().expect("has an element"));
        }
        Ok(Pattern::Sequence(items))
    }
}

impl Pattern {
    #[instrument]
    pub fn match_str(&self, data: &str) -> bool {
        trace!("Matching starts");
        match self {
            Pattern::StartOfLine => self.match_from_start(data),
            Pattern::EndOfLine => data.is_empty(),
            Pattern::Sequence(patterns) => {
                if patterns.first() == Some(&Pattern::StartOfLine) && patterns.last() == Some(&Pattern::EndOfLine) {
                    // Both start and end anchors
                    let without_anchors = Pattern::Sequence(patterns[1..patterns.len()-1].to_vec());
                    without_anchors.match_from_start(data) && data.len() == without_anchors.match_length(data)
                } else if patterns.first() == Some(&Pattern::StartOfLine) {
                    // Only start anchor
                    self.match_from_start(data)
                } else if patterns.last() == Some(&Pattern::EndOfLine) {
                    // Only end anchor
                    let without_end = Pattern::Sequence(patterns[..patterns.len() - 1].to_vec());
                    without_end.match_from_start(data) && data.len() == without_end.match_length(data)
                } else {
                    // No anchors
                    (0..=data.len()).any(|i| self.match_from_start(&data[i..]))
                }
            }
            _ => (0..=data.len()).any(|i| self.match_from_start(&data[i..]))
        }
    }

    fn match_from_start(&self, data: &str) -> bool {
        match self {
            Pattern::ExactChar(c) => data.starts_with(*c),
            Pattern::AnyChar => !data.is_empty(),
            Pattern::AlphaNumeric => data.chars().next().map_or(false, |c| c.is_alphanumeric() || c == '_'),
            Pattern::Sequence(sub_patterns) => {
                let mut remaining = data;
                for sub_pattern in sub_patterns {
                    if let Some(new_remaining) = sub_pattern.consume_match(remaining) {
                        remaining = new_remaining;
                    } else {
                        return false;
                    }
                }
                true
            },
            Pattern::CharacterSet { chars, negated } => {
                data.chars().next().map_or(false, |c| chars.contains(c) != *negated)
            },
            Pattern::StartOfLine => true,
            Pattern::OneOf(sub_patterns) => sub_patterns.iter().any(|p| p.match_from_start(data)),
            Pattern::Repeated { min, max, pattern } => {
                let mut count = 0;
                let mut remaining = data;
                while max.map_or(true, |m| count < m) {
                    if let Some(new_remaining) = pattern.consume_match(remaining) {
                        remaining = new_remaining;
                        count += 1;
                    } else {
                        break;
                    }
                }
                count >= *min
            },
            Pattern::EndOfLine => data.is_empty(),
        }
    }

    fn consume_match<'a>(&self, data: &'a str) -> Option<&'a str> {
        if self.match_from_start(data) {
            Some(&data[self.match_length(data)..])
        } else {
            None
        }
    }

    fn match_length(&self, data: &str) -> usize {
        match self {
            Pattern::ExactChar(_) => 1,
            Pattern::AnyChar => 1,
            Pattern::AlphaNumeric => 1,
            Pattern::Sequence(sub_patterns) => {
                let mut length = 0;
                let mut remaining = data;
                for sub_pattern in sub_patterns {
                    if let Some(new_remaining) = sub_pattern.consume_match(remaining) {
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
                .filter_map(|p| p.consume_match(data).map(|r| data.len() - r.len()))
                .next()
                .unwrap_or(0),
            Pattern::Repeated { min: _, max, pattern } => {
                let mut count = 0;
                let mut length = 0;
                let mut remaining = data;
                while max.map_or(true, |m| count < m) {
                    if let Some(new_remaining) = pattern.consume_match(remaining) {
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
}