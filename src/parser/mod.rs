use std::fmt::{Debug, Formatter, Display};
use crate::generator::Generator;
use crate::parser::json::value;
use crate::parser::generator::GenError;
use std::error::Error;
use nom::error::ErrorKind;

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


impl Json {
    fn next(&self) -> Json {
        match self {
            Json::Object(fields) =>
                Json::Object(fields.iter().map(Field::next).collect()),
            Json::Array(js) =>
                Json::Array(js.iter().map(Json::next).collect()),
            r @ _ => r.clone()
        }
    }
}

#[derive(Clone)]
pub struct Field {
    name: String,
    value: Json,
    g: Option<Generator>,
}

impl Debug for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f,"{}:",self.name.as_str());
        write!(f,"{}",self.value.to_string())
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}

impl Field {
    pub fn new(name: String, value: Json) -> Self {
        Field { name, value, g: None }
    }
    fn new_from_gen(name: String, g: Generator) -> Self {
        Field { name, value: g.next(), g: Some(g) }
    }
    fn new_with_gen(name: String, value: Json, g: Generator) -> Self {
        Field { name, value, g: Some(g) }
    }
    fn get_next(&self) -> Option<Json> {
        match &self.g {
            None => None,
            Some(g) => Some(g.next()),
        }
    }
    fn next(&self) -> Field {
        let name = self.name.clone();
        match &self.g {
            None => Field::new(name, self.value.next()),
            Some(g) => Field::new_from_gen(name, g.clone()),
        }
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

#[derive(Debug)]
pub struct ParserError {
    cause: String
}

impl Error for ParserError {}

impl Display for ParserError{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f,"{}",self.cause)
    }
}

pub fn parse_json(json: &str) -> Result<Json,ParserError>{
    match value(json){
        Ok((_,js_res)) => Ok(js_res),
        Err(e) => Err(ParserError{cause:format!("{:?}",e)}),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{Field, Json};
    use crate::generator::Generator;
    use crate::generator::generators::{Sequence, UUID, RandomString, Constant};

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

    #[test]
    fn field_generate_test() {
        let field = Field::new_with_gen("name".to_string(),
                                        Json::Num(1),
                                        Generator::new(Sequence { val: 1 }));
        let new_field = field.next();
        assert_eq!(new_field.value, Json::Num(2));
        let new_field = field.next();
        assert_eq!(new_field.value, Json::Num(3));
    }

    #[test]
    fn json_generate_test() {
        let obj = Json::Object(vec![
            Field::new("p".to_string(),
                       Json::Object(vec![
                           Field::new_with_gen("id".to_string(), Json::Num(1), Generator::new(Sequence { val: 2 }))
                       ])),
            Field::new("c".to_string(), Json::Str("t".to_string())),
            Field::new_with_gen("l".to_string(), Json::Str("t".to_string()), Generator::new(Constant { value: "a".to_string() })),
        ]);

        assert_eq!(obj.next().to_string(), r#"{"p":{"id":3},"c":"t","l":"a"}"#.to_string());
        assert_eq!(obj.next().to_string(), r#"{"p":{"id":4},"c":"t","l":"a"}"#.to_string());
        assert_eq!(obj.next().to_string(), r#"{"p":{"id":5},"c":"t","l":"a"}"#.to_string());
    }
}