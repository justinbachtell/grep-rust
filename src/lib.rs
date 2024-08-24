use std::{collections::HashSet, str::FromStr};
use map_macro::hash_set;
use tracing::{instrument, trace};
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Pattern {
    ExactChar(char),
    AnyChar,
    Digit,        // 0-9
    AlphaNumeric, // a-zA-Z0-9_
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
}
trait CharOperations {
    fn first_char(&self) -> Option<char>;
    fn first_char_in(&self, options: &str) -> bool;
    fn skip_first_char(&self) -> Self;
}
impl CharOperations for &str {
    fn first_char(&self) -> Option<char> {
        return self.chars().next();
    }
    fn first_char_in(&self, options: &str) -> bool {
        match self.first_char() {
            Some(c) => options.contains(c),
            None => false,
        }
    }
    fn skip_first_char(&self) -> Self {
        &self[1..]
    }
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
                    Some('d') => Pattern::Digit,
                    Some(c) => Pattern::ExactChar(c), // assume an escape
                    None => return Err(format!("Unterminated escape in {:?}", s)),
                },
                '.' => Pattern::AnyChar,
                '*' => {
                    // need to grab last item and repeat
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
                            // TODO: should we handle escapes here?
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
    pub fn match_str<'a>(&'_ self, data: &'a str) -> HashSet<&'a str> {
        trace!("Matching starts");
        match self {
            Pattern::AnyChar if data.first_char().is_some() => hash_set! {data.skip_first_char()},
            Pattern::ExactChar(c) if data.first_char() == Some(*c) => {
                hash_set! {data.skip_first_char()}
            }
            Pattern::Digit if data.first_char_in("0123456789") => {
                hash_set! {data.skip_first_char()}
            }
            Pattern::AlphaNumeric
                if data.first_char_in(
                    "_0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
                ) =>
            {
                hash_set! {data.skip_first_char()}
            }
            Pattern::Sequence(sub_patterns) => {
                let mut remaining = hash_set! {data};
                for sub_pattern in sub_patterns {
                    let mut next_remaining = HashSet::new();
                    for r in remaining.iter() {
                        for i in 0..r.len() {
                            let sub_matches = sub_pattern.match_str(&r[i..]);
                            if !sub_matches.is_empty() {
                                next_remaining.extend(sub_matches);
                                break;
                            }
                        }
                    }
                    remaining = next_remaining;
                    if remaining.is_empty() {
                        break;
                    }
                }
                remaining
            }
            Pattern::CharacterSet { chars, negated } => {
                trace!(
                    "TEST: {} and {} (for {})",
                    data.first_char_in(chars),
                    negated,
                    chars
                );
                if !data.is_empty() && data.first_char_in(chars) != *negated {
                    hash_set! {data.skip_first_char()}
                } else {
                    HashSet::new()
                }
            }
            Pattern::OneOf(sub_patterns) => {
                let mut result = HashSet::new();
                for sub_pattern in sub_patterns {
                    result.extend(sub_pattern.match_str(data))
                }
                result
            }
            Pattern::Repeated { min, max, pattern } => {
                let mut results: HashSet<&str> = HashSet::new();
                let mut remaining = vec![data];
                let mut count = 0;
                while !remaining.is_empty() {
                    if count >= *min {
                        // all matches appended
                        results.extend(remaining.iter());
                    }
                    count += 1;
                    // did we reach max count
                    if max.map(|m| m < count).unwrap_or(false) {
                        break;
                    }
                    // try matching for the pattern and append
                    let mut new_ends = Vec::new();
                    for r in remaining {
                        for x in pattern.match_str(r) {
                            if results.contains(x) {
                                continue; // already considered
                            }
                            new_ends.push(x);
                        }
                    }
                    remaining = new_ends;
                }
                results
            }
            Pattern::StartOfLine => {
                if data.is_empty() || data.starts_with('\n') {
                    hash_set! { data }
                } else {
                    HashSet::new()
                }
            }
            _ => HashSet::new(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_match_str_exact_char() {
        assert_eq!(Pattern::ExactChar('A').match_str("ABC"), hash_set! {"BC"});
        assert!(Pattern::ExactChar('X').match_str("ABC").is_empty());
        assert_eq!(Pattern::ExactChar('C').match_str("C"), hash_set![""]);
    }
    #[test]
    fn test_match_str_digit() {
        assert_eq!(Pattern::Digit.match_str("123"), hash_set!["23"]);
        assert!(Pattern::Digit.match_str("ABC").is_empty());
        assert_eq!(Pattern::Digit.match_str("9"), hash_set![""]);
    }
    #[test]
    fn test_match_repeated() {
        assert_eq!(
            Pattern::Repeated {
                min: 0,
                max: Some(2),
                pattern: Box::new(Pattern::Digit)
            }
            .match_str("123"),
            hash_set!["123", "23", "3"],
        );
        assert_eq!(
            Pattern::Repeated {
                min: 2,
                max: Some(3),
                pattern: Box::new(Pattern::Digit)
            }
            .match_str("12345"),
            hash_set!["345", "45"]
        );
        assert_eq!(
            Pattern::Repeated {
                min: 2,
                max: None,
                pattern: Box::new(Pattern::Digit)
            }
            .match_str("12345"),
            hash_set!["345", "45", "5", ""]
        );
        assert_eq!(
            Pattern::Repeated {
                min: 2,
                max: None,
                pattern: Box::new(Pattern::Digit)
            }
            .match_str("123ABC"),
            hash_set!["3ABC", "ABC"]
        );
    }
    #[test]
    fn test_match_str_sequence() {
        assert_eq!(
            Pattern::Sequence(vec![
                Pattern::Digit,
                Pattern::ExactChar('Z'),
                Pattern::Digit,
            ])
            .match_str("1Z2XY"),
            hash_set!["XY"]
        );
    }
    #[test_log::test]
    fn test_matches() {
        assert_eq!(
            Pattern::from_str("AB\\d\\dZZ")
                .expect("valid")
                .match_str("AB12ZZCD"),
            hash_set!["CD"]
        );
        assert_eq!(
            Pattern::from_str("..\\dA")
                .expect("valid")
                .match_str("A12A"),
            hash_set![""]
        );
        assert_eq!(
            Pattern::from_str(".*foo")
                .expect("valid")
                .match_str("foobar"),
            hash_set!["bar"]
        );
        assert_eq!(
            Pattern::from_str(".*foo")
                .expect("valid")
                .match_str("somefoobar"),
            hash_set!["bar"]
        );
        assert_eq!(
            Pattern::from_str(".*ZZ.*X")
                .expect("valid")
                .match_str("ABCZZZ12XX"),
            hash_set!["X", ""]
        );
        assert_eq!(
            Pattern::from_str("[abc]*test")
                .expect("valid")
                .match_str("aabbcatest12"),
            hash_set!["12"]
        );
        assert_eq!(
            Pattern::from_str("[^xyz]*xtest")
                .expect("valid")
                .match_str("aabbcaxtest12"),
            hash_set!["12"]
        );
        assert_eq!(
            Pattern::from_str("[^xyz]*test")
                .expect("valid")
                .match_str("aabbcatest12"),
            hash_set!["12"]
        );
        assert_eq!(
            Pattern::from_str("\\d apple")
                .expect("valid")
                .match_str("1 apple"),
            hash_set![""]
        );
    }

    #[test]
    fn test_any_char() {
        assert_eq!(Pattern::AnyChar.match_str("ABC"), hash_set!["BC"]);
        assert_eq!(Pattern::AnyChar.match_str("A"), hash_set![""]);
        assert!(Pattern::AnyChar.match_str("").is_empty());
    }

    #[test]
    fn test_alpha_numeric() {
        assert_eq!(Pattern::AlphaNumeric.match_str("a123"), hash_set!["123"]);
        assert_eq!(Pattern::AlphaNumeric.match_str("_abc"), hash_set!["abc"]);
        assert_eq!(Pattern::AlphaNumeric.match_str("9xyz"), hash_set!["xyz"]);
        assert!(Pattern::AlphaNumeric.match_str("!abc").is_empty());
    }

    #[test]
    fn test_one_of() {
        let pattern = Pattern::OneOf(vec![
            Pattern::ExactChar('a'),
            Pattern::Digit,
            Pattern::ExactChar('x'),
        ]);
        assert_eq!(pattern.match_str("abc"), hash_set!["bc"]);
        assert_eq!(pattern.match_str("123"), hash_set!["23"]);
        assert_eq!(pattern.match_str("xyz"), hash_set!["yz"]);
        assert!(pattern.match_str("bcd").is_empty());
    }

    #[test]
    fn test_character_set() {
        let pattern = Pattern::CharacterSet {
            chars: "aeiou".to_string(),
            negated: false,
        };
        assert_eq!(pattern.match_str("apple"), hash_set!["pple"]);
        assert!(pattern.match_str("xyz").is_empty());

        let negated_pattern = Pattern::CharacterSet {
            chars: "aeiou".to_string(),
            negated: true,
        };
        assert_eq!(negated_pattern.match_str("xyz"), hash_set!["yz"]);
        assert!(negated_pattern.match_str("apple").is_empty());
    }

    #[test]
    fn test_from_str_errors() {
        assert!(Pattern::from_str("a[bc").is_err());
        assert!(Pattern::from_str("a\\").is_err());
        assert!(Pattern::from_str("*").is_err());
    }
}