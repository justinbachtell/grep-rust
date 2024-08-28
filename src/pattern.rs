// Define the Pattern enum to represent different regex pattern elements
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Pattern {
    ExactChar(char),
    AnyChar,
    AlphaNumeric,
    Sequence(Vec<Pattern>),
    Repeated {
        min: usize,
        max: Option<usize>,
        pattern: Box<Pattern>,
    },
    OneOf(Vec<Pattern>),
    CharacterSet {
        chars: String,
        negated: bool,
    },
    StartOfLine,
    EndOfLine,
    OneOrMore(Box<Pattern>),
    ZeroOrOne(Box<Pattern>),
    Alternation(Vec<Pattern>),
    Backreference(usize),
    CaptureGroup(Box<Pattern>),
    NestedCapture(Box<Pattern>),
}