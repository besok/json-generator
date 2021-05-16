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
use crate::generator::{GeneratorFunc, Generator};
use crate::generator::generators::{Sequence, UUID, CurrentDateTime, RandomString, RandomInt, RandomFromFile, RandomFromList, RandomArray};
use std::error::Error;
use std::fmt::{Display, Formatter, Debug};
use std::num::ParseIntError;
use nom::bytes::complete::is_not;
use std::str::FromStr;
use nom::error::{ErrorKind, ParseError};


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

pub fn elem(v: &str) -> IResult<&str, &str> {
    take_while(move |c| c != ')' && c != ',')(v)
}

fn gen_func<'a, F>(label: &'a str, extractor: F) -> impl Fn(&'a str) -> IResult<&'a str, Generator>
    where F: Fn(&'a str) -> IResult<&'a str, Generator> {
    preceded(sp, preceded(
        tag(label),
        preceded(
            sp,
            preceded(
                char('('),
                terminated(
                    extractor,
                    preceded(
                        sp,
                        preceded(char(')'), sp))),
            ),
        ),
    ))
}

fn args<'a, F>(transformer: F) -> impl Fn(&'a str) -> IResult<&'a str, Generator>
    where F: Fn(Vec<&str>) -> Result<Generator, GenError> {
    map_res(separated_list(char(','), elem), transformer)
}

fn current_dt(i: &str) -> IResult<&str, Generator> {
    gen_func("dt",
             map_res(take_while(end_br),
                     |s: &str| {
                         let format =
                             if s.chars().filter(|c| !c.is_whitespace()).count() < 2 { "%Y-%m-%d %H:%M:%S".to_string() } else { s.to_string() };
                         new(CurrentDateTime { format })
                     }))(i)
}

fn uuid(i: &str) -> IResult<&str, Generator> {
    gen_func("uuid", args(|_| { new(UUID {}) }))(i)
}


fn sequence(i: &str) -> IResult<&str, Generator> {
    gen_func("seq", args(|elems| {
        new({
            let val = if let Some(Ok(new_val)) = elems.get(0).map(|e| e.parse()) {
                new_val
            } else { 0 };

            let step =
                if let Some(Ok(new_step)) = elems.get(1).map(|e| e.parse()) {
                    new_step
                } else { 1 };

            Sequence { val, step }
        })
    }),
    )(i)
}

fn random_string(i: &str) -> IResult<&str, Generator> {
    gen_func("str", args(|elems| {
        new({
            let n = if let Some(Ok(new_n)) = elems.get(0).map(|e| e.parse()) {
                new_n
            } else { 0 };

            let prefix =
                if let Some(Ok(new_p)) = elems.get(1).map(|e| e.parse()) {
                    new_p
                } else { String::new() };

            let suffix =
                if let Some(Ok(new_s)) = elems.get(2).map(|e| e.parse()) {
                    new_s
                } else { String::new() };


            RandomString::new_with(n, prefix, suffix)
        })
    }),
    )(i)
}

fn random_int(i: &str) -> IResult<&str, Generator> {
    preceded(tag("random_int("),
             terminated(
                 map_res(separated_list(preceded(sp, char(',')), str_to_int),
                         |v: Vec<i64>| {
                             match v[..] {
                                 [s, f] => new(RandomInt::new(s, f)),
                                 _ => Err(GenError::new())
                             }
                         }),
                 preceded(sp, preceded(char(')'), sp)),
             ),
    )(i)
}


// fn random_str_from_list(i: &str) -> IResult<&str, Generator> {
//     random_from_list(i, "random_str_from_list(", |s| String::from(s))
// }
//
// fn random_int_from_list(i: &str) -> IResult<&str, Generator> {
//     random_from_list(i, "random_int_from_list(", |s| {
//         let result: Result<i64, ParseIntError> = s.trim().parse();
//         match result {
//             Ok(e) => e,
//             Result::Err(e) => panic!(" can no possible to parse the list of values: {}", e.to_string()),
//         }
//     })
// }
//
// fn random_from_list<'a, T: 'static + Into<Json> + Clone, F: Fn(&str) -> T>(i: &'a str, label: &str, mp: F) -> IResult<&'a str, Generator> {
//     preceded(tag(label),
//              terminated(
//                  map_res(separated_list(preceded(sp, char(',')),
//                                         |s| {
//                                             map_res(take_while(|c| c != ')' && c != ','),
//                                                     |n: &str| {
//                                                         let res: Result<&str, GenError> = Ok(n);
//                                                         res
//                                                     })(s)
//                                         },
//                  ),
//                          |v: Vec<&str>| {
//                              if v.is_empty() {
//                                  Err(GenError::new())
//                              } else {
//                                  new(RandomFromList::new(v.into_iter().map(|s| mp(s)).collect()))
//                              }
//                          }),
//                  preceded(sp, char(')')),
//              ),
//     )(i)
// }

fn random_array(i: &str) -> IResult<&str, Generator> {
    preceded(tag("array("),
             terminated(map_res(separated_pair(
                 preceded(sp,
                          |s| {
                              map_res(take_while1(char::is_numeric),
                                      |s: &str| {
                                          let res: Result<&str, GenError> = Ok(s);
                                          res
                                      })(s)
                          }),
                 cut(preceded(sp, char(','))),
                 |s| {
                     map_res(take_while1(|c| c != ')'),
                             |s: &str| {
                                 let res: Result<&str, GenError> = Ok(s);
                                 res
                             })(s)
                 }),
                                |v: (&str, &str)| {
                                    match v {
                                        (s, f) =>
                                            match generator(format!("{})", f).as_str()) {
                                                Ok((r, g)) =>
                                                    new(RandomArray::new(s.parse().unwrap(), g)),
                                                Result::Err(err) => Err(GenError::new())
                                            }
                                    }
                                }),
                        preceded(sp, char(')')),
             ),
    )(i)
}
//
// fn random_from_file<'a, T: 'static + FromStr + Clone + Into<Json>>(i: &'a str, label: &str) -> IResult<&'a str, Generator>
//     where <T as FromStr>::Err: Debug {
//     preceded(tag(label),
//              terminated(
//                  map_res(separated_list(preceded(sp, char(',')),
//                                         |s| {
//                                             map_res(take_while(|c| c != ')' && c != ','),
//                                                     |n: &str| {
//                                                         let res: Result<&str, GenError> = Ok(n);
//                                                         res
//                                                     })(s)
//                                         },
//                  ),
//                          |v: Vec<&str>| {
//                              match v[..] {
//                                  [p] => new_if_ok(RandomFromFile::<T>::new(p, ",")),
//                                  [p, d] => {
//                                      new_if_ok(RandomFromFile::<T>::new(p, d))
//                                  }
//                                  _ => Err(GenError::new())
//                              }
//                          }),
//                  preceded(sp, char(')')),
//              ),
//     )(i)
// }
//
// fn random_str_from_file(i: &str) -> IResult<&str, Generator> {
//     random_from_file::<String>(i, "random_str_from_file(")
// }
//
// fn random_int_from_file(i: &str) -> IResult<&str, Generator> {
//     random_from_file::<i64>(i, "random_int_from_file(")
// }

pub fn generator(i: &str) -> IResult<&str, Generator> {
    preceded(sp,
             alt((
                 sequence,
                 uuid,
                 random_string,
                 random_int,
                 current_dt,
                 // random_str_from_file,
                 // random_int_from_file,
                 // random_str_from_list,
                 // random_int_from_list,
                 random_array
             )))(i)
}

fn new<T: GeneratorFunc + 'static>(gf: T) -> Result<Generator, GenError> {
    Ok(Generator::new(gf))
}

//todo exception will not be shown : at the end it will be suppressed
fn new_if_ok<T: GeneratorFunc + 'static>(gf: Result<T, std::io::Error>) -> Result<Generator, GenError> {
    match gf {
        Ok(f) => Ok(Generator::new(f)),
        Result::Err(e) => Err(GenError::new_with(e.to_string())),
    }
}


#[derive(Debug)]
pub struct GenError {
    reason: String
}

impl GenError {
    pub fn new() -> Self {
        GenError { reason: "generator error".to_string() }
    }
    pub fn new_with(reason: String) -> Self {
        GenError { reason }
    }
}

impl Error for GenError {}

impl Display for GenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "error while parsing a generator func, reason: {}", self.reason)
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::generator::{uuid, generator, current_dt, random_string, random_int,
                                   random_array};
    use nom::error::ErrorKind;
    use crate::generator::Generator;
    use serde_json::Value;

    #[test]
    fn current_dt_test() {
        if_let!(current_dt("dt ()")
                => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!(19, el.len())));

        if_let!(current_dt(" dt (  ) ")
                => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!(19, el.len())));
        if_let!(current_dt("dt(%Y-%m-%d)")
                => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!(10, el.len())));
    }

    #[test]
    fn uuid_test() {
        if_let!(uuid("uuid()") => Ok((_, g)) => if_let!(g.next() => Value::String(el) => assert_eq!(el.len(), 36)));
        if_let!(uuid(" uuid ( ) ") => Ok((_, g)) => if_let!(g.next() => Value::String(el) => assert_eq!(el.len(), 36)));
    }

    #[test]
    fn seq_test() {
        if_let!(generator("seq(1)") => Ok((_, gen)) => {
            assert_eq!(Some(2), gen.next().as_i64());
            assert_eq!(Some(3), gen.next().as_i64());
            assert_eq!(Some(4), gen.next().as_i64());
        });

        if_let!(generator("seq(-1)") => Ok((_, gen)) => {
            assert_eq!(Some(0), gen.next().as_i64());
            assert_eq!(Some(1), gen.next().as_i64());
            assert_eq!(Some(2), gen.next().as_i64());
        });

        if_let!(generator("seq(-1,-1)") => Ok((_, gen)) => {
            assert_eq!(Some(-2), gen.next().as_i64());
            assert_eq!(Some(-3), gen.next().as_i64());
            assert_eq!(Some(-4), gen.next().as_i64());
        });
        if_let!(generator("seq(10,-10)") => Ok((_, gen)) => {
            assert_eq!(Some(0), gen.next().as_i64());
            assert_eq!(Some(-10), gen.next().as_i64());
            assert_eq!(Some(-20), gen.next().as_i64());
        });
    }

    #[test]
    fn random_string_test() {
        if_let!(random_string("str(10)") => Ok((_, g)) => if_let!(g.next() => Value::String(el) => assert_eq!(el.len(), 10)));
        if_let!(random_string("str(abc)") => Ok((_, g)) => if_let!(g.next() => Value::String(el) => assert_eq!(el.len(), 0)));
        if_let!(random_string("str(10,abc)") => Ok((_, g)) => if_let!(g.next() => Value::String(el) => {
            assert_eq!(el.len(), 13);
            assert!(el.starts_with("abc"));
            }));
        if_let!(random_string("str(10,,abc)") => Ok((_, g)) => if_let!(g.next() => Value::String(el) => {
            assert_eq!(el.len(), 13);
            assert!(el.ends_with("abc"));
            }));
        if_let!(random_string("str(10,cba,abc)") => Ok((_, g)) => if_let!(g.next() => Value::String(el) => {
            assert_eq!(el.len(), 16);
            assert!(el.ends_with("abc"));
            assert!(el.starts_with("cba"));
            }));
        if_let!(random_string("str(0,jimmy_smith,@gmail.com)") => Ok((_, g)) => if_let!(g.next() => Value::String(el) => {
            assert_eq!(el, "jimmy_smith@gmail.com");
            }));
    }

    // #[test]
    // fn random_int_array_test() {
    //     if_let!(
    //     random_array("array(3,random_int_from_list(1,2,3,4))") =>  Ok((_, el))
    //         => if_let!(el.next() => Json::Array(v)
    //             => if_let!(v[..] => [Json::Num(e1), Json::Num(e2), Json::Num(e3)]
    //                 => assert_eq!(e1 > 0 && e1 < 5 && e2 > 0 && e2 < 5 && e3 > 0 && e3 < 5, true)))
    //                 );
    // }
    //
    //
    //
    // #[test]
    // fn random_int_test() {
    //     if_let!(random_int("random_int(10,20)")=> Ok((_, g))
    //             => if_let!(g.next() => Json::Num(el) => assert_eq!(el > 9 && el < 20, true)));
    // }
    //
    // #[test]
    // fn random_str_from_list_test() {
    //     if_let!(random_str_from_list(r#"random_str_from_list(a,b,c,d)"#) => Ok((_, g))
    //             => if_let!(g.next() => Json::Str(el)
    //                 => assert_eq!("abcd".contains(el.as_str()), true)));
    // }
    //
    // #[test]
    // fn random_int_from_list_test() {
    //     if_let!(random_int_from_list(r#"random_int_from_list(1,2,3)"#) => Ok((_, g))
    //             => if_let!(g.next() => Json::Num(el)
    //                 => assert_eq!(el > 0 && el < 4, true)));
    // }
    //
    // #[test]
    // fn random_str_from_file_nl_test() {
    //     if_let!(random_str_from_file(r#"random_str_from_file(C:\projects\json-generator\jsons\cities, \r\n)"#)
    //             => Ok((_, g))
    //             => if_let!(g.next() => Json::Str(el)
    //                 => assert_eq!("BerlinPragueMoscowLondonHelsinkiRomeBarcelonaViennaAmsterdamDublin"
    //                                .contains(el.as_str()), true)));
    // }
    //
    // #[test]
    // fn random_str_from_file_test() {
    //     if_let!(random_str_from_file(r#"random_str_from_file(C:\projects\json-generator\jsons\list.txt,;)"#)
    //             => Ok((_, g))
    //             => if_let!(g.next() => Json::Str(el)
    //                 => assert_eq!(el, "1,2,3,4,5,6".to_string())));
    //
    //     if_let!(random_str_from_file(r#"random_str_from_file(C:\projects\json-generator\jsons\list.txt)"#)
    //             => Ok((_, g))
    //             => if_let!(g.next() => Json::Str(el)
    //                 => assert_eq!("1,2,3,4,5,6".contains(el.as_str()), true)));
    // }
    //
    // #[test]
    // fn random_int_from_file_test() {
    //     if_let!(random_int_from_file(r#"random_int_from_file(C:\projects\json-generator\jsons\list.txt)"#)
    //             => Ok((_, g))
    //             => if_let!(g.next() => Json::Num(el)
    //                 => assert_eq!(el > 0 && el < 7, true)));
    // }
}