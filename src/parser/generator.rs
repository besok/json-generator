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
use crate::generator::generators::{Sequence, UUID, CurrentDateTime, RandomString, RandomInt, RandomFromFile, RandomFromList, RandomArray};
use crate::generator::from_string::FromStringTo;
use std::error::Error;
use std::fmt::{Display, Formatter, Debug};
use std::num::ParseIntError;
use nom::bytes::complete::is_not;
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

pub fn plain_string(v: &str) -> IResult<&str, &str> {
    preceded(sp, take_while(move |c| c != ')' && c != ','))(v)
}

fn func<'a, F>(label: &'a str, extractor: F) -> impl FnMut(&'a str) -> IResult<&'a str, Generator>
    where F: FnMut(&'a str) -> IResult<&'a str, Generator> {
    preceded(sp, preceded(
        tag(label),
        preceded(
            sp,
            preceded(
                char('('),
                terminated(
                    extractor,
                    preceded(
                        sp, char(')'),
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

fn random_array(i: &str) -> IResult<&str, Generator> {
    func("array", args_string(|elems| {
        match elems[..] {
            [f, it] => new(RandomArray::new(FromStringTo::parse(it, true)?, generator(f)?.1)),
            [f] => new(RandomArray::new(1, generator(f)?.1)),
            _ => Err(GenError::new())
        }
    }))(i)
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


pub fn generator(i: &str) -> IResult<&str, Generator> {
    preceded(sp,
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
                 random_array
             )))(i)
}

fn new<T: GeneratorFunc + 'static>(gf: T) -> Result<Generator, GenError> {
    Ok(Generator::new(gf))
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

impl From<std::io::Error> for GenError {
    fn from(e: std::io::Error) -> Self {
        GenError::new_with(e.to_string())
    }
}

impl From<std::string::String> for GenError {
    fn from(e: std::string::String) -> Self {
        GenError::new_with(e)
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for GenError {
    fn from(e: nom::Err<nom::error::Error<&str>>) -> Self {
        GenError::new_with(e.to_string())
    }
}

impl Display for GenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "error while parsing a generator func, reason: {}", self.reason)
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::generator::{uuid, generator, current_dt, random_string, random_int, random_array, GenError};
    use nom::error::ErrorKind;
    use crate::generator::Generator;
    use serde_json::Value;

    #[test]
    fn current_dt_test() {
        if_let!(current_dt("dt()")
                => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!(19, el.len())));

        if_let!(current_dt(" dt (  ) ")
                => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!(19, el.len())));

        if_let!(current_dt("dt( %Y-%m-%d )")
                => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                    => {
                    println!("{}",el);
                    assert_eq!(10, el.len())
                    }));
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

    #[test]
    fn random_int_test() {
        if_let!(generator("int(0,10)") => Ok((_, g)) => {
             for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n >-1 && n < 11 )
               }
        });

        if_let!(generator("int(-10,11)") => Ok((_, g)) => {
            for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -11 && n < 11 )
               }
        });
        if_let!(generator("int(-10)") => Ok((_, g)) => {
            for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -11 && n < 1000 )
               }
        });
        if_let!(generator("int()") => Ok((_, g)) => {
            for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -1 && n < 1000 )
               }
        });
        if_let!(generator("int(,10)") => Ok((_, g)) => {
            for _ in (0..1000).into_iter() {
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -1 && n < 10 )
               }
        });
    }

    #[test]
    fn random_str_from_list_test() {
        if_let!(generator(r#"str_from_list(a,b,c,d)"#) => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                => assert_eq!("abcd".contains(el.as_str()), true)));

        if_let!(generator(r#"str_from_list(,,,)"#) => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                => assert_eq!("".contains(el.as_str()), true)));
        if_let!(generator(r#"str_from_list(abc , bca , cdb)"#) => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                => assert_eq!("abcbcacdb".contains(el.as_str()), true)));
        if_let!(generator(r#"str_from_list( )"#) => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                => assert_eq!(el,"")));
    }

    #[test]
    fn random_int_from_list_test() {
        if_let!(generator(r#"int_from_list(1,2,3)"#) => Ok((_, g))
                => {
                let n = g.next().as_i64().unwrap();
                assert!(vec![1,2,3].contains(&n));
                });
        if_let!(generator(r#"int_from_list()"#) => Ok((_, g)) => assert_eq!(g.next(),Value::Null));
        if_let!(generator(r#"int_from_list(a,b,c)"#) => Err(e)
                => {
                assert!(e.to_string().contains("int_from_list(a,b,c)"));
                })
    }

    #[test]
    fn random_str_from_file_test() {
        if_let!(generator(r#"str_from_file(jsons/cities, \n)"#)
                => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!("BerlinPragueMoscowLondonHelsinkiRomeBarcelonaViennaAmsterdamDublin"
                                   .contains(el.as_str()), true)));
        if_let!(generator(r#"str_from_file(jsons/numbers,,)"#)
                => Ok((_, g))
                => if_let!(g.next() => Value::String(el)
                    => assert_eq!("123456789"
                                   .contains(el.as_str()), true)));

        if_let!(generator(r#"str_from_file()"#) => Err(el) => assert!(el.to_string().contains("str_from_file")));
        if_let!(generator(r#"str_from_file(f,)"#) => Err(el) => assert!(el.to_string().contains("str_from_file")));
    }

    #[test]
    fn random_int_from_file_test() {
        if_let!(generator(r#"int_from_file(jsons/numbers_negate,,)"#)
                => Ok((_, g))
                => {
                for _ in (1..100).into_iter(){
                 let n = g.next().as_i64().unwrap();
                 assert!(n > -4 && n < 4)
                }
                });

        if_let!(generator(r#"int_from_file(jsons/numbers,,)"#)
                => Ok((_, g))
                => {
                let n = g.next().as_i64().unwrap();
                assert!(n > 0 && n < 10)
                });
        if_let!(generator(r#"int_from_file(jsons/cities, \n)"#)
                => Err(e)
                => assert!(e.to_string().contains("int_from_file")));
        if_let!(generator(r#"int_from_file()"#) => Err(el) => assert!(el.to_string().contains("int_from_file")));
    }

    #[test]
    fn random_array_test() {
        if_let!(
        generator("array(int_from_list(1,2,3,4),3)") =>  Ok((_, el))
            => if_let!(el.next() => Value::Array(elems) => {
            println!("{:?}",elems)
            }));
    }
}