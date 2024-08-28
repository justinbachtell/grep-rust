use crate::Pattern;
use crate::matcher::Matcher;
use crate::parser::parse_pattern;

#[test]
fn test_match_str_exact_char() {
    assert!(Matcher::match_str(&Pattern::ExactChar('A'), "ABC"));
    assert!(!Matcher::match_str(&Pattern::ExactChar('X'), "ABC"));
    assert!(Matcher::match_str(&Pattern::ExactChar('C'), "C"));
}

#[test]
fn test_match_str_digit() {
    assert!(Matcher::match_str(&parse_pattern("\\d").unwrap(), "123"));
    assert!(!Matcher::match_str(&parse_pattern("\\d").unwrap(), "ABC"));
    assert!(Matcher::match_str(&parse_pattern("\\d").unwrap(), "9"));
}

#[test]
fn test_match_repeated() {
    assert!(Matcher::match_str(&parse_pattern("\\d{0,2}").unwrap(), "123"));
    assert!(Matcher::match_str(&parse_pattern("\\d{2,3}").unwrap(), "12345"));
    assert!(Matcher::match_str(&parse_pattern("\\d{2,}").unwrap(), "12345"));
    assert!(Matcher::match_str(&parse_pattern("\\d{2,}").unwrap(), "123ABC"));
}

#[test]
fn test_match_str_sequence() {
    assert!(Matcher::match_str(&parse_pattern("\\dZ\\d").unwrap(), "1Z2XY"));
}

#[test]
fn test_matches() {
    assert!(Matcher::match_str(&parse_pattern("AB\\d\\dZZ").unwrap(), "AB12ZZCD"));
    assert!(Matcher::match_str(&parse_pattern("..\\dA").unwrap(), "A12A"));
    assert!(Matcher::match_str(&parse_pattern(".*foo").unwrap(), "foobar"));
    assert!(Matcher::match_str(&parse_pattern(".*foo").unwrap(), "somefoobar"));
    assert!(Matcher::match_str(&parse_pattern(".*ZZ.*X").unwrap(), "ABCZZZ12XX"));
    assert!(Matcher::match_str(&parse_pattern("[abc]*test").unwrap(), "aabbcatest12"));
    assert!(Matcher::match_str(&parse_pattern("[^xyz]*xtest").unwrap(), "aabbcaxtest12"));
    assert!(Matcher::match_str(&parse_pattern("[^xyz]*test").unwrap(), "aabbcatest12"));
    assert!(Matcher::match_str(&parse_pattern("\\d apple").unwrap(), "1 apple"));
}

#[test]
fn test_any_char() {
    assert!(Matcher::match_str(&Pattern::AnyChar, "ABC"));
    assert!(Matcher::match_str(&Pattern::AnyChar, "A"));
    assert!(!Matcher::match_str(&Pattern::AnyChar, ""));
}

#[test]
fn test_alpha_numeric() {
    assert!(Matcher::match_str(&Pattern::AlphaNumeric, "a123"));
    assert!(Matcher::match_str(&Pattern::AlphaNumeric, "_abc"));
    assert!(Matcher::match_str(&Pattern::AlphaNumeric, "9xyz"));
    assert!(!Matcher::match_str(&Pattern::AlphaNumeric, "!abc"));
}

#[test]
fn test_one_of() {
    let pattern = Pattern::OneOf(vec![
        Pattern::ExactChar('a'),
        Pattern::CharacterSet { chars: "0123456789".to_string(), negated: false },
        Pattern::ExactChar('x'),
    ]);
    assert!(Matcher::match_str(&pattern, "abc"));
    assert!(Matcher::match_str(&pattern, "123"));
    assert!(Matcher::match_str(&pattern, "xyz"));
    assert!(!Matcher::match_str(&pattern, "bcd"));
}

#[test]
fn test_character_set() {
    let pattern = Pattern::CharacterSet {
        chars: "aeiou".to_string(),
        negated: false,
    };
    assert!(Matcher::match_str(&pattern, "apple"));
    assert!(!Matcher::match_str(&pattern, "xyz"));

    let negated_pattern = Pattern::CharacterSet {
        chars: "aeiou".to_string(),
        negated: true,
    };
    assert!(Matcher::match_str(&negated_pattern, "xyz"));
    assert!(!Matcher::match_str(&negated_pattern, "apple"));
}

#[test]
fn test_start_of_line() {
    assert!(Matcher::match_str(&parse_pattern("^log").unwrap(), "log"));
    assert!(!Matcher::match_str(&parse_pattern("^log").unwrap(), "slog"));
    assert!(Matcher::match_str(&parse_pattern("^\\d\\d").unwrap(), "12abc"));
    assert!(!Matcher::match_str(&parse_pattern("^\\d\\d").unwrap(), "a12bc"));
}

#[test]
fn test_end_of_line() {
    assert!(Matcher::match_str(&parse_pattern("cat$").unwrap(), "cat"));
    assert!(!Matcher::match_str(&parse_pattern("cat$").unwrap(), "cats"));
    assert!(Matcher::match_str(&parse_pattern("cat$").unwrap(), "a cat"));
    assert!(Matcher::match_str(&parse_pattern("^cat$").unwrap(), "cat"));
    assert!(!Matcher::match_str(&parse_pattern("^cat$").unwrap(), "a cat"));
}

#[test]
fn test_one_or_more() {
    assert!(Matcher::match_str(&parse_pattern("a+").unwrap(), "a"));
    assert!(Matcher::match_str(&parse_pattern("a+").unwrap(), "aa"));
    assert!(!Matcher::match_str(&parse_pattern("a+").unwrap(), ""));
    assert!(!Matcher::match_str(&parse_pattern("a+").unwrap(), "b"));
    assert!(Matcher::match_str(&parse_pattern("ca+ts").unwrap(), "caats"));
    assert!(Matcher::match_str(&parse_pattern("ca+ts").unwrap(), "cats"));
    assert!(!Matcher::match_str(&parse_pattern("ca+ts").unwrap(), "cts"));
}

#[test]
fn test_zero_or_one() {
    assert!(Matcher::match_str(&parse_pattern("dogs?").unwrap(), "dogs"));
    assert!(Matcher::match_str(&parse_pattern("dogs?").unwrap(), "dog"));
    assert!(!Matcher::match_str(&parse_pattern("dogs?").unwrap(), "dogss"));
    assert!(!Matcher::match_str(&parse_pattern("dogs?").unwrap(), "cat"));
    assert!(Matcher::match_str(&parse_pattern("colou?r").unwrap(), "color"));
    assert!(Matcher::match_str(&parse_pattern("colou?r").unwrap(), "colour"));
}

#[test]
fn test_alternation() {
    assert!(Matcher::match_str(&parse_pattern("(cat|dog)").unwrap(), "cat"));
    assert!(Matcher::match_str(&parse_pattern("(cat|dog)").unwrap(), "dog"));
    assert!(!Matcher::match_str(&parse_pattern("(cat|dog)").unwrap(), "apple"));
    assert!(Matcher::match_str(&parse_pattern("a(b|c)d").unwrap(), "abd"));
    assert!(Matcher::match_str(&parse_pattern("a(b|c)d").unwrap(), "acd"));
    assert!(!Matcher::match_str(&parse_pattern("a(b|c)d").unwrap(), "ad"));
}

#[test]
fn test_backreferences() {
    assert!(Matcher::match_str(&parse_pattern("(cat) and \\1").unwrap(), "cat and cat"));
    assert!(!Matcher::match_str(&parse_pattern("(cat) and \\1").unwrap(), "cat and dog"));
    assert!(Matcher::match_str(&parse_pattern("(\\w+) and \\1").unwrap(), "cat and cat"));
    assert!(Matcher::match_str(&parse_pattern("(\\w+) and \\1").unwrap(), "dog and dog"));
    assert!(!Matcher::match_str(&parse_pattern("(\\w+) and \\1").unwrap(), "cat and dog"));
    assert!(Matcher::match_str(&parse_pattern("(\\w+)\\s+\\1").unwrap(), "hello hello"));
    assert!(!Matcher::match_str(&parse_pattern("(\\w+)\\s+\\1").unwrap(), "hello world"));
    assert!(Matcher::match_str(&parse_pattern("(\\d+)-(\\w+)-(\\d+)\\s+\\3-\\2-\\1").unwrap(), "123-abc-456 456-abc-123"));
    assert!(!Matcher::match_str(&parse_pattern("(\\d+)-(\\w+)-(\\d+)\\s+\\3-\\2-\\1").unwrap(), "123-abc-456 456-def-123"));
}

#[test]
fn test_multiple_backreferences() {
    assert!(Matcher::match_str(&parse_pattern("(\\d+) (\\w+) squares and \\1 \\2 circles").unwrap(),
        "3 red squares and 3 red circles"));
    assert!(!Matcher::match_str(&parse_pattern("(\\d+) (\\w+) squares and \\1 \\2 circles").unwrap(),
        "3 red squares and 4 red circles"));
    assert!(Matcher::match_str(&parse_pattern("(\\w+) (\\w+) (\\w+) and \\3 \\2 \\1").unwrap(),
        "one two three and three two one"));
    assert!(!Matcher::match_str(&parse_pattern("(\\w+) (\\w+) (\\w+) and \\3 \\2 \\1").unwrap(),
        "one two three and three one two"));
}

#[test]
fn test_nested_backreferences() {
    assert!(Matcher::match_str(&parse_pattern("('(cat) and \\2') is the same as \\1").unwrap(),
        "'cat and cat' is the same as 'cat and cat'"));
    assert!(!Matcher::match_str(&parse_pattern("('(cat) and \\2') is the same as \\1").unwrap(),
        "'cat and dog' is the same as 'cat and dog'"));
}