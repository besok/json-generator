use std::fmt::{Debug, Formatter, Error};
use crate::generator::{GeneratorFunc, Generator};
use std::rc::Rc;
use std::cell::RefCell;

pub mod json;
pub mod generator;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Num(i64),
    Str(String),
    Bool(bool),
    Null,
    Object(Vec<Field>),
    Array(Vec<Json>),
}


#[derive( Clone )]
pub struct Field {
    name: String,
    value: Json,
    g: Option<Generator>,
}

impl Debug for Field{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.name.as_str());
        f.write_str(":");
        f.write_str(self.value.to_string().as_str())
    }
}

impl PartialEq for Field{
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value

    }
}

impl Field {
    fn new(name: String, value: Json) -> Self {
        Field { name, value, g: None }
    }
}

impl ToString for Field {
    fn to_string(&self) -> String {
        format!(r#""{}":{}"#, self.name, self.value.to_string())
    }
}

impl ToString for Json {
    fn to_string(&self) -> String {
        match self {
            Json::Num(e) => format!("{}", e),
            Json::Str(e) => format!(r#""{}""#, e),
            Json::Bool(e) => format!(r#"{}"#, e),
            Json::Null => format!("null"),
            Json::Object(v) => format!(r#"{{{}}}"#, vec_to_string(v)),
            Json::Array(v) => format!(r#"[{}]"#, vec_to_string(v)),
        }
    }
}

fn vec_to_string<V: ToString>(arr: &Vec<V>) -> String {
    arr.iter().map(ToString::to_string).fold(String::new(), join)
}

fn join(a: String, b: String) -> String {
    if a.is_empty() { format!("{}", b) } else { format!("{},{}", a, b) }
}


#[cfg(test)]
mod tests {
    use crate::parser::{Field, Json};

    #[test]
    fn field_test() {
        let f = Field::new("name".to_string(), Json::Null);
        assert_eq!("\"name\":null", f.to_string())
    }

    #[test]
    fn value_test() {
        let null = Json::Null;
        assert_eq!("null", null.to_string());

        let num = Json::Num(10);
        assert_eq!("10", num.to_string());

        let bool = Json::Bool(true);
        assert_eq!("true", bool.to_string());

        let string = Json::Str("string".to_string());
        assert_eq!(r#""string""#, string.to_string());

        let object = Json::Object(vec![
            Field::new("one".to_string(), Json::Object(vec![Field::new("under_one".to_string(), Json::Null)])),
            Field::new("two".to_string(), Json::Object(vec![Field::new("under_two".to_string(), Json::Num(100))])),
            Field::new("one".to_string(), Json::Bool(false)),
        ]);

        assert_eq!(r#"{"one":{"under_one":null},"two":{"under_two":100},"one":false}"#, object.to_string());

        let arr = Json::Array(vec![
            Json::Object(vec![Field::new("under_one".to_string(), Json::Null)]),
            Json::Object(vec![Field::new("under_one".to_string(), Json::Null)]),
            Json::Object(vec![Field::new("under_one".to_string(), Json::Null)])
        ]);
        assert_eq!(r#"[{"under_one":null},{"under_one":null},{"under_one":null}]"#, arr.to_string());
    }
}