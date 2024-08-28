use crate::Pattern;
use std::str::FromStr;

#[test]
fn test_match_str_exact_char() {
    assert!(Pattern::from_str("A").unwrap().match_str("ABC"));
    assert!(!Pattern::from_str("X").unwrap().match_str("ABC"));
    assert!(Pattern::from_str("C").unwrap().match_str("C"));
}

#[test]
fn test_match_str_digit() {
    assert!(Pattern::from_str("\\d").unwrap().match_str("123"));
    assert!(!Pattern::from_str("\\d").unwrap().match_str("ABC"));
    assert!(Pattern::from_str("\\d").unwrap().match_str("9"));
}

#[test]
fn test_match_repeated() {
    assert!(Pattern::from_str("\\d{0,2}").unwrap().match_str("123"));
    assert!(Pattern::from_str("\\d{2,3}").unwrap().match_str("12345"));
    assert!(Pattern::from_str("\\d{2,}").unwrap().match_str("12345"));
    assert!(Pattern::from_str("\\d{2,}").unwrap().match_str("123ABC"));
}

#[test]
fn test_match_str_sequence() {
    assert!(Pattern::from_str("\\dZ\\d").unwrap().match_str("1Z2XY"));
}

#[test]
fn test_matches() {
    assert!(Pattern::from_str("AB\\d\\dZZ").unwrap().match_str("AB12ZZCD"));
    assert!(Pattern::from_str("..\\dA").unwrap().match_str("A12A"));
    assert!(Pattern::from_str(".*foo").unwrap().match_str("foobar"));
    assert!(Pattern::from_str(".*foo").unwrap().match_str("somefoobar"));
    assert!(Pattern::from_str(".*ZZ.*X").unwrap().match_str("ABCZZZ12XX"));
    assert!(Pattern::from_str("[abc]*test").unwrap().match_str("aabbcatest12"));
    assert!(Pattern::from_str("[^xyz]*xtest").unwrap().match_str("aabbcaxtest12"));
    assert!(Pattern::from_str("[^xyz]*test").unwrap().match_str("aabbcatest12"));
    assert!(Pattern::from_str("\\d apple").unwrap().match_str("1 apple"));
}

#[test]
fn test_any_char() {
    assert!(Pattern::from_str(".").unwrap().match_str("ABC"));
    assert!(Pattern::from_str(".").unwrap().match_str("A"));
    assert!(!Pattern::from_str(".").unwrap().match_str(""));
}

#[test]
fn test_alpha_numeric() {
    assert!(Pattern::from_str("\\w").unwrap().match_str("a123"));
    assert!(Pattern::from_str("\\w").unwrap().match_str("_abc"));
    assert!(Pattern::from_str("\\w").unwrap().match_str("9xyz"));
    assert!(!Pattern::from_str("\\w").unwrap().match_str("!abc"));
}

#[test]
fn test_one_of() {
    let pattern = Pattern::from_str("[a0-9x]").unwrap();
    assert!(pattern.match_str("abc"));
    assert!(pattern.match_str("123"));
    assert!(pattern.match_str("xyz"));
    assert!(!pattern.match_str("bcd"));
}

#[test]
fn test_character_set() {
    let pattern = Pattern::from_str("[aeiou]").unwrap();
    assert!(pattern.match_str("apple"));
    assert!(!pattern.match_str("xyz"));

    let negated_pattern = Pattern::from_str("[^aeiou]").unwrap();
    assert!(negated_pattern.match_str("xyz"));
    assert!(!negated_pattern.match_str("apple"));
}

#[test]
fn test_start_of_line() {
    assert!(Pattern::from_str("^log").unwrap().match_str("log"));
    assert!(!Pattern::from_str("^log").unwrap().match_str("slog"));
    assert!(Pattern::from_str("^\\d\\d").unwrap().match_str("12abc"));
    assert!(!Pattern::from_str("^\\d\\d").unwrap().match_str("a12bc"));
}

#[test]
fn test_end_of_line() {
    assert!(Pattern::from_str("cat$").unwrap().match_str("cat"));
    assert!(!Pattern::from_str("cat$").unwrap().match_str("cats"));
    assert!(Pattern::from_str("cat$").unwrap().match_str("a cat"));
    assert!(Pattern::from_str("^cat$").unwrap().match_str("cat"));
    assert!(!Pattern::from_str("^cat$").unwrap().match_str("a cat"));
}

#[test]
fn test_one_or_more() {
    assert!(Pattern::from_str("a+").unwrap().match_str("a"));
    assert!(Pattern::from_str("a+").unwrap().match_str("aa"));
    assert!(!Pattern::from_str("a+").unwrap().match_str(""));
    assert!(!Pattern::from_str("a+").unwrap().match_str("b"));
    assert!(Pattern::from_str("ca+ts").unwrap().match_str("caats"));
    assert!(Pattern::from_str("ca+ts").unwrap().match_str("cats"));
    assert!(!Pattern::from_str("ca+ts").unwrap().match_str("cts"));
}

#[test]
fn test_zero_or_one() {
    assert!(Pattern::from_str("dogs?").unwrap().match_str("dogs"));
    assert!(Pattern::from_str("dogs?").unwrap().match_str("dog"));
    assert!(!Pattern::from_str("dogs?").unwrap().match_str("dogss"));
    assert!(!Pattern::from_str("dogs?").unwrap().match_str("cat"));
    assert!(Pattern::from_str("colou?r").unwrap().match_str("color"));
    assert!(Pattern::from_str("colou?r").unwrap().match_str("colour"));
}

#[test]
fn test_alternation() {
    assert!(Pattern::from_str("(cat|dog)").unwrap().match_str("cat"));
    assert!(Pattern::from_str("(cat|dog)").unwrap().match_str("dog"));
    assert!(!Pattern::from_str("(cat|dog)").unwrap().match_str("apple"));
    assert!(Pattern::from_str("a(b|c)d").unwrap().match_str("abd"));
    assert!(Pattern::from_str("a(b|c)d").unwrap().match_str("acd"));
    assert!(!Pattern::from_str("a(b|c)d").unwrap().match_str("ad"));
}

#[test]
fn test_backreferences() {
    assert!(Pattern::from_str("(cat) and \\1").unwrap().match_str("cat and cat"));
    assert!(!Pattern::from_str("(cat) and \\1").unwrap().match_str("cat and dog"));
    assert!(Pattern::from_str("(\\w+) and \\1").unwrap().match_str("cat and cat"));
    assert!(Pattern::from_str("(\\w+) and \\1").unwrap().match_str("dog and dog"));
    assert!(!Pattern::from_str("(\\w+) and \\1").unwrap().match_str("cat and dog"));
    assert!(Pattern::from_str("(\\w+)\\s+\\1").unwrap().match_str("hello hello"));
    assert!(!Pattern::from_str("(\\w+)\\s+\\1").unwrap().match_str("hello world"));
    assert!(Pattern::from_str("(\\d+)-(\\w+)-(\\d+)\\s+\\3-\\2-\\1").unwrap().match_str("123-abc-456 456-abc-123"));
    assert!(!Pattern::from_str("(\\d+)-(\\w+)-(\\d+)\\s+\\3-\\2-\\1").unwrap().match_str("123-abc-456 456-def-123"));
}

#[test]
fn test_multiple_backreferences() {
    assert!(Pattern::from_str("(\\d+) (\\w+) squares and \\1 \\2 circles").unwrap()
        .match_str("3 red squares and 3 red circles"));
    assert!(!Pattern::from_str("(\\d+) (\\w+) squares and \\1 \\2 circles").unwrap()
        .match_str("3 red squares and 4 red circles"));
    assert!(Pattern::from_str("(\\w+) (\\w+) (\\w+) and \\3 \\2 \\1").unwrap()
        .match_str("one two three and three two one"));
    assert!(!Pattern::from_str("(\\w+) (\\w+) (\\w+) and \\3 \\2 \\1").unwrap()
        .match_str("one two three and three one two"));
}

#[test]
fn test_nested_backreferences() {
    assert!(Pattern::from_str("('(cat) and \\2') is the same as \\1").unwrap()
        .match_str("'cat and cat' is the same as 'cat and cat'"));
    assert!(!Pattern::from_str("('(cat) and \\2') is the same as \\1").unwrap()
        .match_str("'cat and dog' is the same as 'cat and dog'"));
}