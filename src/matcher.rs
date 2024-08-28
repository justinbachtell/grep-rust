use crate::Pattern;
use log::debug;

impl Pattern {
    pub fn match_str(&self, s: &str) -> bool {
        (0..s.len()).any(|i| {
            let mut captured_groups = Vec::new();
            self.match_from_start(&s[i..], &mut captured_groups, 0)
        })
    }

    fn match_from_start(&self, data: &str, captured_groups: &mut Vec<String>, nested_level: usize) -> bool {
        debug!("match_from_start: pattern={:?}, data={:?}, nested_level={}", self, data, nested_level);
        match self {
            Pattern::ExactChar(c) => data.starts_with(*c),
            Pattern::AnyChar => !data.is_empty(),
            Pattern::AlphaNumeric => data.chars().next().map_or(false, |c| c.is_alphanumeric()),
            Pattern::Sequence(patterns) => {
                let mut remaining = data;
                for pattern in patterns {
                    if let Some(new_remaining) = pattern.consume_match(remaining, captured_groups, nested_level) {
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
                    if let Some(new_remaining) = pattern.consume_match(remaining, captured_groups, nested_level) {
                        remaining = new_remaining;
                        count += 1;
                    } else {
                        break;
                    }
                }
                count >= *min
            },
            Pattern::OneOf(patterns) => patterns.iter().any(|p| p.match_from_start(data, captured_groups, nested_level)),
            Pattern::CharacterSet { chars, negated } => {
                data.chars().next().map_or(false, |c| chars.contains(c) != *negated)
            },
            Pattern::StartOfLine => true, // Assuming we're always at the start in this context
            Pattern::EndOfLine => data.is_empty(),
            Pattern::OneOrMore(pattern) => {
                let mut count = 0;
                let mut remaining = data;
                while let Some(new_remaining) = pattern.consume_match(remaining, captured_groups, nested_level) {
                    remaining = new_remaining;
                    count += 1;
                }
                count > 0
            },
            Pattern::ZeroOrOne(pattern) => {
                pattern.consume_match(data, captured_groups, nested_level).is_some() || true
            },
            Pattern::Alternation(patterns) => patterns.iter().any(|p| p.match_from_start(data, captured_groups, nested_level)),
            Pattern::Backreference(n) => {
                let index = n - 1;
                debug!("Backreference: n={}, index={}, captured_groups={:?}", n, index, captured_groups);
                if let Some(group) = captured_groups.get(index) {
                    let result = data.starts_with(group);
                    debug!("Backreference match: group={:?}, data={:?}, result={}", group, data, result);
                    result
                } else {
                    debug!("Backreference not found: index={}", index);
                    false
                }
            },
            Pattern::CaptureGroup(pattern) => {
                let start_len = captured_groups.len();
                let result = pattern.match_from_start(data, captured_groups, nested_level);
                if result {
                    let length = pattern.match_length(data, captured_groups, nested_level);
                    let captured = data[..length].to_string();
                    captured_groups.push(captured);
                } else {
                    captured_groups.truncate(start_len);
                }
                result
            },
            Pattern::NestedCapture(pattern) => {
                let start_len = captured_groups.len();
                let mut inner_captured_groups = Vec::new();
                let result = pattern.match_from_start(data, &mut inner_captured_groups, nested_level + 1);
                if result {
                    let length = pattern.match_length(data, &mut inner_captured_groups, nested_level + 1);
                    let captured = data[..length].to_string();
                    captured_groups.insert(nested_level, captured.clone());
                    captured_groups.extend(inner_captured_groups);
                    debug!("NestedCapture: captured={:?}, captured_groups={:?}", captured, captured_groups);
                    true
                } else {
                    captured_groups.truncate(start_len);
                    false
                }
            },
        }
    }

    fn match_length(&self, data: &str, captured_groups: &mut Vec<String>, nested_level: usize) -> usize {
        debug!("match_length: pattern={:?}, data={:?}, nested_level={}", self, data, nested_level);
        match self {
            Pattern::ExactChar(c) => if data.starts_with(*c) { 1 } else { 0 },
            Pattern::AnyChar => if !data.is_empty() { 1 } else { 0 },
            Pattern::AlphaNumeric => if data.chars().next().map_or(false, |c| c.is_alphanumeric()) { 1 } else { 0 },
            Pattern::Sequence(patterns) => {
                let mut length = 0;
                let mut remaining = data;
                for pattern in patterns {
                    if let Some(new_remaining) = pattern.consume_match(remaining, captured_groups, nested_level) {
                        length += remaining.len() - new_remaining.len();
                        remaining = new_remaining;
                    } else {
                        break;
                    }
                }
                length
            },
            Pattern::Repeated { min, max, pattern } => {
                let mut count = 0;
                let mut length = 0;
                let mut remaining = data;
                while max.map_or(true, |m| count < m) {
                    if let Some(new_remaining) = pattern.consume_match(remaining, captured_groups, nested_level) {
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
                .filter_map(|p| p.consume_match(data, captured_groups, nested_level).map(|r| data.len() - r.len()))
                .next()
                .unwrap_or(0),
            Pattern::CharacterSet { chars, negated } => {
                if data.chars().next().map_or(false, |c| chars.contains(c) != *negated) { 1 } else { 0 }
            },
            Pattern::StartOfLine => 0,
            Pattern::EndOfLine => 0,
            Pattern::OneOrMore(pattern) => {
                let mut length = 0;
                let mut remaining = data;
                while let Some(new_remaining) = pattern.consume_match(remaining, captured_groups, nested_level) {
                    length += remaining.len() - new_remaining.len();
                    remaining = new_remaining;
                }
                length
            },
            Pattern::ZeroOrOne(pattern) => {
                pattern.consume_match(data, captured_groups, nested_level)
                    .map(|r| data.len() - r.len())
                    .unwrap_or(0)
            },
            Pattern::Alternation(patterns) => patterns
                .iter()
                .filter_map(|p| p.consume_match(data, captured_groups, nested_level).map(|r| data.len() - r.len()))
                .max()
                .unwrap_or(0),
            Pattern::Backreference(n) => {
                let index = n - 1;
                debug!("Backreference: n={}, index={}, captured_groups={:?}", n, index, captured_groups);
                if let Some(group) = captured_groups.get(index) {
                    let length = if data.starts_with(group) {
                        group.len()
                    } else {
                        0
                    };
                    debug!("Backreference match: group={:?}, data={:?}, length={}", group, data, length);
                    length
                } else {
                    debug!("Backreference not found: index={}", index);
                    0
                }
            },
            Pattern::CaptureGroup(pattern) => pattern.match_length(data, captured_groups, nested_level),
            Pattern::NestedCapture(pattern) => {
                let start_len = captured_groups.len();
                let mut inner_captured_groups = Vec::new();
                let length = pattern.match_length(data, &mut inner_captured_groups, nested_level + 1);
                if length > 0 {
                    let captured = data[..length].to_string();
                    captured_groups.insert(nested_level, captured.clone());
                    captured_groups.extend(inner_captured_groups);
                    debug!("NestedCapture: captured={:?}, captured_groups={:?}", captured, captured_groups);
                    length
                } else {
                    captured_groups.truncate(start_len);
                    0
                }
            },
        }
    }

    fn consume_match<'a>(&self, data: &'a str, captured_groups: &mut Vec<String>, nested_level: usize) -> Option<&'a str> {
        let start_len = captured_groups.len();
        let length = self.match_length(data, captured_groups, nested_level);
        if length > 0 {
            Some(&data[length..])
        } else {
            captured_groups.truncate(start_len);
            None
        }
    }
}