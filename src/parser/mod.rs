use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use nom::error::{ErrorKind, ParseError};

use crate::generator::Generator;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while, take_while1, is_a},
    character::complete::{alphanumeric1 as alphanumeric, char, one_of},
    combinator::{map, map_res, opt, cut, iterator},
    number::complete::double,
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, terminated, pair},
    Err, IResult, HexDisplay,
};
use std::num::ParseIntError;
use crate::error::GenError;
pub mod generators;


fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";
    take_while(move |c| chars.contains(c))(i)
}

fn end_br(c: char) -> bool {
    c != ')'
}

fn is_numeric_with_neg(c: char) -> bool {
    char::is_numeric(c) || c == '-'
}

fn str_to_int(i: &str) -> IResult<&str, i64> {
    map_res(take_while1(is_numeric_with_neg),
            |s: &str| {
                let res: Result<i64, ParseIntError> = s.parse();
                res
            })(i)
}

fn escaped_string(v: &str) -> IResult<&str, &str> {
    terminated(
        preceded(sp,
                 preceded(
                     char('\''),
                     escaped(
                         take_while1(move |c| c != '\'' && c != '\\'), '\\', one_of("'")),
                 ), ),
        char('\''))(v)

}

fn start_from_esc_string(v: &str) -> IResult<&str, &str> {
    terminated(
        preceded(sp,
                 preceded(
                     char('\\'),
                     escaped_string,
                 ), ),
        char('\\'))(v)
}

fn string(v: &str) -> IResult<&str, &str> {
    preceded(sp, take_while(move |c| c != ')' && c != ','))(v)
}


pub fn plain_string(v: &str) -> IResult<&str, &str> {
    alt((start_from_esc_string, escaped_string, string))(v)
}

fn func<'a, F>(label: &'a str, extractor: F) -> impl FnMut(&'a str) -> IResult<&'a str, Generator>
    where F: FnMut(&'a str) -> IResult<&'a str, Generator> {
    func_with_br(label, '(', ')', extractor)
}

fn func_with_br<'a, F>(label: &'a str, br_l: char, br_r: char, extractor: F) -> impl FnMut(&'a str) -> IResult<&'a str, Generator>
    where F: FnMut(&'a str) -> IResult<&'a str, Generator> {
    preceded(sp, preceded(
        tag(label),
        preceded(
            sp,
            preceded(
                char(br_l),
                terminated(
                    extractor,
                    preceded(
                        sp, char(br_r),
                    ),
                ),
            ),
        ),
    ))
}

fn args_string<'a, F>(transformer: F) -> impl FnMut(&'a str) -> IResult<&'a str, Generator>
    where F: Fn(Vec<&'a str>) -> Result<Generator, GenError> {
    args(transformer, plain_string)
}

fn args<'a, F, T, S>(transformer: F, elem_transformer: S) -> impl FnMut(&'a str) -> IResult<&'a str, Generator>
    where
        F: Fn(Vec<T>) -> Result<Generator, GenError>,
        S: Fn(&'a str) -> IResult<&'a str, T> {
    map_res(separated_list0(char(','), elem_transformer), transformer)
}