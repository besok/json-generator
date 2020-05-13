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
use crate::generator::{Gen, new, Generator};
use crate::generator::generators::{Sequence, UUID};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;


fn sequence(i: &str) -> IResult<&str, Gen<Sequence>> {
    preceded(tag("sequence("),
             terminated(
                 map_res(take_while1(char::is_numeric),
                         |s: &str| {
                             let val: i64 = s.parse()?;
                             let res: Result<Gen<Sequence>, ParseIntError> =
                                 Ok(new(Sequence { val: val as usize }));
                             res
                         })
                 , char('i'),
             ),
    )(i)
}

fn uuid(i: &str) -> IResult<&str, Gen<UUID>> {
    map_res(tag("uuid()"), |_: &str| {
        res(new(UUID{}))
    })(i)
}

#[derive(Debug)]
pub struct GenError {}

impl Error for GenError{}

impl Display for GenError{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("error while parsing a generator func");
        Ok(())
    }
}

fn res<T:Generator>(entity: Gen<T>) -> Result<Gen<T>, GenError>{
    Ok(entity)
}

#[cfg(test)]
mod tests {
    use crate::parser::generator::{sequence,uuid};
    use crate::parser::Json;
    use crate::generator::next;

    #[test]
    fn seq_gen_test() {
        if let Ok((_, el)) = sequence("sequence(10)") {
            assert_eq!(Json::Num(10), next(el.clone()));
            assert_eq!(Json::Num(11), next(el.clone()));
            assert_eq!(Json::Num(12), next(el.clone()));
        }
    }

    #[test]
    fn uuid_test(){
        if let Ok((_, el)) = uuid("uuid()") {
            if let Json::Str(el) = next(el.clone()){
                assert_eq!(el.len(),36)
            }

        }
    }

}