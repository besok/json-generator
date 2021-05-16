use crate::generator::{GeneratorFunc, Generator};
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
use serde_json::Value;
use std::iter::FromIterator;

pub struct Null {}

impl GeneratorFunc for Null {
    fn next_value(&mut self) -> Value {
        Value::Null
    }
}

pub struct UUID {}

impl GeneratorFunc for UUID {
    fn next_value(&mut self) -> Value {
        Value::from(format!("{}", Uuid::new_v4()))
    }
}

pub struct Sequence {
    pub val: i32,
    pub step: i32,
}


impl GeneratorFunc for Sequence {
    fn next_value(&mut self) -> Value {
        self.val += self.step;
        Value::from(self.val)
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
    fn next_value(&mut self) -> Value {
        Value::from(
            self.rng.gen_range(self.start, self.end)
        )
    }
}

pub struct RandomString {
    len: usize,
    rng: ThreadRng,
    prefix: String,
    postfix: String,
}

impl RandomString {
    pub fn new_with(len: usize, prefix: String, postfix: String) -> Self {
        RandomString {
            len,
            prefix,
            postfix,
            rng: rand::thread_rng(),
        }
    }
    pub fn new(len: usize) -> Self {
        RandomString {
            len,
            rng: rand::thread_rng(),
            prefix: String::new(),
            postfix: String::new(),
        }
    }
}


impl GeneratorFunc for RandomString {
    fn next_value(&mut self) -> Value {
        let random_str = String::from_iter(
            self.rng
                .sample_iter(&Alphanumeric)
                .take(self.len)
                .into_iter());
        Value::from(format!("{}{}{}", self.prefix, random_str, self.postfix))
    }
}

pub struct CurrentDateTime {
    pub format: String
}

impl GeneratorFunc for CurrentDateTime {
    fn next_value(&mut self) -> Value {
        let time = Utc::now();
        Value::from(
            if self.format.is_empty() {
                time.to_string()
            } else {
                time.format(self.format.as_str()).to_string()
            }
        )
    }
}

pub struct RandomFromList<T: Into<Value>> {
    values: Vec<T>,
    rng: ThreadRng,
}

impl<T: Into<Value> + Clone> RandomFromList<T> {
    pub fn new(values: Vec<T>) -> Self {
        RandomFromList { values, rng: rand::thread_rng() }
    }
}


impl<T> GeneratorFunc for RandomFromList<T>
    where T: Into<Value> + Clone {
    fn next_value(&mut self) -> Value {
        match self.values.choose(&mut self.rng) {
            None => Value::Null,
            Some(v) => v.clone().into(),
        }
    }
}

//todo in general with small files, having them in the memory is fine but if it is going to be a pitfall,
// better to keep a path and read the file randomly.
pub struct RandomFromFile<T: FromStr + Clone + Into<Value>>
    where <T as FromStr>::Err: Debug {
    path: String,
    delim: String,
    delegate: RandomFromList<T>,
}

impl<T: FromStr + Clone + Into<Value>> RandomFromFile<T>
    where <T as FromStr>::Err: Debug {
    pub fn new(path: &str, delim: &str) -> Result<Self, Error> {
        let values = process_string(read_file_into_string(path)?, delim);
        let rng = Default::default();
        Ok(
            RandomFromFile {
                path: String::from(path),
                delim: String::from(delim),
                delegate: RandomFromList { values, rng },
            }
        )
    }
}

impl<T: Clone + FromStr + Into<Value>> GeneratorFunc for RandomFromFile<T>
    where <T as FromStr>::Err: Debug {
    fn next_value(&mut self) -> Value {
        self.delegate.next_value()
    }
}

fn process_string<T: FromStr>(v: String, d: &str) -> Vec<T>
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
    delegate: Generator,
}

impl RandomArray {
    pub fn new(len: usize, delegate: Generator) -> Self {
        RandomArray { len, delegate }
    }
}

impl GeneratorFunc for RandomArray {
    fn next_value(&mut self) -> Value {
        Value::Array(
            (0..self.len).map(|_| self.delegate.next()).collect()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::generators::{RandomString, UUID, RandomInt, CurrentDateTime, RandomFromList, read_file_into_string, process_string, RandomFromFile, RandomArray, Null, Sequence};
    use crate::generator::{GeneratorFunc, Generator};
    use std::io::Error;
    use serde_json::Value;
    use std::env;

    fn gen<T: GeneratorFunc + 'static>(f: T) -> Generator {
        Generator::new(f)
    }

    #[test]
    fn null_test() {
        if_let!(gen(Null{}).next() => Value::Null => ())
    }

    #[test]
    fn random_uuid_test() {
        if_let!(gen(UUID {}).next() => Value::String(el) => assert_eq!(el.len(), 36));
    }

    #[test]
    fn sequence_test() {
        let g1 = gen(Sequence { val: 1, step: 2 });

        assert_eq!(g1.next().as_i64(), Some(3));
        assert_eq!(g1.next().as_i64(), Some(5));

        let g1 = gen(Sequence { val: 1, step: -1 });

        assert_eq!(g1.next().as_i64(), Some(0));
        assert_eq!(g1.next().as_i64(), Some(-1));
    }

    #[test]
    fn random_int_test() {
        let g = gen(RandomInt::new(-1000, 1000));

        if_let!(g.next().as_i64() => Some(el) => assert!(el >= -1000 && el <= 1000));
        if_let!(g.next().as_i64() => Some(el) => assert!(el >= -1000 && el <= 1000));
        if_let!(g.next().as_i64() => Some(el) => assert!(el >= -1000 && el <= 1000));
    }

    #[test]
    fn random_string_test() {
        if_let!(gen(RandomString::new(10)).next() => Value::String(el) => assert_eq!(el.len(), 10));
        let g = gen(RandomString::new_with(10, "abc".to_string(), "cba".to_string()));
        if_let!(
            g.next() => Value::String(el) => {
                assert_eq!(el.len(), 16);
                assert!(el.starts_with("abc"));
                assert!(el.ends_with("cba"));
            }
       );
    }

    #[test]
    fn current_ts_test() {
        let mut x = gen(CurrentDateTime { format: "%Y-%m-%d".to_string() });
        if_let!(x.next() => x.next() => ());

        let mut x = CurrentDateTime { format: "%Y-%m-%d %H:%M:%S".to_string() };
        if_let!(x.next_value() => Value::String(el) => {
            print!("{}", el);
            assert_eq!(el.len(), 19);
        });
    }

    #[test]
    fn random_from_list_test() {
        let mut gn = gen(RandomFromList::new((1..10).collect()));
        if_let!(gn.next() => Value::Number(el) =>  {
            let el = el.as_i64().unwrap();
            assert_eq!(el > 0, el < 10)
        });

        let mut gf: RandomFromList<i64> = RandomFromList::new(vec![]);
        let mut gen = gen(gf);
        assert_eq!(gen.next(), Value::Null);
    }

    #[test]
    fn from_file_test() {
        let g = RandomFromFile::<i64>::new(r#"jsons/numbers"#, ",")
            .map(|f| Generator::new(f));

        if_let!(g => Ok(mut g) => if_let!(g.next() => Value::Number(el) => {
            let el = el.as_i64().unwrap();
            assert!(el > 0 && el < 10)
        }));

        let g = RandomFromFile::<String>::new(r#"jsons/cities"#, "\n")
            .map(|f| Generator::new(f));

        if_let!(g => Ok(mut g) => if_let!(g.next() => Value::String(city) => {
            assert!("BerlinPragueMoscowLondonHelsinkiRomeBarcelonaViennaAmsterdamDublin".contains(&city))
        }));
    }

    #[test]
    fn read_file_from_path() {
        match read_file_into_string("jsons/numbers") {
            Ok(f) => assert_eq!(f, "1,2,3,4,5,6,7,8,9"),
            Err(e) => panic!("error {}", e)
        };
    }

    #[test]
    fn from_string_test() {
        let vec = process_string::<String>("a,b,c".to_string(), ",");
        assert_eq!(vec, vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let vec = process_string::<i32>("1,2,3".to_string(), ",");
        assert_eq!(vec, vec![1, 2, 3]);

        let vec = process_string::<i32>("1,c,3".to_string(), ",");
        assert_eq!(vec, vec![1, 3]);
    }

    #[test]
    fn array_test() {
        let g_int = gen(RandomInt::new(1, 100));
        let gen = gen(RandomArray::new(3, g_int));

        if_let!(
            gen.next() => Value::Array(elems) => {
                assert_eq!(elems.len(), 3);
                for e in elems.into_iter() {
                    if_let!(e => Value::Number(el) => {
                    let el = el.as_i64().unwrap();
                    assert_eq!(el > 0 && el < 100, true)
                    })
                }
            }
        );
    }
}