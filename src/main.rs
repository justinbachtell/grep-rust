use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() == 1 {
        return input_line.contains(pattern);
    } else if pattern == "\\d" {
        return input_line.chars().any(|c| c.is_digit(10));
    } else {
        panic!("Unhandled pattern: {}", pattern)
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

    // Check if the input line matches the pattern
    if match_pattern(&input_line, &pattern) {
        println!("Pattern matches!");
        process::exit(0)
    } else {
        println!("Pattern does not match.");
        process::exit(1)
    }
}
