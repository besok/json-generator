use crate::generator::{GeneratorFunc, Generator};
use crate::parser::Json;
use rand::distributions::Alphanumeric;
use rand::prelude::ThreadRng;
use uuid::Uuid;
use rand::Rng;
use chrono::Utc;
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::{Read, Error, ErrorKind};
use std::str::FromStr;
use std::num::ParseIntError;
use std::string::ParseError;
use std::fmt::Debug;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Null {}

impl GeneratorFunc for Null {
    fn next(&mut self) -> Json {
        Json::Null
    }
}

pub struct Constant<T: Into<Json> + Clone> {
    pub value: T
}

impl Into<Json> for String {
    fn into(self) -> Json {
        Json::Str(self.clone())
    }
}

impl Into<Json> for i64 {
    fn into(self) -> Json {
        Json::Num(self.clone())
    }
}

impl<T: Into<Json> + Clone> GeneratorFunc for Constant<T> {
    fn next(&mut self) -> Json {
        self.value.clone().into()
    }
}

pub struct Sequence {
    pub val: usize
}

pub struct UUID {}

impl GeneratorFunc for UUID {
    fn next(&mut self) -> Json {
        Json::Str(format!("{}", Uuid::new_v4()))
    }
}

impl GeneratorFunc for Sequence {
    fn next(&mut self) -> Json {
        self.val += 1;
        Json::Num(self.val as i64)
    }
}

pub struct RandomInt {
    start: i64,
    end: i64,
    rng: ThreadRng,
}

impl RandomInt {
    pub fn new(start: i64, end: i64) -> Self {
        RandomInt { start, end, rng: rand::thread_rng() }
    }
}

impl GeneratorFunc for RandomInt {
    fn next(&mut self) -> Json {
        Json::Num(
            self.rng.gen_range(self.start, self.end)
        )
    }
}

pub struct RandomString {
    len: usize,
    rng: ThreadRng,
}

impl RandomString {
    pub fn new(len: usize) -> Self {
        RandomString { len, rng: rand::thread_rng() }
    }
}


impl GeneratorFunc for RandomString {
    fn next(&mut self) -> Json {
        Json::Str(
            self.rng
                .sample_iter(&Alphanumeric)
                .take(self.len)
                .collect())
    }
}

pub struct CurrentDateTime {
    pub format: String
}

impl GeneratorFunc for CurrentDateTime {
    fn next(&mut self) -> Json {
        let time = Utc::now();
        Json::Str(
            if self.format.is_empty() {
                time.to_string()
            } else {
                time.format(self.format.as_str()).to_string()
            }
        )
    }
}

pub struct RandomFromList<T: Into<Json> + Clone> {
    values: Vec<T>,
    rng: ThreadRng,
}

impl<T: Into<Json> + Clone> RandomFromList<T> {
    pub fn new(values: Vec<T>) -> Self {
        RandomFromList { values, rng: rand::thread_rng() }
    }
}


impl<T> GeneratorFunc for RandomFromList<T>
    where T: Into<Json> + Clone {
    fn next(&mut self) -> Json {
        match self.values.choose(&mut self.rng) {
            None => Json::Null,
            Some(v) => v.clone().into(),
        }
    }
}

pub struct RandomFromFile<T: FromStr + Clone + Into<Json>>
    where <T as FromStr>::Err: Debug {
    path: String,
    delim: String,
    g: RandomFromList<T>,
}

impl<T: FromStr + Clone + Into<Json>> RandomFromFile<T>
    where <T as FromStr>::Err: Debug {
    pub fn new(path: &str, delim: &str) -> Result<Self, Error> {
        let values = from_string(read_file_into_string(path)?, delim);
        Ok(
            RandomFromFile {
                path: path.to_string(),
                delim: delim.to_string(),
                g: RandomFromList { values, rng: Default::default() },
            }
        )
    }
}

impl<T: Clone + FromStr + Into<Json>> GeneratorFunc for RandomFromFile<T>
    where <T as FromStr>::Err: Debug {
    fn next(&mut self) -> Json {
        self.g.next()
    }
}

fn from_string<T: FromStr>(v: String, d: &str) -> Vec<T>
    where <T as FromStr>::Err: Debug {
    let mut del = match d.trim() {
        r#"\r\n"# => "\r\n",
        r#"\n"# => "\n",
        r#"\r"# => "\r",
        r#"\n\r"# => "\n\r",
        _ => d
    };

    v.split(del)
        .map(FromStr::from_str)
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .collect()
}

pub fn read_file_into_string(path: &str) -> Result<String, Error> {
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;
    Ok(contents)
}


pub struct RandomArray {
    len: usize,
    g: Generator,
}

impl RandomArray {
    pub fn new(len: usize, g: Generator) -> Self {
        RandomArray { len, g }
    }
}

impl GeneratorFunc for RandomArray {
    fn next(&mut self) -> Json {
        Json::Array(
            (0..self.len).map(|_| self.g.next()).collect()
        )
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::Json;
    use crate::generator::generators::{RandomString, UUID, RandomInt, CurrentDateTime, RandomFromList,
                                       read_file_into_string, from_string, RandomFromFile, RandomArray};
    use crate::generator::{GeneratorFunc, Generator};
    use std::io::Error;

    #[test]
    fn array_test() {
        let g_int = Generator::new(RandomInt::new(1, 100));
        let gen = Generator::new(RandomArray::new(3, g_int));

        if_let!(
            gen.next() => Json::Array(v) => {
                assert_eq!(v.len(), 3);
                for e in v.into_iter() {
                    if_let!(e => Json::Num(el) => assert_eq!(el > 0 && el < 100, true))
                }
            }
        );
    }

    #[test]
    fn random_string_test() {
        if_let!(RandomString::new(10).next() => Json::Str(el) => assert_eq!(el.len(), 10));
    }

    #[test]
    fn random_uuid_test() {
        let mut g = UUID {};
        if_let!(g.next() => Json::Str(el) => assert_eq!(el.len(), 36));
    }


    #[test]
    fn random_int_test() {
        if_let!(RandomInt::new(-1000, 1000).next() => Json::Num(el) =>  assert_eq!(el < 1000, el > -1001));
    }

    #[test]
    fn current_ts_test() {
        let mut x = CurrentDateTime { format: "%Y-%m-%d".to_string() };
        let json1 = x.next();
        let json2 = x.next();
        assert_eq!(json1, json2);

        let mut x = CurrentDateTime { format: "%Y-%m-%d %H:%M:%S".to_string() };
        if_let!(x.next() => Json::Str(el) =>  {
            print!("{}", el);
            assert_eq!(el.len(), 19);
        });
    }

    #[test]
    fn random_from_list_test() {
        let mut g = RandomFromList::new((1..10).collect());
        if_let!(g.next() => Json::Num(el) =>  assert_eq!(el > 0, el < 10));

        let mut g: RandomFromList<i64> = RandomFromList::new(vec![]);
        assert_eq!(g.next(), Json::Null);
    }

    #[test]
    fn test_string_from_file() {
        match read_file_into_string(r#"C:\projects\json-generator\jsons\list.txt"#) {
            Ok(v) => assert_eq!("1,2,3,4,5,6", v),
            Err(e) => panic!("error {}", e),
        };
    }

    #[test]
    fn from_string_test() {
        let vec = from_string::<String>("a,b,c".to_string(), ",");
        assert_eq!(vec, vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let vec = from_string::<i32>("1,2,3".to_string(), ",");
        assert_eq!(vec, vec![1, 2, 3]);

        let vec = from_string::<i32>("1,c,3".to_string(), ",");
        assert_eq!(vec, vec![1, 3]);
    }

    #[test]
    fn from_file_test() {
        let r = RandomFromFile::<i64>::new(r#"C:\projects\json-generator\jsons\list.txt"#, ",");
        if_let!(r => Ok(mut g)
            => if_let!(g.next() => Json::Num(el) => assert!(el > 0 && el < 7)));
    }
}