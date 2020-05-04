extern crate nom;

use std::str;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while, take_while1, is_a},
    character::complete::{alphanumeric1 as alphanumeric, char, one_of},
    combinator::{map, map_res, opt, cut, iterator},
    multi::separated_list,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated, pair},
    Err, IResult, HexDisplay,
};
use self::nom::Err::Error;
use crate::parser::Value;

pub fn is_string_character(c: char) -> bool {
    c != '"' && c != '\\'
}

fn bool(i: &str) -> IResult<&str, bool> {
    alt((
        map(tag("false"), |_| false),
        map(tag("true"), |_| true)
    ))(i)
}

fn escaped_string(i: &str) -> IResult<&str, &str> {
    escaped(take_while1(is_string_character), '\\', one_of("\"bfnrt\\"))(i)
}

fn string(i: &str) -> IResult<&str, Value> {
    preceded(
        char('\"'),
        cut(terminated(map_res(escaped_string, str_to_val), char('\"'), )),
    )(i)
}

fn str_to_val(v: &str) -> Result<Value, Err<String>> {
    Ok(Value::Str(String::from(v)))
}

#[cfg(test)]
mod tests {
    use crate::parser::parser::{bool, escaped_string, string};
    use super::nom::Err::Error;
    use super::nom::error::ErrorKind::Tag;
    use crate::parser::Value;
    #[test]
    fn bool_test() {
        assert_eq!(bool("true 1"), Ok((" 1", true)));
        assert_eq!(bool("1 true"), Err(Error(("1 true", Tag))))
    }


    #[test]
    fn escaped_string_test() {
        assert_eq!(escaped_string("abc"), Ok(("", "abc")));
        assert_eq!(escaped_string("abc\"bcd\"cde"), Ok(("bcd", "abc")));
    }

    #[test]
    fn string_test() {
        assert_eq!(string("\"abc\""), Ok(("", Value::Str(String::from("abc")))));
        assert_eq!(string(r#""ab\"cb\"cd""#), Ok(("", Value::Str(String::from(r#"ab\"cb\"cd"#)))));
    }
}