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
use self::nom::character::complete::none_of;
use self::nom::bytes::complete::is_not;
use self::nom::character::is_digit;
use std::num::ParseIntError;
use crate::generator::Generator;
use crate::parser::generator::generator;
use self::nom::combinator::map_parser;
use self::nom::error::ErrorKind;
//
// pub fn is_space(c: char) -> bool {
//     c == ' ' || c == '\t' || c == '\r' || c == '\n'
// }
//
// pub fn sp(i: &str) -> IResult<&str, &str> {
//     take_while(is_space)(i)
// }
//
// pub fn is_string_character(c: char) -> bool {
//     c != '"' && c != '\\'
// }
//
// // fn boolean(i: &str) -> IResult<&str, Json> {
// //     alt((
// //         map(tag("false"), |_| Json::Bool(false)),
// //         map(tag("true"), |_| Json::Bool(true))
// //     ))(i)
// // }
// //
// // fn null(i: &str) -> IResult<&str, Json> {
// //     map(tag("null"), |_| Json::Null)(i)
// // }
// //
// // fn escaped_string(i: &str) -> IResult<&str, &str> {
// //     escaped(take_while1(is_string_character), '\\', one_of("\"bfnrt\\"))(i)
// // }
// //
// // fn string(i: &str) -> IResult<&str, Json> {
// //     preceded(
// //         char('\"'),
// //         cut(terminated(map_res(escaped_string, str_to_val), char('\"'))),
// //     )(i)
// // }
// //
// // //todo  add float and double values
// // fn num(i: &str) -> IResult<&str, Json> {
// //     map_res(take_while1(char::is_numeric), str_to_num_json)(i)
// // }
//
// // fn field(i: &str) -> IResult<&str, Field> {
// //     let field =
// //         separated_pair(preceded(sp, key),
// //                        cut(preceded(sp, char(':'))),
// //                        value);
// //
// //
// //     match generator_opt(i) {
// //         Ok((rest, Some(g))) =>
// //             match field(rest) {
// //                 Ok((s, (n, j))) =>
// //                     Ok((s, Field::new_with_gen(n.to_string(), j, g.clone()))),
// //                 Result::Err(x) => Result::Err(x)
// //             }
// //         _ =>
// //             match field(i) {
// //                 Ok((s, (n, j))) => Ok((s, Field::new(n.to_string(), j))),
// //                 Result::Err(x) => Result::Err(x)
// //             }
// //         ,
// //     }
// // }
//
// fn key(i: &str) -> IResult<&str, &str> {
//     preceded(
//         char('\"'), cut(terminated(take_while1(is_string_character), char('\"'))),
//     )(i)
// }
//
// fn array(i: &str) -> IResult<&str, Json> {
//     preceded(char('['),
//              cut(terminated(
//                  map_res(separated_list(preceded(sp, char(',')), value),
//                          arr_to_val),
//                  preceded(sp, char(']')),
//              )),
//     )(i)
// }
//
// fn object(i: &str) -> IResult<&str, Json> {
//     preceded(char('{'),
//              cut(terminated(
//                  map_res(separated_list(preceded(sp, char(',')), field),
//                          obj_to_val),
//                  preceded(sp, char('}')),
//              )),
//     )(i)
// }
//
//
// pub fn value(i: &str) -> IResult<&str, Json> {
//     preceded(sp,
//              alt((
//                  boolean, null, num, array, string, object
//              )),
//     )(i)
// }
//
//
// fn generator_opt(i: &str) -> IResult<&str, Option<Generator>> {
//     opt(generator_func)(i)
// }
//
// fn generator_func(i: &str) -> IResult<&str, Generator> {
//     preceded(sp, preceded(tag("/*"),
//                           terminated(
//                               map_parser(is_not("/**/"), generator),
//                               tag("*/")),
//     ))(i)
// }
//
// fn str_to_val(v: &str) -> Result<Json, Err<String>> {
//     Ok(Json::Str(String::from(v)))
// }
//
// fn arr_to_val(v: Vec<Json>) -> Result<Json, Err<String>> {
//     Ok(Json::Array(v))
// }
//
// fn obj_to_val(v: Vec<Field>) -> Result<Json, Err<String>> {
//     Ok(Json::Object(v))
// }
//
// fn str_to_num_json(v: &str) -> Result<Json, ParseIntError> {
//     let res: i64 = v.parse()?;
//     Ok(Json::Num(res))
// }
//
// pub fn str_to_str(v: &str) -> Result<&str, Err<String>> {
//     Ok(v)
// }

// #[cfg(test)]
// mod tests {
//     use crate::parser::json::{ generator_func, num, array, object, generator_opt};
//     use super::nom::Err::Error;
//     use super::nom::error::ErrorKind::Tag;
//     use crate::parser::Json::{Array, Num, Object};
//     use super::nom::error::ErrorKind;
//     use crate::generator::Generator;
//
//     #[test]
//     fn bool_test() {
//         assert_eq!(boolean("true 1"), Ok((" 1", Json::Bool(true))));
//         assert_eq!(boolean("1 true"), Err(Error(("1 true", Tag))))
//     }
//
//
//     #[test]
//     fn escaped_string_test() {
//         assert_eq!(escaped_string("abc"), Ok(("", "abc")));
//         assert_eq!(escaped_string("abc\"bcd\"cde"), Ok(("\"bcd\"cde", "abc")));
//     }
//
//     #[test]
//     fn string_test() {
//         assert_eq!(string("\"abc\""), Ok(("", Json::Str(String::from("abc")))));
//         assert_eq!(string(r#""ab\"cb\"cd""#), Ok(("", Json::Str(String::from(r#"ab\"cb\"cd"#)))));
//     }
//
//     #[test]
//     fn limiter_test() {
//
//         if_let!(
//             generator_func("/* sequence(10) */") => Ok((_, g)) => assert_eq!(Json::Num(11), g.next())
//         );
//         if_let!(
//             generator_func("/*uuid()*/") => Ok((_, g))
//             => if_let!(g.next() => Json::Str(uuid) => assert_eq!(uuid.len(), 36))
//         );
//
//     }
//
//     #[test]
//     fn number_test() {
//         assert_eq!(num("54 abc"), Ok((" abc", Json::Num(54))));
//     }
//
//     // #[test]
//     // fn array_test() {
//     //     assert_eq!(array("[1,2,3 , 4]"), Ok(("", Array(vec![Num(1), Num(2), Num(3), Num(4)]))));
//     //     assert_eq!(array("[ ]"), Ok(("", Array(vec![]))));
//     // }
//     //
//     // #[test]
//     // fn field_test() {
//     //     assert_eq!(field(r#""field":"string""#), Ok(("", Field::new("field".to_string(), Json::Str("string".to_string())))));
//     //     let f = field(r#"
//     //     /* sequence(10) */
//     //     "field":"string"
//     //     "#);
//     //     assert_eq!(f,
//     //                Ok(("\n        ", Field::new("field".to_string(), Json::Str("string".to_string()))))
//     //     );
//     //
//     //     let (_, field) = f.unwrap();
//     //     let g = field.g.clone().unwrap();
//     //     assert_eq!(g.next(), Json::Num(11));
//     // }
//     //
//     //
//     // #[test]
//     // fn generator_opt_test() {
//     //     if_let!(
//     //         generator_opt(r#"/*
//     //                                  sequence(10) */ "#) =>  Ok((_, Some(g)))
//     //         => assert_eq!(Json::Num(11), g.next())
//     //         );
//     //
//     //     if_let!( generator_opt(r#"sequence(10) */"#) =>  Ok((_, None))=> ());
//     //     if_let!(
//     //         generator_opt(r#"
//     //     /* sequence(10) */
//     //     "field":"string"
//     //     "#)
//     //         =>  Ok((s, Some(_)))
//     //         => assert_eq!(s, r#"
//     //     "field":"string"
//     //     "#));
//     // }
//     //
//     //
//     // #[test]
//     // fn object_test() {
//     //     assert_eq!(object(r#"{"field": {"next_field": {"final_field": 42}}}"#),
//     //                Ok(("", Object(vec![Field {
//     //                    name: "field".to_string(),
//     //                    value: Object(vec![Field {
//     //                        name: "next_field".to_string(),
//     //                        value: Object(vec![Field {
//     //                            name: "final_field".to_string(),
//     //                            value: Num(42),
//     //                            g: None,
//     //                        }]),
//     //                        g: None,
//     //                    }]),
//     //                    g: None,
//     //                }]))));
//     //     let res = object(r#"{
//     // /* sequence(10) */
//     // "id": 1
//     // }"#);
//     //
//     //     if_let!( res =>  Ok((_, Json::Object(f))) =>
//     //         if_let!(f.get(0) => Some(field) => assert_eq!(field.get_next(), Some(Json::Num(11))))
//     //     );
//     //
//     // }
// }