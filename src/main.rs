use std::env;
use std::io::{self, Read};
use std::process;
use std::str::FromStr;
use codecrafters_grep::Pattern;

fn main() {
    // Check if the first argument is '-E'
    if env::args().nth(1).unwrap() != "-E" {
        eprintln!("Expected first argument to be '-E'");
        process::exit(1);
    }

    // Get the pattern from the second argument
    let pattern_str = env::args().nth(2).expect("No pattern provided");
    let pattern = Pattern::from_str(&pattern_str).expect("Invalid pattern");

    // Read the entire input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    // Check if the input matches the pattern
    let matches = pattern.match_str(&input);

    if !matches.is_empty() {
        println!("Pattern matches!");
        process::exit(0);
    } else {
        println!("Pattern does not match.");
        process::exit(1);
    }
}