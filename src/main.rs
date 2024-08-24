use std::env;
use std::io;
use std::process;

// check if the input line matches the pattern
fn match_pattern(input_line: &str, pattern: &str) -> bool {
    match pattern {
        s if s.chars().count() == 1 => input_line.contains(pattern),
        r#"\d"# => input_line.chars().any(|c| c.is_digit(10)),
        r#"\w"# => input_line.chars().any(|c| c.is_ascii_alphanumeric() || c == '_'),
        s if s.starts_with("[^") && s.ends_with(']') => {
            let char_group = &pattern[2..pattern.len() - 1];
            input_line.chars().any(|c| !char_group.contains(c))
        }
        s if s.starts_with('[') && s.ends_with(']') => {
            let char_group = &pattern[1..pattern.len() - 1];
            input_line.chars().any(|c| char_group.contains(c))
        }
        _ => panic!("Unhandled pattern: {}", pattern),
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // Check if the first argument is '-E'
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    // Get the pattern from the second argument
    let pattern = env::args().nth(2).unwrap();

    // Define a mutable string to store the input line
    let mut input_line = String::new();

    // Read the input line from stdin
    io::stdin().read_line(&mut input_line).unwrap();

    // Trim the input line to remove any trailing newline
    let input_line = input_line.trim();

    // Check if the input line matches the pattern
    if match_pattern(input_line, &pattern) {
        println!("Pattern matches!");
        process::exit(0)
    } else {
        println!("Pattern does not match.");
        process::exit(1)
    }
}