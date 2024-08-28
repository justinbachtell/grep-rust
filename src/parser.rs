use crate::Pattern;
use std::str::FromStr;

impl FromStr for Pattern {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_group(s: &str, chars: &mut std::str::Chars, nested_level: usize) -> Result<Pattern, String> {
            let mut alternatives = vec![];
            let mut current = vec![];

            while let Some(c) = chars.next() {
                match c {
                    '(' => {
                        let nested = parse_group(s, chars, nested_level + 1)?;
                        if nested_level == 0 {
                            current.push(Pattern::NestedCapture(Box::new(nested)));
                        } else {
                            current.push(Pattern::CaptureGroup(Box::new(nested)));
                        }
                    },
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
                    '\\' => match chars.next() {
                        Some('w') => current.push(Pattern::AlphaNumeric),
                        Some('d') => current.push(Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false }),
                        Some(d) if d.is_digit(10) => {
                            let backreference = d.to_digit(10).unwrap() as usize;
                            current.push(Pattern::Backreference(backreference));
                        }
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

        parse_group(s, &mut s.chars(), 0)
    }
}