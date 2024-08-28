use crate::Pattern;
use std::str::FromStr;

#[test]
fn test_parse_exact_char() {
    assert_eq!(Pattern::from_str("a").unwrap(), Pattern::ExactChar('a'));
}

#[test]
fn test_parse_any_char() {
    assert_eq!(Pattern::from_str(".").unwrap(), Pattern::AnyChar);
}

#[test]
fn test_parse_digit() {
    assert_eq!(Pattern::from_str("\\d").unwrap(), Pattern::CharacterSet { 
        chars: "0123456789".to_string(), 
        negated: false 
    });
}

#[test]
fn test_parse_alphanumeric() {
    assert_eq!(Pattern::from_str("\\w").unwrap(), Pattern::AlphaNumeric);
}

#[test]
fn test_parse_sequence() {
    assert_eq!(
        Pattern::from_str("abc").unwrap(),
        Pattern::Sequence(vec![
            Pattern::ExactChar('a'),
            Pattern::ExactChar('b'),
            Pattern::ExactChar('c')
        ])
    );
}

#[test]
fn test_parse_repeated() {
    assert_eq!(
        Pattern::from_str("a{2,3}").unwrap(),
        Pattern::Repeated {
            min: 2,
            max: Some(3),
            pattern: Box::new(Pattern::ExactChar('a'))
        }
    );
}

#[test]
fn test_parse_one_or_more() {
    assert_eq!(
        Pattern::from_str("a+").unwrap(),
        Pattern::OneOrMore(Box::new(Pattern::ExactChar('a')))
    );
}

#[test]
fn test_parse_zero_or_one() {
    assert_eq!(
        Pattern::from_str("a?").unwrap(),
        Pattern::ZeroOrOne(Box::new(Pattern::ExactChar('a')))
    );
}

#[test]
fn test_parse_character_set() {
    assert_eq!(
        Pattern::from_str("[abc]").unwrap(),
        Pattern::CharacterSet {
            chars: "abc".to_string(),
            negated: false
        }
    );
}

#[test]
fn test_parse_negated_character_set() {
    assert_eq!(
        Pattern::from_str("[^abc]").unwrap(),
        Pattern::CharacterSet {
            chars: "abc".to_string(),
            negated: true
        }
    );
}

#[test]
fn test_parse_alternation() {
    assert_eq!(
        Pattern::from_str("(a|b)").unwrap(),
        Pattern::Alternation(vec![
            Pattern::ExactChar('a'),
            Pattern::ExactChar('b')
        ])
    );
}

#[test]
fn test_parse_capture_group() {
    assert_eq!(
        Pattern::from_str("(abc)").unwrap(),
        Pattern::CaptureGroup(Box::new(Pattern::Sequence(vec![
            Pattern::ExactChar('a'),
            Pattern::ExactChar('b'),
            Pattern::ExactChar('c')
        ])))
    );
}

#[test]
fn test_parse_backreference() {
    assert_eq!(
        Pattern::from_str("(a)\\1").unwrap(),
        Pattern::Sequence(vec![
            Pattern::CaptureGroup(Box::new(Pattern::ExactChar('a'))),
            Pattern::Backreference(1)
        ])
    );
}

#[test]
fn test_parse_nested_capture() {
    assert_eq!(
        Pattern::from_str("((a)b)").unwrap(),
        Pattern::NestedCapture(Box::new(Pattern::Sequence(vec![
            Pattern::CaptureGroup(Box::new(Pattern::ExactChar('a'))),
            Pattern::ExactChar('b')
        ])))
    );
}

#[test]
fn test_parse_errors() {
    assert!(Pattern::from_str("[abc").is_err());
    assert!(Pattern::from_str("\\").is_err());
    assert!(Pattern::from_str("*").is_err());
}