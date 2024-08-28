mod pattern;
mod parser;
mod matcher;

pub use pattern::Pattern;
pub use parser::parse_pattern;
pub use matcher::Matcher;

use std::env;
use std::io::{self, Read};
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    // Check if the first argument is '-E'
    if env::args().nth(1).ok_or("No arguments provided")? != "-E" {
        return Err("Expected first argument to be '-E'".into());
    }

    // Get the pattern from the second argument
    let pattern_str = env::args().nth(2).ok_or("No pattern provided")?;
    log::debug!("Pattern string: {:?}", pattern_str);
    let pattern = parse_pattern(&pattern_str)?;
    log::debug!("Parsed pattern: {:?}", pattern);

    // Read input
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Remove trailing newline if present
    let input = input.trim_end();
    log::debug!("Input: {:?}", input);

    let has_match = Matcher::match_str(&pattern, input);
    log::debug!("Match result: {}", has_match);
    
    if has_match {
        println!("Pattern matches!");
        Ok(())
    } else {
        Err("Pattern does not match.".into())
    }
}

#[cfg(test)]
mod tests {
    mod matcher_tests;
    mod parser_tests;
    mod pattern_tests;
}