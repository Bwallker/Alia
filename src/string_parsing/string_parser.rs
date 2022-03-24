use util::misc_extension_traits::*;

#[derive(Debug, Eq, PartialOrd, PartialEq, Ord, Hash, Clone)]
pub enum StringParseErrorCode {
    StringWithoutOpeningQuote,
    InvalidString,
    EmptyString,
    StringWithoutClosingQuote,
}

use crate::util;
use std::fmt::{Display, Formatter};
use StringParseErrorCode::*;

impl Display for StringParseErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", string_parse_error_to_string(self))
    }
}

fn string_parse_error_to_string(err: &StringParseErrorCode) -> String {
    match err {
    StringWithoutOpeningQuote => "Your string did not contain an opening quote.",
    InvalidString => "Your string consisted of only an opening parenthesis and nothing else.",
    EmptyString => "Your string was empty. This is not allowed as all aliases must have names and values that are not empty.",
    StringWithoutClosingQuote => "Your string did not contain a closing quote.",
  }.to_string()
}

pub fn parse_string(slice: &mut &str) -> Result<String, StringParseErrorCode> {
    parse_string_track_lines(slice, &mut 0)
}

pub fn parse_string_track_lines(
    slice: &mut &str,
    current_line: &mut u32,
) -> Result<String, StringParseErrorCode> {
    *slice = slice.trim_start();
    if slice.pop_char() != Some('"') {
        return Err(StringWithoutOpeningQuote);
    }
    let ret_slice: &str = *slice;
    let current_char = slice.chars().next();
    if let None = current_char {
        return Err(InvalidString);
    }
    let mut current_char = unsafe { current_char.unwrap_unchecked() };
    let mut index = 1;
    let mut backslashes_in_a_row: usize = 0;
    loop {
        if current_char == '\n' {
            *current_line += 1;
        }
        if current_char == '"' && backslashes_in_a_row % 2 == 0 {
            slice.pop_char();

            break;
        }
        if current_char == '\\' {
            backslashes_in_a_row += 1;
        } else {
            backslashes_in_a_row = 0;
        }
        slice.pop_char();
        let current_char_option = slice.chars().next();
        if let None = current_char_option {
            return Err(StringWithoutClosingQuote);
        }
        current_char = unsafe { current_char_option.unwrap_unchecked() };
        index += 1;
    }
    let mut ret = String::with_capacity(index - 1);
    let mut iter = ret_slice.chars();
    let mut last_char = '\0';

    for _ in 0..index - 1 {
        let char = unsafe { iter.next().unwrap_unchecked() };
        if last_char == '\\' && char == '"' {
            ret.pop();
        }
        ret.push(char);
        last_char = char;
    }

    if ret.len() == 0 {
        return Err(EmptyString);
    }
    ret.shrink_to_fit();
    Ok(ret)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_without_opening_quote() {
        assert_eq!(
            parse_string(&mut "dsasd\"dsadaasdfdsd\"dsaadsasd"),
            Err(StringWithoutOpeningQuote)
        );
    }
    #[test]
    fn test_successful_string_parse() {
        assert_eq!(
            parse_string(&mut "\"Snail\"abcdef\"PENIS\""),
            Ok("Snail".to_string())
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string(&mut "\"Snail\""), Ok("Snail".to_string()));
        assert_eq!(
            parse_string(&mut "\"S\\\"nail\""),
            Ok("S\"nail".to_string())
        );
        assert_eq!(parse_string(&mut "\"SSS"), Err(StringWithoutClosingQuote));

        let string = String::from("\"Snail\"");
        let slice = &mut string.as_str();
        let parsed = parse_string(slice);
        assert_eq!(parsed, Ok("Snail".into()));
        assert_eq!(slice, &"");
        util("\"Snail\"", "", Ok("Snail".into()));
        util(
            "\"HAAAAAA\"some more stuff",
            "some more stuff",
            Ok("HAAAAAA".into()),
        );
    }

    fn util(
        input: &'static str,
        expected_slice_at_end: &'static str,
        expected_result: Result<String, StringParseErrorCode>,
    ) {
        let string = String::from(input);
        let slice = &mut string.as_str();
        let parsed = parse_string(slice);
        assert_eq!(parsed, expected_result);
        assert_eq!(slice, &expected_slice_at_end);
    }
}
