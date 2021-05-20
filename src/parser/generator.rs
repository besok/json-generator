use std::str;
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
use crate::generator::{GeneratorFunc, Generator};
use crate::generator::generators::{Sequence, UUID, CurrentDateTime, RandomString, RandomInt, RandomFromFile, RandomFromList, RandomArray, RandomBool, Null};
use crate::generator::from_string::FromStringTo;
use std::error::Error;
use std::fmt::{Display, Formatter, Debug};
use std::num::ParseIntError;
use nom::bytes::complete::is_not;
use nom::error::{ErrorKind, ParseError};
use nom::character::complete::satisfy;
use crate::parser::{func, args_string, args, str_to_int, sp, func_with_br, GenError};
use uuid::Version::Random;
use std::rc::Rc;
use std::cell::RefCell;

fn current_dt(i: &str) -> IResult<&str, Generator> {
    func("dt",
         args_string(|elems| {
             let format =
                 elems.get(0)
                     .filter(|e| !e.is_empty())
                     .map(move |f| f.trim())
                     .unwrap_or("%Y-%m-%d %H:%M:%S").to_string();
             new(CurrentDateTime { format })
         }))(i)
}

fn uuid(i: &str) -> IResult<&str, Generator> {
    func("uuid", args_string(|_| { new(UUID {}) }))(i)
}

fn bool(i: &str) -> IResult<&str, Generator> {
    func("bool", args_string(|_| { new(RandomBool::new()) }))(i)
}


fn sequence(i: &str) -> IResult<&str, Generator> {
    func("seq", args_string(|elems| {
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
    func("str", args_string(|elems| {
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
    }))(i)
}

fn random_int(i: &str) -> IResult<&str, Generator> {
    fn get_or_def(elems: &Vec<&str>, idx: usize, def: i32) -> i32 {
        if let Some(Ok(v)) =
        elems.get(idx).map(|s| if s.is_empty() { Ok(def) } else { s.parse() }) {
            v
        } else { def }
    }

    func("int", args_string(|elems| {
        new({
            let lower = get_or_def(&elems, 0, 0);
            let upper = get_or_def(&elems, 1, 1000);
            RandomInt::new(lower, upper)
        })
    }))(i)
}


fn random_str_from_list(i: &str) -> IResult<&str, Generator> {
    func("str_from_list",
         args_string(|elems| {
             let values =
                 elems
                     .into_iter()
                     .map(|e| e.trim())
                     .map(String::from)
                     .collect();
             new(RandomFromList::new(values))
         }))(i)
}

fn random_int_from_list(i: &str) -> IResult<&str, Generator> {
    func("int_from_list",
         args(|elems| {
             new(RandomFromList::new(elems))
         }, str_to_int))(i)
}

fn random_array_empty(i: &str) -> IResult<&str, Generator> {
    func("array", args(|elems| {
        let len = if let Some(idx) = elems.get(0) { *idx as usize } else { 1 as usize };
        new(RandomArray::new_size(len))
    }, str_to_int))(i)
}

fn random_str_from_file(i: &str) -> IResult<&str, Generator> {
    func("str_from_file",
         args_string(|elems| {
             match elems[..] {
                 [path, d1, d2] if d1 == "" && d2 == "" =>
                     new(RandomFromFile::<String>::new(path, ",")?),
                 [path, d] =>
                     new(RandomFromFile::<String>::new(path, d)?),
                 _ => Err(GenError::new())
             }
         }))(i)
}

fn random_int_from_file(i: &str) -> IResult<&str, Generator> {
    func("int_from_file",
         args_string(|elems| {
             match elems[..] {
                 [path, d1, d2] if d1 == "" && d2 == "" =>
                     new(RandomFromFile::<i64>::new(path, ",")?),
                 [path, d] =>
                     new(RandomFromFile::<i64>::new(path, d)?),
                 _ => Err(GenError::new())
             }
         }))(i)
}


pub fn generator(i: &str) -> Result<Generator, GenError> {
    map_res(preceded(sp, separated_list0(tag("->"), atomic_generator)),
            |gens| {
                let mut res: Result<Generator, GenError> =
                    gens
                        .get(0)
                        .map(|g| g.clone())
                        .ok_or(GenError::new());

                for el in gens.iter().skip(1) {
                    res = res.and_then(|g| el.merge(&g))
                }
                res
            },
    )(i)
        .map(|e| e.1)
        .map_err(|e| GenError::new_with(e.to_string()))
}

pub fn atomic_generator(i: &str) -> IResult<&str, Generator> {
    terminated(
        preceded(
            sp,
            alt((
                sequence,
                uuid,
                random_string,
                random_int,
                current_dt,
                random_str_from_file,
                random_int_from_file,
                random_str_from_list,
                random_int_from_list,
                random_array_empty,
                bool
            ))), sp)(i)
}

fn new<T: GeneratorFunc + 'static>(gf: T) -> Result<Generator, GenError> {
    Ok(Generator::new(gf))
}

#[cfg(test)]
mod tests {
    use crate::parser::generator::{uuid, atomic_generator, generator, current_dt, random_string, random_int, GenError};
    use nom::error::{ErrorKind, Error};
    use crate::generator::Generator;
    use serde_json::{Value, json};

    fn gen(i: &str) -> Result<Generator, GenError> {
        generator(i)
    }

    #[test]
    fn current_dt_test() {
        if_let!(gen("dt()")
                => Ok(g)
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!(19, el.len())));

        if_let!(gen(" dt (  ) ")
                => Ok(g)
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!(19, el.len())));

        if_let!(gen("dt( %Y-%m-%d )")
                => Ok(g)
                => if_let!(g.next() => Value::String(el)
                    => {
                    println!("{}",el);
                    assert_eq!(10, el.len())
                    }));
    }

    #[test]
    fn uuid_test() {
        if_let!(gen("uuid()") => Ok(g) => if_let!(g.next() => Value::String(el) => assert_eq!(el.len(), 36)));
        if_let!(gen(" uuid ( ) ") => Ok(g) => if_let!(g.next() => Value::String(el) => assert_eq!(el.len(), 36)));
    }

    #[test]
    fn bool_test() {
        if_let!(gen("bool()") => Ok(g) => assert!(g.next().is_boolean()));
    }

    #[test]
    fn seq_test() {
        if_let!(gen("seq(1)") => Ok(g) => {
            assert_eq!(Some(2), g.next().as_i64());
            assert_eq!(Some(3), g.next().as_i64());
            assert_eq!(Some(4), g.next().as_i64());
        });

        if_let!(gen("seq(-1)") => Ok(g) => {
            assert_eq!(Some(0), g.next().as_i64());
            assert_eq!(Some(1), g.next().as_i64());
            assert_eq!(Some(2), g.next().as_i64());
        });

        if_let!(gen("seq(-1,-1)") => Ok(g) => {
            assert_eq!(Some(-2), g.next().as_i64());
            assert_eq!(Some(-3), g.next().as_i64());
            assert_eq!(Some(-4), g.next().as_i64());
        });
        if_let!(gen("seq(10,-10)") => Ok(g) => {
            assert_eq!(Some(0), g.next().as_i64());
            assert_eq!(Some(-10), g.next().as_i64());
            assert_eq!(Some(-20), g.next().as_i64());
        });
    }

    #[test]
    fn random_string_test() {
        if_let!(gen("str(10)") => Ok(g) => if_let!(g.next() => Value::String(el) => assert_eq!(el.len(), 10)));
        if_let!(gen("str(abc)") => Ok(g) => if_let!(g.next() => Value::String(el) => assert_eq!(el.len(), 0)));
        if_let!(gen("str(10,abc)") => Ok(g) => if_let!(g.next() => Value::String(el) => {
            assert_eq!(el.len(), 13);
            assert!(el.starts_with("abc"));
            }));
        if_let!(gen("str(10,,abc)") => Ok(g) => if_let!(g.next() => Value::String(el) => {
            assert_eq!(el.len(), 13);
            assert!(el.ends_with("abc"));
            }));
        if_let!(gen("str(10,cba,abc)") => Ok(g) => if_let!(g.next() => Value::String(el) => {
            assert_eq!(el.len(), 16);
            assert!(el.ends_with("abc"));
            assert!(el.starts_with("cba"));
            }));
        if_let!(gen("str(0,jimmy_smith,@gmail.com)") => Ok(g) => if_let!(g.next() => Value::String(el) => {
            assert_eq!(el, "jimmy_smith@gmail.com");
            }));
        if_let!(gen("str(0,'jimmy_smith',@gmail.com)") => Ok(g) => if_let!(g.next() => Value::String(el) => {
            assert_eq!(el, "jimmy_smith@gmail.com");
            }));
    }

    #[test]
    fn random_int_test() {
        if_let!(gen("int(0,10)") => Ok(g) => {
             for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n >-1 && n < 11 )
               }
        });

        if_let!(gen("int(-10,11)") => Ok(g) => {
            for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -11 && n < 11 )
               }
        });
        if_let!(gen("int(-10)") => Ok(g) => {
            for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -11 && n < 1000 )
               }
        });
        if_let!(gen("int()") => Ok(g) => {
            for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -1 && n < 1000 )
               }
        });
        if_let!(gen("int(,10)") => Ok(g) => {
            for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -1 && n < 10 )
               }
        });
    }

    #[test]
    fn random_str_from_list_test() {
        if_let!(gen(r#"str_from_list(a,b,c,d)"#) => Ok(g)
                => if_let!(g.next() => Value::String(el)
                => assert_eq!("abcd".contains(el.as_str()), true)));

        if_let!(gen(r#"str_from_list(,,,)"#) => Ok(g)
                => if_let!(g.next() => Value::String(el)
                => assert_eq!("".contains(el.as_str()), true)));
        if_let!(gen(r#"str_from_list(abc , bca , cdb)"#) => Ok(g)
                => if_let!(g.next() => Value::String(el)
                => assert_eq!("abcbcacdb".contains(el.as_str()), true)));
        if_let!(gen(r#"str_from_list( )"#) => Ok(g)
                => if_let!(g.next() => Value::String(el)
                => assert_eq!(el,"")));
    }

    #[test]
    fn random_int_from_list_test() {
        if_let!(gen(r#"int_from_list(1,2,3)"#) => Ok(g)
                => {
                let n = g.next().as_i64().unwrap();
                assert!(vec![1,2,3].contains(&n));
                });
        if_let!(gen(r#"int_from_list()"#) => Ok(g) => assert_eq!(g.next(),Value::Null));
        if_let!(gen(r#"int_from_list(a,b,c)"#) => Err(e)
                => {
                assert!(e.to_string().contains("int_from_list(a,b,c)"));
                })
    }

    #[test]
    fn random_str_from_file_test() {
        if_let!(gen(r#"str_from_file(jsons/cities, \n)"#)
                => Ok(g)
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!("BerlinPragueMoscowLondonHelsinkiRomeBarcelonaViennaAmsterdamDublin"
                                   .contains(el.as_str()), true)));
        if_let!(gen(r#"str_from_file(jsons/numbers,,)"#)
                => Ok(g)
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!("123456789"
                                   .contains(el.as_str()), true)));

        if_let!(gen(r#"str_from_file()"#) => Err(el) => assert!(el.to_string().contains("str_from_file")));
        if_let!(gen(r#"str_from_file(f,)"#) => Err(el) => assert!(el.to_string().contains("str_from_file")));
    }

    #[test]
    fn random_int_from_file_test() {
        if_let!(gen(r#"int_from_file(jsons/numbers_negate,,)"#)
                => Ok(g)
                => {
                for _ in (1..100).into_iter(){
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -4 && n < 4)
                }
                });

        if_let!(gen(r#"int_from_file(jsons/numbers,,)"#)
                => Ok(g)
                => {
                let n = g.next().as_i64().unwrap();
                assert!(n > 0 && n < 10)
                });
        if_let!(gen(r#"int_from_file(jsons/cities, \n)"#)
                => Err(e)
                => assert!(e.to_string().contains("int_from_file")));
        if_let!(gen(r#"int_from_file()"#) => Err(el) => assert!(el.to_string().contains("int_from_file")));
    }

    #[test]
    fn random_array_test() {
        if_let!(
        gen(r#"int_from_list(1,2,3,4) -> array()"#) =>  Ok(g)
            => if_let!(g.next() => Value::Array(elems) => {
             assert_eq!(elems.len(),1);
             assert!(!elems.get(0).unwrap().is_null());
            elems.iter().flat_map(|e|e.as_i64()).for_each(|e|assert!(e > 0 && e < 5))
            }));

        if_let!(
        generator(r#"int_from_list(1,2,3,4) -> array(3)"#) =>  Ok(g)
            => if_let!(g.next() => Value::Array(elems) => {
            assert!(!elems.get(0).unwrap().is_null());
            assert_eq!(elems.len(),3);
            elems
            .iter()
            .flat_map(|e|e.as_i64())
            .for_each(|e|assert!(e > 0 && e < 5))
            }));

        if_let!(
        generator(r#"str_from_list(aaa,'bbb',ccc) -> array(3)"#) =>  Ok(g)
            => if_let!(g.next() => Value::Array(elems) => {
            elems
            .iter()
            .flat_map(|e|e.as_str())
            .for_each(|e|assert!("aaa'bbb'ccc".contains(e)))
            }));

        if_let!(
        generator(r#"seq(1) -> array(3) -> array()"#) =>  Ok(g)
            => if_let!(g.next() => Value::Array(elems) => {
            assert_eq!(elems.len(),1);
            assert!(!elems.get(0).unwrap().is_null());
            elems
            .iter()
            .flat_map(|e|e.as_array())
            .for_each(|e|{
                assert_eq!(e.len(),3);
                assert_eq!(e,json!([2,3,4]).as_array().unwrap());
            })
            }));

        // if_let!(
        // generator(r#"seq(1) -> array(3) -> array() -> seq(1)"#) => Err(GenError{reason})
        //     => {
        //     println!("{}",reason);
        //     assert!(reason.contains("the functions are unable to merge in the order"))
        //     });
    }
}