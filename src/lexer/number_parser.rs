use crate::lexer::{language_features::LiteralTypes, lexer::TokenTypes};
use std::{fmt::Display, iter::Peekable, str::Chars};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct StoredSize {
    // ints
    pub is_unsigned: bool,
    pub is_long: bool,
    pub is_long_long: bool,
    // floats
    pub is_float: bool,
    pub is_double: bool,
    pub is_long_double: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct IntType {
    pub value: i128,
    pub storage: StoredSize,
}

impl Display for IntType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from(self.value.to_string());

        if self.storage.is_unsigned {
            output.push('u');
        }

        if self.storage.is_long {
            output.push('l');
        }

        if self.storage.is_long_long {
            output.push('l');
        }

        write!(display, "{output}")
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct FloatType {
    pub value: f64,
    pub storage: StoredSize,
}

impl Display for FloatType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from(self.value.to_string());

        if self.storage.is_float {
            output.push('f');
        }

        if self.storage.is_long_double {
            output.push('l');
        }

        write!(display, "{output}")
    }
}

// Ignores complex and imaginary numbers for the time being
// this might be added in the future

#[derive(Debug)]
enum PrefixTypes {
    Decimal,
    Hexadecimal,
    Octal,
    Binary,
    // binary is a compiler extension, though its so widely used
    // (GCC, Clang, MSVC), it may as well be part of the standard
}

fn get_numerical_string(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut final_str = String::new();

    let mut is_hex = false;
    let mut previous_was_exponent = false;

    while let Some(char) = chars.peek().cloned() {
        let lowercase_char = char.to_ascii_lowercase();

        if lowercase_char == 'x' {
            is_hex = true;
        }

        if !(lowercase_char == 'x' // hex
            || lowercase_char == 'b'// binary
            || lowercase_char == 'e'// exponent (or hex char)
            || lowercase_char == 'p'// hex float exponenet
            || lowercase_char == 'u'// unsigned
            || lowercase_char == 'l'// long
            || lowercase_char == 'f'// float
            || lowercase_char == '.'// float decimal
            || lowercase_char.is_digit(16)
            || (char == '+' || char == '-') && previous_was_exponent)
        // dont accept + or - unless we know its part of an exponent
        // not it will just blindly take expressions
        {
            break;
        }

        final_str.push(char);
        chars.next();

        if lowercase_char == 'e' && !is_hex || lowercase_char == 'p' && is_hex {
            previous_was_exponent = true;
        }
    }

    final_str
}

fn get_prefix(numerical_str: &str) -> PrefixTypes {
    let mut prefix_type = PrefixTypes::Decimal;

    if numerical_str.starts_with('0') {
        if let Some(next_char) = numerical_str.chars().nth(1) {
            match next_char {
                'b' => prefix_type = PrefixTypes::Binary,
                'x' => prefix_type = PrefixTypes::Hexadecimal,
                _ => prefix_type = PrefixTypes::Octal,
            }
        }
    }

    return prefix_type;
}

fn parse_floating_point(
    mut numerical_str: String,
    is_hex_float: bool,
) -> Result<LiteralTypes, String> {
    let is_float = numerical_str.ends_with("f");
    let is_long_double = numerical_str.ends_with("l");
    let is_double = !is_float && !is_long_double;

    numerical_str = numerical_str.trim_end_matches("l").to_string();
    numerical_str = numerical_str.trim_end_matches("f").to_string();

    let stored_size = StoredSize {
        is_float,
        is_double,
        is_long_double,
        ..Default::default()
    };

    if is_hex_float {
        return parse_hex_floating_point(numerical_str, stored_size);
    }

    return parse_decimal_floating_point(numerical_str, stored_size);
}

// finally my 10th grade knowledge coming in useful
// love you Mr Ramones
fn base_16_to_base_10(hex: &str) -> Result<f64, String> {
    let split_at_point = hex.split('.').collect::<Vec<&str>>();
    let base = split_at_point[0];

    let mut final_value = 0_f64;

    for (i, char) in base.chars().enumerate() {
        let Some(num) = char.to_digit(16) else {
            return Err(format!("Expected hex float base part to be a hex number"));
        };

        let exponent = base.len() - i - 1;
        final_value += num as f64 * 16_f64.powi(exponent as i32);
    }

    if split_at_point.len() == 1 {
        return Ok(final_value);
    }

    let fractional = split_at_point[1];

    for (i, char) in fractional.chars().enumerate() {
        let Some(num) = char.to_digit(16) else {
            return Err(format!(
                "Expected hex float fractional part to be a hex number"
            ));
        };

        let exponent = -(i as i32) - 1;

        final_value += num as f64 * 16_f64.powi(exponent);
    }

    println!("{final_value}");

    Ok(final_value)
}

fn parse_hex_floating_point(
    numerical_str: String,
    stored_size: StoredSize,
) -> Result<LiteralTypes, String> {
    let split_at_exponent: Vec<String> = numerical_str
        .split('p')
        .collect::<Vec<&str>>()
        .iter()
        .map(|x| x.to_string())
        .collect();

    if split_at_exponent.len() != 2 {
        return Err(format!(
            "Expected hex float to have corresponding exponent value",
        ));
    }

    let converted_base_value = base_16_to_base_10(&split_at_exponent[0])?;

    let exponent_value = match i64::from_str_radix(&split_at_exponent[1], 10) {
        Ok(value) => value,
        Err(_) => {
            return Err(format!(
                "Exponent value of hex float {} was invalid",
                split_at_exponent[1]
            ));
        }
    };

    let new_float_value = converted_base_value * f64::from(2).powi(exponent_value as i32);

    return Ok(LiteralTypes::Float(FloatType {
        value: new_float_value,
        storage: stored_size,
    }));
}

fn parse_decimal_floating_point(
    numerical_str: String,
    storage: StoredSize,
) -> Result<LiteralTypes, String> {
    let err_msg = format!("Failed to parse numeric literal");

    // we are stil using a f64 for a long double since rust's f128's are incomplete
    // may use a library and switch to a f64 at a future date
    match numerical_str.parse::<f64>() {
        Ok(value) => {
            return Ok(LiteralTypes::Float(FloatType { value, storage }));
        }
        Err(_) => return Err(err_msg),
    };
}

pub fn parse_number_literal(chars: &mut Peekable<Chars<'_>>) -> Result<TokenTypes, String> {
    let mut numerical_str = get_numerical_string(chars);
    if numerical_str.contains("lL") || numerical_str.contains("lL") {
        return Err(String::from(
            "String literal with long long suffix must have the same case",
        ));
    }

    numerical_str = numerical_str.to_ascii_lowercase();

    let mut prefix_type = get_prefix(&numerical_str);

    let is_hex_float = numerical_str.contains('p');
    let is_decimal_float = (numerical_str.contains('e') || numerical_str.contains('.'))
        && !matches!(prefix_type, PrefixTypes::Hexadecimal);
    // its still valid for a decimal float to have an octal prefix if its a float (e.g., 0789e10)
    let decimal_float_has_invalid_prefix = is_decimal_float
        && (matches!(prefix_type, PrefixTypes::Hexadecimal)
            || matches!(prefix_type, PrefixTypes::Binary));

    let hex_float_has_invalid_prefix =
        is_hex_float && !matches!(prefix_type, PrefixTypes::Hexadecimal);

    if decimal_float_has_invalid_prefix || hex_float_has_invalid_prefix {
        return Err(String::from("Given invalid numerical literal"));
    }

    // making sure it doesnt mistake it as an octal if it has a leading zero
    if is_decimal_float {
        prefix_type = PrefixTypes::Decimal;
    }

    let radix: u32;

    match prefix_type {
        PrefixTypes::Hexadecimal => {
            numerical_str = numerical_str.trim_start_matches("0x").to_string();
            radix = 16;
        }

        PrefixTypes::Binary => {
            numerical_str = numerical_str.trim_start_matches("0b").to_string();
            radix = 2;
        }

        PrefixTypes::Octal => {
            numerical_str = numerical_str.trim_start_matches("0").to_string();
            radix = 8;
        }

        PrefixTypes::Decimal => {
            radix = 10;
        }
    }

    if is_hex_float || is_decimal_float {
        return Ok(TokenTypes::Literal(parse_floating_point(
            numerical_str,
            is_hex_float,
        )?));
    }

    let mut is_unsigned = false;
    let mut is_long = false;
    let mut is_long_long = false;

    let numerical_str_copy = numerical_str.clone();
    let mut final_chars = numerical_str_copy.chars().rev();

    // might be cleaner if I nest but I hate nesting
    while let Some(char) = final_chars.next()
        && (char == 'l' || char == 'u')
    {
        if char == 'l' && is_long_long {
            return Err(format!("Invalid suffix on integer literal"));
        }

        if char == 'l' && is_long {
            is_long_long = true;
        }

        if char == 'l' {
            is_long = true;
        }

        if char == 'u' && is_unsigned {
            return Err(format!("Invalid suffix on integer literal"));
        }

        if char == 'u' {
            is_unsigned = true;
        }

        numerical_str.pop();
    }

    let storage = StoredSize {
        is_unsigned,
        is_long,
        is_long_long,
        ..Default::default()
    };

    match i128::from_str_radix(&numerical_str.to_string(), radix) {
        Ok(value) => {
            return Ok(TokenTypes::Literal(LiteralTypes::Integer(IntType {
                value,
                storage,
            })));
        }
        Err(_) => {
            return Err(String::from(&format!(
                "Failed to convert number literal {numerical_str} with radix {radix} to integer"
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_number_literal;
    use crate::lexer::{language_features::LiteralTypes, lexer::TokenTypes};

    fn run_tests<const IS_INT: bool>(test_cases: Vec<(&str, &str)>) {
        for (test_case, result) in test_cases {
            if test_case == "" {
                continue;
            }

            let test_result = parse_number_literal(&mut test_case.chars().peekable()).unwrap();

            if IS_INT {
                let int_test_result = match test_result {
                    TokenTypes::Literal(LiteralTypes::Integer(int_type)) => int_type.to_string(),
                    _ => unreachable!(),
                };

                assert_eq!(result, int_test_result)
            } else {
                let float_test_result = match test_result {
                    TokenTypes::Literal(LiteralTypes::Float(float_type)) => float_type.to_string(),
                    _ => unreachable!(),
                };

                assert_eq!(result, float_test_result)
            }
        }
    }

    #[test]
    fn numerical_int_parser() {
        let test_cases = vec![
            ("1", "1"),
            ("42", "42"),
            ("123456", "123456"),
            ("987654321", "987654321"),
            ("7", "7"),
            ("10", "10"),
            ("999999", "999999"),
            ("123u", "123u"),
            ("123L", "123l"),
            ("123UL", "123ul"),
            ("123LL", "123ll"),
            ("123Ull", "123ull"),
            ("0", "0"),
            ("01", "1"),
            ("07", "7"),
            ("010", "8"),
            ("077", "63"),
            ("0777", "511"),
            ("01234567", "342391"),
            ("077u", "63u"),
            ("077L", "63l"),
            ("077uL", "63ul"),
            ("077ll", "63ll"),
            ("077uLL", "63ull"),
            ("0x0", "0"),
            ("0x1", "1"),
            ("0xA", "10"),
            ("0xF", "15"),
            ("0x10", "16"),
            ("0xFF", "255"),
            ("0x1234", "4660"),
            ("0xABCDEF", "11259375"),
            ("0xDEADBEEF", "3735928559"),
            ("0xFFU", "255u"),
            ("0xFFL", "255l"),
            ("0xFFUL", "255ul"),
            ("0xFFLL", "255ll"),
            ("0xFFULL", "255ull"),
            ("0xffffffffffffffffULL", "18446744073709551615ull"),
            ("0xFFFFFFFFFFFFFFFFULL", "18446744073709551615ull"),
            ("12345678901234567890ULL", "12345678901234567890ull"),
        ];

        run_tests::<true>(test_cases);
    }

    #[test]
    fn numerical_float_parser() {
        let test_cases = vec![
            ("1.0", "1"),
            ("0.0", "0"),
            ("3.14", "3.14"),
            ("123.456", "123.456"),
            (".5", "0.5"),
            (".25", "0.25"),
            (".001", "0.001"),
            ("1.", "1"),
            ("123.", "123"),
            ("42.", "42"),
            ("1.0f", "1f"),
            ("1.0F", "1f"),
            ("1.0L", "1l"),
            ("3.14f", "3.14f"),
            ("1e0", "1"),
            ("1e1", "10"),
            ("1e10", "10000000000"),
            ("3.14e2", "314"),
            (".5e3", "500"),
            ("5.e3", "5000"),
            ("1e+10", "10000000000"),
            ("1e-10", "0.0000000001"),
            ("3.14E+5", "314000"),
            ("3.14E-5", "0.0000314"),
            ("0x1p0", "1"),
            ("0x1p1", "2"),
            ("0x1p2", "4"),
            ("0x1p-1", "0.5"),
            ("0x1p-2", "0.25"),
            ("0x1.0p0", "1"),
            ("0x1.8p0", "1.5"),
            ("0x1.8p1", "3"),
            ("0x1.8p2", "6"),
            ("0x1.8p-1", "0.75"),
            ("0x1.p0", "1"),
            ("0x.8p0", "0.5"),
            ("0x.8p1", "1"),
            ("0x.8p2", "2"),
            ("0xA.BCp3", "85.875"),
            ("0xABC.DEFp10", "2814843.75"),
        ];

        run_tests::<false>(test_cases);
    }
}
