use crate::Pattern;

pub struct Matcher;

impl Matcher {
    pub fn match_str(pattern: &Pattern, s: &str) -> bool {
        s.lines().any(|line| Self::match_line(pattern, line))
    }

    fn match_line(pattern: &Pattern, line: &str) -> bool {
        match pattern {
            Pattern::StartOfLine => Self::match_from_start(pattern, line, &mut Vec::new()),
            Pattern::EndOfLine => line.is_empty() || Self::match_from_start(pattern, line, &mut Vec::new()) || Self::match_from_end(pattern, line),
            _ => (0..=line.len()).any(|i| Self::match_from_start(pattern, &line[i..], &mut Vec::new()))
        }
    }

    fn match_from_start(pattern: &Pattern, data: &str, captured_groups: &mut Vec<String>) -> bool {
        match pattern {
            Pattern::ExactChar(c) => data.starts_with(*c),
            Pattern::AnyChar => !data.is_empty(),
            Pattern::AlphaNumeric => data.chars().next().map_or(false, |c| c.is_alphanumeric() || c == '_'),
            Pattern::Sequence(patterns) => {
                let mut remaining = data;
                for p in patterns {
                    if let Some(new_remaining) = Self::consume_match(p, remaining, captured_groups) {
                        remaining = new_remaining;
                    } else {
                        return false;
                    }
                }
                true
            },
            Pattern::Repeated { min, max, pattern } => {
                let mut count = 0;
                let mut remaining = data;
                while max.map_or(true, |m| count < m) {
                    if let Some(new_remaining) = Self::consume_match(pattern, remaining, captured_groups) {
                        remaining = new_remaining;
                        count += 1;
                    } else {
                        break;
                    }
                }
                count >= *min
            },
            Pattern::OneOf(patterns) => patterns.iter().any(|p| Self::match_from_start(p, data, captured_groups)),
            Pattern::CharacterSet { chars, negated } => {
                data.chars().next().map_or(false, |c| chars.contains(c) != *negated)
            },
            Pattern::StartOfLine => data.lines().next() == Some(data),
            Pattern::EndOfLine => data.is_empty(),
            Pattern::OneOrMore(pattern) => {
                let mut count = 0;
                let mut remaining = data;
                while let Some(new_remaining) = Self::consume_match(pattern, remaining, captured_groups) {
                    remaining = new_remaining;
                    count += 1;
                }
                count > 0
            },
            Pattern::ZeroOrOne(pattern) => {
                Self::match_from_start(pattern, data, captured_groups) || true
            },
            Pattern::Alternation(patterns) => patterns.iter().any(|p| Self::match_from_start(p, data, captured_groups)),
            Pattern::Backreference(n) => {
                let index = n - 1;
                captured_groups.get(index).map_or(false, |group| data.starts_with(group))
            },
            Pattern::CaptureGroup(pattern) | Pattern::NestedCapture(pattern) => {
                let start_len = captured_groups.len();
                let result = Self::match_from_start(pattern, data, captured_groups);
                if result {
                    let length = Self::match_length(pattern, data, captured_groups);
                    let captured = data[..length].to_string();
                    captured_groups.push(captured);
                } else {
                    captured_groups.truncate(start_len);
                }
                result
            },
        }
    }

    fn match_from_end(pattern: &Pattern, data: &str) -> bool {
        match pattern {
            Pattern::EndOfLine => data.is_empty(),
            _ => false,
        }
    }

    fn consume_match<'a>(pattern: &Pattern, data: &'a str, captured_groups: &mut Vec<String>) -> Option<&'a str> {
        let start_len = captured_groups.len();
        let length = Self::match_length(pattern, data, captured_groups);
        if length > 0 {
            Some(&data[length..])
        } else {
            captured_groups.truncate(start_len);
            None
        }
    }

    fn match_length(pattern: &Pattern, data: &str, captured_groups: &mut Vec<String>) -> usize {
        match pattern {
            Pattern::ExactChar(c) => if data.starts_with(*c) { 1 } else { 0 },
            Pattern::AnyChar => if !data.is_empty() { 1 } else { 0 },
            Pattern::AlphaNumeric => if data.chars().next().map_or(false, |c| c.is_alphanumeric() || c == '_') { 1 } else { 0 },
            Pattern::Sequence(patterns) => {
                let mut length = 0;
                let mut remaining = data;
                for pattern in patterns {
                    if let Some(new_remaining) = Self::consume_match(pattern, remaining, captured_groups) {
                        length += remaining.len() - new_remaining.len();
                        remaining = new_remaining;
                    } else {
                        return 0;
                    }
                }
                length
            },
            Pattern::Repeated { min, max, pattern } => {
                let mut count = 0;
                let mut length = 0;
                let mut remaining = data;
                while max.map_or(true, |m| count < m) {
                    if let Some(new_remaining) = Self::consume_match(pattern, remaining, captured_groups) {
                        length += remaining.len() - new_remaining.len();
                        remaining = new_remaining;
                        count += 1;
                    } else {
                        break;
                    }
                }
                if count >= *min { length } else { 0 }
            },
            Pattern::OneOf(patterns) => patterns
                .iter()
                .map(|p| Self::match_length(p, data, captured_groups))
                .max()
                .unwrap_or(0),
            Pattern::CharacterSet { chars, negated } => {
                if data.chars().next().map_or(false, |c| chars.contains(c) != *negated) { 1 } else { 0 }
            },
            Pattern::StartOfLine => 0,
            Pattern::EndOfLine => 0,
            Pattern::OneOrMore(pattern) => {
                let mut length = 0;
                let mut remaining = data;
                while let Some(new_remaining) = Self::consume_match(pattern, remaining, captured_groups) {
                    length += remaining.len() - new_remaining.len();
                    remaining = new_remaining;
                }
                length
            },
            Pattern::ZeroOrOne(pattern) => {
                Self::consume_match(pattern, data, captured_groups)
                    .map(|r| data.len() - r.len())
                    .unwrap_or(0)
            },
            Pattern::Alternation(patterns) => patterns
                .iter()
                .map(|p| Self::match_length(p, data, captured_groups))
                .max()
                .unwrap_or(0),
            Pattern::Backreference(n) => {
                let index = n - 1;
                captured_groups.get(index)
                    .filter(|&group| data.starts_with(group))
                    .map(|group| group.len())
                    .unwrap_or(0)
            },
            Pattern::CaptureGroup(pattern) | Pattern::NestedCapture(pattern) => {
                let start_len = captured_groups.len();
                let length = Self::match_length(pattern, data, captured_groups);
                if length > 0 {
                    let captured = data[..length].to_string();
                    captured_groups.push(captured);
                    length
                } else {
                    captured_groups.truncate(start_len);
                    0
                }
            },
        }
    }
}