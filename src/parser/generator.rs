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
use crate::parser::json::{sp, str_to_str};
use nom::bytes::complete::is_not;
use std::str::FromStr;
use crate::parser::Json;
use nom::error::ErrorKind;


fn end_br(c: char) -> bool {
    c != ')'
}

fn str_to_int(i: &str) -> IResult<&str, i64> {
    map_res(take_while1(char::is_numeric),
            |s: &str| {
                let res: Result<i64, ParseIntError> = s.parse();
                res
            })(i)
}

fn current_dt(i: &str) -> IResult<&str, Generator> {
    preceded(tag("current_date_time("),
             terminated(
                 map_res(take_while(end_br),
                         |s: &str| {
                             let format =
                                 if s.len() < 2 {
                                     "%Y-%m-%d %H:%M:%S".to_string()
                                 } else { s.to_string() };
                             new(CurrentDateTime { format })
                         }),
                 preceded(sp, char(')')),
             ),
    )(i)
}

fn sequence(i: &str) -> IResult<&str, Generator> {
    preceded(tag("sequence("),
             terminated(
                 map_res(take_while1(char::is_numeric),
                         |s: &str| {
                             let res: Result<Generator, ParseIntError> =
                                 Ok(Generator::new(Sequence { val: s.parse()? }));
                             res
                         })
                 , char(')'),
             ),
    )(i)
}

fn uuid(i: &str) -> IResult<&str, Generator> {
    map_res(tag("uuid()"), |_: &str| {
        new(UUID {})
    })(i)
}

fn random_string(i: &str) -> IResult<&str, Generator> {
    preceded(tag("random_str("),
             terminated(
                 map_res(take_while1(char::is_numeric),
                         |s: &str| {
                             let res: Result<Generator, ParseIntError> =
                                 Ok(Generator::new(RandomString::new(s.parse()?)));
                             res
                         })
                 , char(')'),
             ),
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
                 preceded(sp, char(')')),
             ),
    )(i)
}


fn random_str_from_list(i: &str) -> IResult<&str, Generator> {
    random_from_list(i, "random_str_from_list(", |s| String::from(s))
}

fn random_int_from_list(i: &str) -> IResult<&str, Generator> {
    random_from_list(i, "random_int_from_list(", |s| {
        let result: Result<i64, ParseIntError> = s.trim().parse();
        match result {
            Ok(e) => e,
            Result::Err(e) => panic!(" can no possible to parse the list of values: {}", e.to_string()),
        }
    })
}

fn random_from_list<'a, T: 'static + Into<Json> + Clone, F: Fn(&str) -> T>(i: &'a str, label: &str, mp: F) -> IResult<&'a str, Generator> {
    preceded(tag(label),
             terminated(
                 map_res(separated_list(preceded(sp, char(',')),
                                        |s| {
                                            map_res(take_while(|c| c != ')' && c != ','),
                                                    |n: &str| {
                                                        let res: Result<&str, GenError> = Ok(n);
                                                        res
                                                    })(s)
                                        },
                 ),
                         |v: Vec<&str>| {
                             if v.is_empty() {
                                 Err(GenError::new())
                             } else {
                                 new(RandomFromList::new(v.into_iter().map(|s| mp(s)).collect()))
                             }
                         }),
                 preceded(sp, char(')')),
             ),
    )(i)
}

fn random_array(i: &str) -> IResult<&str, Generator> {
    preceded(tag("array("),
             terminated(
                 map_res(
                     separated_pair(
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

fn random_from_file<'a, T: 'static + FromStr + Clone + Into<Json>>(i: &'a str, label: &str) -> IResult<&'a str, Generator>
    where <T as FromStr>::Err: Debug {
    preceded(tag(label),
             terminated(
                 map_res(separated_list(preceded(sp, char(',')),
                                        |s| {
                                            map_res(take_while(|c| c != ')' && c != ','),
                                                    |n: &str| {
                                                        let res: Result<&str, GenError> = Ok(n);
                                                        res
                                                    })(s)
                                        },
                 ),
                         |v: Vec<&str>| {
                             match v[..] {
                                 [p] => new_if_ok(RandomFromFile::<T>::new(p, ",")),
                                 [p, d] => {
                                     new_if_ok(RandomFromFile::<T>::new(p, d))
                                 }
                                 _ => Err(GenError::new())
                             }
                         }),
                 preceded(sp, char(')')),
             ),
    )(i)
}

fn random_str_from_file(i: &str) -> IResult<&str, Generator> {
    random_from_file::<String>(i, "random_str_from_file(")
}

fn random_int_from_file(i: &str) -> IResult<&str, Generator> {
    random_from_file::<i64>(i, "random_int_from_file(")
}

pub fn generator(i: &str) -> IResult<&str, Generator> {
    preceded(sp,
             alt((
                 uuid,
                 sequence,
                 random_string,
                 random_int,
                 current_dt,
                 random_str_from_file,
                 random_int_from_file,
                 random_str_from_list,
                 random_int_from_list,
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
    use crate::parser::generator::{sequence, uuid, generator, current_dt, random_string, random_int, random_str_from_file, random_int_from_file, random_str_from_list, random_int_from_list, random_array};
    use crate::parser::{Json, Field};
    use nom::error::ErrorKind;
    use crate::generator::Generator;


    #[test]
    fn random_int_array_test() {
        match random_array("array(3,random_int_from_list(1,2,3,4))") {
            Ok((_, el)) => {
                if let Json::Array(v) = el.next() {
                    match v[..] {
                        [Json::Num(e1), Json::Num(e2), Json::Num(e3)] => assert_eq!(
                            e1 > 0 && e1 < 5 && e2 > 0 && e2 < 5 && e3 > 0 && e3 < 5, true),
                        _ => panic!("!")
                    }
                } else {
                    panic!("should not be")
                }
            }
            Err(e) => panic!("{}", e)
        }
        match random_array("array(3,random_int_from_list(1,2,3,4 ) )") {
            Ok((_, el)) => {
                if let Json::Array(v) = el.next() {
                    match v[..] {
                        [Json::Num(e1), Json::Num(e2), Json::Num(e3)] => assert_eq!(
                            e1 > 0 && e1 < 5 && e2 > 0 && e2 < 5 && e3 > 0 && e3 < 5, true),
                        _ => panic!("!")
                    }
                } else {
                    panic!("should not be")
                }
            }
            Err(e) => panic!("{}", e)
        }
    }


    #[test]
    fn seq_gen_test() {
        if let Ok((_, el)) = sequence("sequence(10)") {
            assert_eq!(Json::Num(11), el.next());
            assert_eq!(Json::Num(12), el.next());
            assert_eq!(Json::Num(13), el.next());
        }
    }

    #[test]
    fn uuid_test() {
        match uuid("uuid()") {
            Ok((_, g)) =>
                if let Json::Str(el) = g.next() {
                    assert_eq!(el.len(), 36)
                } else {
                    panic!("test failed")
                },
            Err(e) => panic!("{}", e)
        }
    }

    #[test]
    fn random_string_test() {
        if let Ok((_, el)) = random_string("random_str(10)") {
            if let Json::Str(el) = el.next() {
                assert_eq!(el.len(), 10)
            } else {
                panic!("test failed")
            }
        } else {
            panic!("test failed")
        }
    }

    #[test]
    fn random_int_test() {
        if let Ok((_, el)) = random_int("random_int(10,20)") {
            if let Json::Num(el) = el.next() {
                assert_eq!(el > 9 && el < 20, true)
            } else {
                panic!("test failed")
            }
        } else {
            panic!("test failed")
        }
    }

    #[test]
    fn random_str_from_list_test() {
        match random_str_from_list(r#"random_str_from_list(a,b,c,d)"#) {
            Ok((_, el)) =>
                if let Json::Str(el) = el.next() {
                    assert_eq!("abcd".contains(el.as_str()), true)
                } else {
                    panic!("test failed")
                },
            Err(err) => panic!("{:?}", err)
        }
    }

    #[test]
    fn random_int_from_list_test() {
        match random_int_from_list(r#"random_int_from_list(1,2,3)"#) {
            Ok((_, el)) =>
                if let Json::Num(el) = el.next() {
                    assert_eq!(el > 0 && el < 4, true)
                } else {
                    panic!("test failed")
                },
            Err(err) => panic!("{:?}", err)
        }
    }

    #[test]
    fn random_str_from_file_nl_test() {
        match random_str_from_file(r#"random_str_from_file(C:\projects\json-generator\jsons\cities.txt, \r\n)"#) {
            Ok((_, el)) =>
                if let Json::Str(el) = el.next() {
                    assert_eq!("BerlinPragueMoscowLondonHelsinkiRomeBarcelonaViennaAmsterdamDublin"
                                   .contains(el.as_str()), true)
                } else {
                    panic!("test failed")
                },
            Err(err) => panic!("{:?}", err)
        }
    }

    fn random_str_from_file_test() {
        match random_str_from_file(r#"random_str_from_file(C:\projects\json-generator\jsons\list.txt,;)"#) {
            Ok((_, el)) =>
                if let Json::Str(el) = el.next() {
                    assert_eq!(el, "1,2,3,4,5,6".to_string())
                } else {
                    panic!("test failed")
                },
            Err(err) => panic!("{:?}", err)
        }

        match random_str_from_file(r#"random_str_from_file(C:\projects\json-generator\jsons\list.txt)"#) {
            Ok((_, el)) =>
                if let Json::Str(el) = el.next() {
                    assert_eq!("1,2,3,4,5,6".contains(el.as_str()), true)
                } else {
                    panic!("test failed")
                },
            Err(err) => panic!("{:?}", err)
        }
    }

    #[test]
    fn random_int_from_file_test() {
        match random_int_from_file(r#"random_int_from_file(C:\projects\json-generator\jsons\list.txt)"#) {
            Ok((_, el)) =>
                if let Json::Num(el) = el.next() {
                    assert_eq!(el > 0 && el < 7, true)
                } else {
                    panic!("test failed")
                },
            Err(err) => panic!("{:?}", err)
        }
    }

    #[test]
    fn current_dt_test() {
        match current_dt("current_date_time()") {
            Ok((_, el)) =>
                if let Json::Str(s) = el.next() {
                    assert_eq!(19, s.len())
                } else {
                    panic!("")
                },
            Err(e) => panic!("{:?}", e),
        }
        match current_dt("current_date_time(%Y-%m-%d)") {
            Ok((_, el)) =>
                if let Json::Str(s) = el.next() {
                    assert_eq!(10, s.len())
                } else {
                    panic!("")
                },
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn generator_test() {
        if let Ok((_, gen)) = generator(" uuid() ") {
            if let Json::Str(uuid) = gen.next() {
                assert_eq!(uuid.len(), 36)
            } else {
                panic!("should be str")
            }
        } else {
            panic!("panic!")
        }


        if let Ok((_, gen)) = generator("sequence(1)") {
            assert_eq!(Json::Num(2), gen.next());
            assert_eq!(Json::Num(3), gen.next());
            assert_eq!(Json::Num(4), gen.next());
        } else {
            panic!("panic!")
        }
    }
}