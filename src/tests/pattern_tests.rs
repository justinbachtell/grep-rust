use crate::Pattern;

#[test]
fn test_pattern_equality() {
    assert_eq!(Pattern::ExactChar('a'), Pattern::ExactChar('a'));
    assert_ne!(Pattern::ExactChar('a'), Pattern::ExactChar('b'));
    assert_eq!(Pattern::AnyChar, Pattern::AnyChar);
    assert_ne!(Pattern::AnyChar, Pattern::ExactChar('a'));
}

#[test]
fn test_pattern_clone() {
    let pattern = Pattern::Sequence(vec![
        Pattern::ExactChar('a'),
        Pattern::AnyChar,
        Pattern::ExactChar('b')
    ]);
    let cloned_pattern = pattern.clone();
    assert_eq!(pattern, cloned_pattern);
}

#[test]
fn test_pattern_debug() {
    let pattern = Pattern::Repeated {
        min: 2,
        max: Some(3),
        pattern: Box::new(Pattern::ExactChar('a'))
    };
    let debug_output = format!("{:?}", pattern);
    assert!(debug_output.contains("Repeated"));
    assert!(debug_output.contains("min: 2"));
    assert!(debug_output.contains("max: Some(3)"));
    assert!(debug_output.contains("ExactChar('a')"));
}

#[test]
fn test_pattern_partial_ord() {
    assert!(Pattern::ExactChar('a') < Pattern::ExactChar('b'));
    assert!(Pattern::AnyChar > Pattern::ExactChar('z'));
    assert!(Pattern::AlphaNumeric <= Pattern::AlphaNumeric);
}

#[test]
fn test_nested_patterns() {
    let nested_pattern = Pattern::OneOrMore(Box::new(Pattern::Alternation(vec![
        Pattern::ExactChar('a'),
        Pattern::ExactChar('b')
    ])));
    assert_ne!(nested_pattern, Pattern::ExactChar('a'));
}

#[test]
fn test_character_set_creation() {
    let char_set = Pattern::CharacterSet {
        chars: "abc".to_string(),
        negated: false
    };
    assert_eq!(char_set, Pattern::CharacterSet {
        chars: "abc".to_string(),
        negated: false
    });
    assert_ne!(char_set, Pattern::CharacterSet {
        chars: "abc".to_string(),
        negated: true
    });
}

#[test]
fn test_backreference_creation() {
    let backreference = Pattern::Backreference(1);
    assert_eq!(backreference, Pattern::Backreference(1));
    assert_ne!(backreference, Pattern::Backreference(2));
}

#[test]
fn test_capture_group_creation() {
    let capture_group = Pattern::CaptureGroup(Box::new(Pattern::ExactChar('a')));
    assert_eq!(capture_group, Pattern::CaptureGroup(Box::new(Pattern::ExactChar('a'))));
    assert_ne!(capture_group, Pattern::CaptureGroup(Box::new(Pattern::ExactChar('b'))));
}

#[test]
fn test_nested_capture_creation() {
    let nested_capture = Pattern::NestedCapture(Box::new(Pattern::Sequence(vec![
        Pattern::ExactChar('a'),
        Pattern::ExactChar('b')
    ])));
    assert_eq!(nested_capture, Pattern::NestedCapture(Box::new(Pattern::Sequence(vec![
        Pattern::ExactChar('a'),
        Pattern::ExactChar('b')
    ]))));
    assert_ne!(nested_capture, Pattern::NestedCapture(Box::new(Pattern::Sequence(vec![
        Pattern::ExactChar('b'),
        Pattern::ExactChar('a')
    ]))));
}