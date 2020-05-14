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
use crate::generator::generators::{Sequence, UUID, CurrentDateTime};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use crate::parser::json::sp;


fn sequence(i: &str) -> IResult<&str, Generator> {
    preceded(tag("sequence("),
             terminated(
                 map_res(take_while1(char::is_numeric),
                         |s: &str| {
                             let val: i64 = s.parse()?;
                             let res: Result<Generator, ParseIntError> =
                                 Ok(Generator::new(Sequence { val: val as usize }));
                             res
                         })
                 , char(')'),
             ),
    )(i)
}

fn uuid(i: &str) -> IResult<&str, Generator> {
    map_res(tag("uuid()"), |_: &str| {
        res(UUID {})
    })(i)
}

fn generator(i: &str) -> IResult<&str, Generator> {
    preceded(sp,
             alt((
                 uuid, sequence
             )))(i)
}


fn res<T: GeneratorFunc + 'static>(gf: T) -> Result<Generator, GenError> {
    Ok(Generator::new(gf))
}


#[derive(Debug)]
pub struct GenError {}

impl Error for GenError {}

impl Display for GenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("error while parsing a generator func");
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::generator::{sequence, uuid, generator};
    use crate::parser::{Json, Field};

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
        if let Ok((_, el)) = uuid("uuid()") {
            let field = Field { g: Some(el.get()), name: "".to_string(), value: Json::Null };
            if let Json::Str(el) = el.next() {
                assert_eq!(el.len(), 36)
            }
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