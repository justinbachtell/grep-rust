use std::env;
use std::io::{self, Read};
use std::process;
use std::str::FromStr;
use codecrafters_grep::Pattern;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    // Check if the first argument is '-E'
    if env::args().nth(1).unwrap() != "-E" {
        eprintln!("Expected first argument to be '-E'");
        process::exit(1);
    }

    // Get the pattern from the second argument
    let pattern_str = env::args().nth(2).expect("No pattern provided");
    log::debug!("Pattern string: {:?}", pattern_str);
    let pattern = Pattern::from_str(&pattern_str).expect("Invalid pattern");
    log::debug!("Parsed pattern: {:?}", pattern);

    // Read input line by line
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    // Remove trailing newline if present
    let input = input.trim_end();
    log::debug!("Input: {:?}", input);

    let has_match = pattern.match_str(input);
    log::debug!("Match result: {}", has_match);
    if has_match {
        println!("Pattern matches!");
        process::exit(0);
    } else {
        println!("Pattern does not match.");
        process::exit(1);
    }
}
