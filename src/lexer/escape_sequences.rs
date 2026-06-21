use std::fmt::Display;
use std::iter::Peekable;
use std::str::Chars;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct CharList(pub Vec<CharType>);

impl Display for CharList {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        for char_type in &self.0 {
            output.push_str(&char_type.to_string());
        }

        write!(display, "{output}")
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum CharType {
    Char { value: u32 },
    Octal { value: u32 },
    Hex { value: u64 },
    SmallUnicode { value: u32 },
    LargeUnicode { value: u32 },
    SingleQuote,
    DoubleQuote,
    QuestionMark,
    Backslash,
    Alert,
    Backspace,
    FormFeed,
    NewLine,
    CarriageReturn,
    HorizontalTab,
    VerticalTab,
}

// a char can be initalized as multiple character like 'AB'
// its implementation defined though I still think im doing it a fairly "standard" way
// yet another legacy feature almost no one uses yet still exists
impl FromStr for CharType {
    type Err = String;

    fn from_str(multi_chars: &str) -> Result<Self, Self::Err> {
        let split_chars = split_string(multi_chars)?;

        let mut final_value: u32 = 0;

        for (i, char) in split_chars.0.iter().enumerate() {
            let shift = (split_chars.0.len() - (i + 1)) * 8;

            // 32 bits - 8 bits per char means that we cannot shift more than
            // 24 bits without a panic
            if shift > (32 - 8) {
                continue;
            }

            final_value |= (char.get_value() << shift) as u32;
        }

        // if this number repersents a special value (such as a newline) use that specific enum
        let found_self = Self::MAPPINGS
            .iter()
            .find(|(x, _)| *x == final_value)
            .map(|(_, x)| x);

        if let Some(special_type) = found_self.cloned() {
            return Ok(special_type);
        }

        // otherwise just give it a generic char value
        Ok(Self::Char { value: final_value })
    }
}

impl CharType {
    const MAPPINGS: &'static [(u32, Self); 11] = &[
        (39, CharType::SingleQuote),
        (34, CharType::DoubleQuote),
        (63, CharType::QuestionMark),
        (92, CharType::Backslash),
        (7, CharType::Alert),
        (8, CharType::Backspace),
        (12, CharType::FormFeed),
        (10, CharType::NewLine),
        (13, CharType::CarriageReturn),
        (9, CharType::HorizontalTab),
        (11, CharType::VerticalTab),
    ];

    fn get_value(&self) -> u64 {
        let found_value = Self::MAPPINGS
            .iter()
            .find(|(_, x)| x == self)
            .map(|(x, _)| x);

        if let Some(value) = found_value.cloned() {
            return value as u64;
        }

        match &self {
            CharType::Char { value }
            | CharType::Octal { value }
            | CharType::SmallUnicode { value }
            | CharType::LargeUnicode { value } => *value as u64,

            CharType::Hex { value } => *value,

            _ => {
                unreachable!()
            }
        }
    }
}

impl Display for CharType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = match self {
            CharType::Char { value } => match char::from_u32(*value) {
                Some(x) => x.to_string(),
                None => {
                    format!("\\x{:x}", value)
                }
            },
            CharType::Octal { value } => format!("\\{:o}", value),
            CharType::Hex { value } => format!("\\x{:x}", value),
            CharType::SmallUnicode { value } => format!("\\u{:x}", value),
            CharType::LargeUnicode { value } => format!("\\U{:x}", value),

            CharType::SingleQuote => "\\'".to_string(),
            CharType::DoubleQuote => "\\\"".to_string(),
            CharType::QuestionMark => "\\?".to_string(),
            CharType::Backslash => "\\\\".to_string(),
            CharType::Alert => "\\a".to_string(),
            CharType::Backspace => "\\b".to_string(),
            CharType::FormFeed => "\\f".to_string(),
            CharType::NewLine => "\\n".to_string(),
            CharType::CarriageReturn => "\\r".to_string(),
            CharType::HorizontalTab => "\\t".to_string(),
            CharType::VerticalTab => "\\v".to_string(),
        };

        write!(display, "{final_str}")
    }
}

fn parse_unicode_sequence<const NUM_DIGITS: usize>(
    chars: &mut Peekable<Chars<'_>>,
) -> Result<CharType, String> {
    chars.next();
    let mut final_unicode_str = String::new();

    while let Some(char) = chars.peek() {
        if final_unicode_str.len() == NUM_DIGITS || !char.is_digit(16) {
            break;
        }

        final_unicode_str.push(*char);
        chars.next();
    }

    if final_unicode_str.len() != NUM_DIGITS {
        return Err(format!(
            "Expected unicode sequence to have {NUM_DIGITS} digits got {}",
            final_unicode_str.len()
        ));
    }

    let as_u32 = u32::from_str_radix(&final_unicode_str, 16).unwrap();

    // the specification also says something about unicode values that are valid
    // but I can't be bothered to figure out what it actually means

    match NUM_DIGITS {
        4 => Ok(CharType::SmallUnicode { value: as_u32 }),
        8 => Ok(CharType::LargeUnicode { value: as_u32 }),
        _ => unreachable!(),
    }
}

fn parse_octal_sequence(chars: &mut Peekable<Chars<'_>>) -> Result<CharType, String> {
    let mut final_octal_string = String::new();

    while let Some(char) = chars.peek() {
        // greater lengths are not allowed for some reason
        if final_octal_string.len() == 3 || !char.is_digit(8) {
            break;
        }

        final_octal_string.push(*char);

        chars.next();
    }

    if final_octal_string.is_empty() {
        return Err(String::from(
            "Expected octal sequence to have an octal digit, found nothing",
        ));
    }

    let as_u32 = u32::from_str_radix(&final_octal_string, 8).unwrap();

    Ok(CharType::Octal { value: as_u32 })
}

fn parse_hex_sequence(chars: &mut Peekable<Chars<'_>>) -> Result<CharType, String> {
    chars.next(); // move past the x

    let mut final_hex_string = String::new();

    while let Some(char) = chars.peek() {
        if !char.is_digit(16) {
            break;
        }

        final_hex_string.push(*char);
        chars.next();
    }

    if final_hex_string.is_empty() {
        return Err(String::from(
            "Expected hex sequence to have a hex digit, found nothing",
        ));
    }

    // it takes 16 digits to reach 64 bits, anything after that is truncated
    let max_length = final_hex_string.len().min(16);
    let as_u64 = u64::from_str_radix(&final_hex_string[..max_length], 16).unwrap();

    Ok(CharType::Hex { value: as_u64 })
}

fn parse_escape_sequence(chars: &mut Peekable<Chars<'_>>) -> Result<CharType, String> {
    let Some(next_char) = chars.peek().cloned() else {
        return Err(String::from(
            "Escape sequence expected to have subseqeuent character",
        ));
    };

    let char_type = match next_char {
        '\'' => CharType::SingleQuote,
        '"' => CharType::DoubleQuote,
        '?' => CharType::QuestionMark,
        '\\' => CharType::Backslash,
        'a' => CharType::Alert,
        'b' => CharType::Backspace,
        'f' => CharType::FormFeed,
        'n' => CharType::NewLine,
        'r' => CharType::CarriageReturn,
        't' => CharType::HorizontalTab,
        'v' => CharType::VerticalTab,
        'x' => parse_hex_sequence(chars)?,
        'u' => parse_unicode_sequence::<4>(chars)?,
        'U' => parse_unicode_sequence::<8>(chars)?,
        x if x.is_digit(8) => parse_octal_sequence(chars)?,

        _ => {
            return Err(format!(
                "Unknown escape sequence with character {next_char}"
            ));
        }
    };

    match char_type {
        CharType::SingleQuote
        | CharType::DoubleQuote
        | CharType::QuestionMark
        | CharType::Backslash
        | CharType::Alert
        | CharType::Backspace
        | CharType::FormFeed
        | CharType::NewLine
        | CharType::CarriageReturn
        | CharType::HorizontalTab
        | CharType::VerticalTab => {
            chars.next();
        }
        _ => {}
    };

    Ok(char_type)
}

pub fn split_string(string: &str) -> Result<CharList, String> {
    let mut final_string = Vec::new();

    let mut all_chars = string.chars().peekable();
    while let Some(char) = all_chars.next() {
        if char == '\\' {
            final_string.push(parse_escape_sequence(&mut all_chars)?);
        } else {
            let char_value = char as u32;
            let char_type = CharType::Char { value: char_value };
            final_string.push(char_type);
        }
    }

    Ok(CharList(final_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_escape_sequences() {
        let test_cases = vec![r#"\x"#, r#"\u123"#, r#"\8"#];

        for test_case in test_cases {
            let result = split_string(test_case);

            assert!(result.is_err());
        }
    }

    #[test]
    fn test_valid_escape_sequences() {
        let test_cases = vec![
            (r#""#, vec![]),
            (r#"A"#, vec![CharType::Char { value: 0x41 }]),
            (
                r#"A\nB"#,
                vec![
                    CharType::Char { value: 0x41 },
                    CharType::NewLine,
                    CharType::Char { value: 0x42 },
                ],
            ),
            (
                r#"\a\b\f\n\r\t\v"#,
                vec![
                    CharType::Alert,
                    CharType::Backspace,
                    CharType::FormFeed,
                    CharType::NewLine,
                    CharType::CarriageReturn,
                    CharType::HorizontalTab,
                    CharType::VerticalTab,
                ],
            ),
            (r#"\\"#, vec![CharType::Backslash]),
            (r#"\'"#, vec![CharType::SingleQuote]),
            (r#"\""#, vec![CharType::DoubleQuote]),
            (r#"\?"#, vec![CharType::QuestionMark]),
            (r#"\0"#, vec![CharType::Octal { value: 0 }]),
            (r#"\07"#, vec![CharType::Octal { value: 7 }]),
            (r#"\123"#, vec![CharType::Octal { value: 83 }]),
            (
                r#"\1234"#,
                vec![
                    CharType::Octal { value: 83 },
                    CharType::Char { value: 0x34 },
                ],
            ),
            (r#"\377"#, vec![CharType::Octal { value: 255 }]),
            (r#"\x41"#, vec![CharType::Hex { value: 0x41 }]),
            (r#"\x41B"#, vec![CharType::Hex { value: 0x41B }]),
            (
                r#"\x41G"#,
                vec![
                    CharType::Hex { value: 0x41 },
                    CharType::Char { value: 0x47 },
                ],
            ),
            (r#"\x123abc"#, vec![CharType::Hex { value: 0x123ABC }]),
            (r#"\u0041"#, vec![CharType::SmallUnicode { value: 0x41 }]),
            (r#"\u00E9"#, vec![CharType::SmallUnicode { value: 0xE9 }]),
            (
                r#"\U0001F600"#,
                vec![CharType::LargeUnicode { value: 0x1F600 }],
            ),
            (
                r#"\U00000041"#,
                vec![CharType::LargeUnicode { value: 0x41 }],
            ),
            (
                r#"mix:\t\x41\101\u0042\U0001F600"#,
                vec![
                    CharType::Char { value: 0x6D },
                    CharType::Char { value: 0x69 },
                    CharType::Char { value: 0x78 },
                    CharType::Char { value: 0x3A },
                    CharType::HorizontalTab,
                    CharType::Hex { value: 0x41 },
                    CharType::Octal { value: 65 },
                    CharType::SmallUnicode { value: 0x42 },
                    CharType::LargeUnicode { value: 0x1F600 },
                ],
            ),
        ];

        for (test_case, correct_result) in test_cases {
            let result = split_string(test_case).unwrap().0;

            assert_eq!(result, correct_result);
        }
    }
}
