use crate::generator::Generator;
use crate::parser::Json;
use rand::distributions::Alphanumeric;
use rand::prelude::ThreadRng;
use uuid::Uuid;
use rand::Rng;
use chrono::Utc;
use rand::seq::SliceRandom;
use std::fs::File;
use std::io::{Read, Error};

pub struct Null {}

impl Generator for Null {
    fn next(&mut self) -> Json {
        Json::Null
    }
}

pub struct Constant<T> {
    pub value: T
}

impl Generator for Constant<String> {
    fn next(&mut self) -> Json {
        Json::Str(self.value.clone())
    }
}

impl Generator for Constant<i64> {
    fn next(&mut self) -> Json {
        Json::Num(self.value.clone())
    }
}

pub struct Sequence {
    val: usize
}

pub struct UUID {}

impl Generator for UUID {
    fn next(&mut self) -> Json {
        Json::Str(format!("{}", Uuid::new_v4()))
    }
}

impl Generator for Sequence {
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
    fn new(start: i64, end: i64) -> Self {
        RandomInt { start, end, rng: rand::thread_rng() }
    }
}

impl Generator for RandomInt {
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
    fn new(len: usize) -> Self {
        RandomString { len, rng: rand::thread_rng() }
    }
}


impl Generator for RandomString {
    fn next(&mut self) -> Json {
        Json::Str(
            self.rng
                .sample_iter(&Alphanumeric)
                .take(self.len)
                .collect())
    }
}

struct CurrentDateTime {
    format: String
}

impl Generator for CurrentDateTime {
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

struct RandomFromList<T> {
    values: Vec<T>,
    rng: ThreadRng,
}

impl<T> RandomFromList<T> {
    fn new(values: Vec<T>) -> Self {
        RandomFromList { values, rng: rand::thread_rng() }
    }
}

impl Generator for RandomFromList<String> {
    fn next(&mut self) -> Json {
        match self.values.choose(&mut self.rng) {
            None => Json::Null,
            Some(v) => Json::Str(v.clone()),
        }
    }
}

impl Generator for RandomFromList<i64> {
    fn next(&mut self) -> Json {
        match self.values.choose(&mut self.rng) {
            None => Json::Null,
            Some(v) => Json::Num(v.clone()),
        }
    }
}

struct RandomFromFile<T> {
    path: String,
    delim: String,
    g: RandomFromList<T>,
}

//impl<T> RandomFromFile<T> {
//    fn new(path: String, delim: String) -> Self {
//
//    }
//}

fn read_file_into_string(path: String) -> Result<String, Error> {
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;
    Ok(contents)
}


#[cfg(test)]
mod tests {
    use crate::parser::Json;
    use crate::generator::generators::{RandomString, UUID, RandomInt, CurrentDateTime, RandomFromList, read_file_into_string};
    use crate::generator::Generator;
    use std::io::Error;

    #[test]
    fn random_string_test() {
        let mut g = RandomString::new(10);
        if let Json::Str(el) = g.next() {
            assert_eq!(el.len(), 10)
        } else {
            panic!("should be str")
        }
    }

    #[test]
    fn random_uuid_test() {
        let mut g = UUID {};
        if let Json::Str(el) = g.next() {
            assert_eq!(el.len(), 36)
        } else {
            panic!("should be str")
        }
    }

    #[test]
    fn random_int_test() {
        let mut g = RandomInt::new(-1000, 1000);
        if let Json::Num(el) = g.next() {
            assert_eq!(el < 1000, el > -1001);
        } else {
            panic!("should be str")
        }
    }

    #[test]
    fn current_ts_test() {
        let mut x = CurrentDateTime { format: "%Y-%m-%d".to_string() };
        let json1 = x.next();
        let json2 = x.next();
        assert_eq!(json1, json2);

        let mut x = CurrentDateTime { format: "%Y-%m-%d %H:%M:%S".to_string() };
        if let Json::Str(el) = x.next() {
            print!("{}", el);
            assert_eq!(el.len(), 19)
        } else {
            panic!("should be str")
        }
    }

    #[test]
    fn random_from_list_test() {
        let mut g = RandomFromList::new((1..10).collect());
        if let Json::Num(el) = g.next() {
            assert_eq!(el > 0, el < 10)
        } else {
            panic!("should be num")
        }

        let mut g: RandomFromList<i64> = RandomFromList::new(vec![]);
        assert_eq!(g.next(), Json::Null);
    }

    #[test]
    fn test_string_from_file(){
        match read_file_into_string(r#"C:\projects\json-generator\jsons\list.txt"#.to_string()){
            Ok(v) => assert_eq!("1,2,3,4,5,6",v),
            Err(e) => panic!("error {}",e),
        };
    }
}