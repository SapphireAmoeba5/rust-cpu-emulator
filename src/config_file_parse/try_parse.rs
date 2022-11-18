use crate::debug_println;
use std::borrow::Cow;

pub fn try_parse_number(mut number: &str) -> Result<u64, Cow<str>> {
    let mut sign = 1;
    if number.starts_with('-') {
        sign = -1;
        number = &number[1..];
    }

    match try_parse_hex(number) {
        Ok(Some(val)) => return Ok(if sign == 1 { val } else { val.wrapping_neg() }),
        Err(e) => return Err(e),
        _ => {}
    };

    match try_parse_binary(number) {
        Ok(Some(val)) => return Ok(if sign == 1 { val } else { val.wrapping_neg() }),
        Err(e) => return Err(e),
        _ => {}
    }

    match try_parse_decimal(number) {
        Ok(val) => return Ok(if sign == 1 { val } else { val.wrapping_neg() }),
        Err(e) => Err(e),
    }
}

fn try_parse_hex(hex: &str) -> Result<Option<u64>, Cow<str>> {
    if !hex.starts_with("0x") {
        return Ok(None);
    }

    let hex = &hex[2..];

    if hex.is_empty() {
        return Err(Cow::from("Invalid hexadecimal value"));
    }

    match u64::from_str_radix(hex, 16) {
        Ok(val) => Ok(Some(val)),
        Err(e) => Err(Cow::from(format!("{}", e))),
    }
}

fn try_parse_binary(bin: &str) -> Result<Option<u64>, Cow<str>> {
    if !bin.starts_with("0b") {
        return Ok(None);
    }

    let bin = &bin[2..];

    if bin.is_empty() {
        return Err(Cow::from("Empty binary value"));
    }

    match u64::from_str_radix(bin, 2) {
        Ok(val) => Ok(Some(val)),
        Err(e) => Err(Cow::from(format!("{}", e))),
    }
}

fn try_parse_decimal(bin: &str) -> Result<u64, Cow<str>> {
    match bin.parse::<u64>() {
        Ok(val) => Ok(val),
        Err(e) => Err(Cow::from(format!("{}", e))),
    }
}

fn valid_identifier_character(ch: char) -> bool {
    ch.is_alphabetic() || ch.is_numeric() || ch == '_' || ch == '$' || ch == '-'
}
