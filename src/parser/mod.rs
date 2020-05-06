pub mod parser;

#[derive(Clone, Debug, PartialEq)]
enum Json {
    Num(i64),
    Str(String),
    Bool(bool),
    Null,
    Object(Vec<Field>),
    Array(Vec<Json>),
}

#[derive(Clone, Debug, PartialEq)]
struct Field {
    name: String,
    value: Json,
    generator:Generator,
}

impl Field {
    fn new(name:String, value: Json) -> Self{
        let generator = Generator::Default;
        Field{name,value,generator}
    }
    fn new_with(name:String, value: Json, generator:Generator) -> Self{
        Field{name,value,generator}
    }


}

#[derive(Clone,Debug,PartialEq)]
enum Generator {
    Default,
    Sequence(usize),
    RandomString(usize),
    RandomFromFile(String,String),
    RandomFromList(Vec<>),

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
    arr.iter()
        .map(ToString::to_string)
        .fold(String::new(), join)
}

fn join(a: String, b: String) -> String {
    if a.is_empty() { format!("{}", b) } else { format!("{},{}", a, b) }
}


#[cfg(test)]
mod tests {
    use crate::parser::{Field, Json, Generator};

    #[test]
    fn field_test() {
        let f = Field::new("name".to_string(), Json::Null, );
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
            Field::new ("one".to_string(), Json::Object(vec![Field::new ("under_one".to_string(), Json::Null )]) ),
            Field::new ("two".to_string(), Json::Object(vec![Field::new ("under_two".to_string(), Json::Num(100) )]) ),
            Field ::new("one".to_string(), Json::Bool(false) ),
        ]);

        assert_eq!(r#"{"one":{"under_one":null},"two":{"under_two":100},"one":false}"#, object.to_string());

        let arr = Json::Array(vec![
            Json::Object(vec![Field::new ("under_one".to_string(), Json::Null )]),
            Json::Object(vec![Field::new ("under_one".to_string(), Json::Null )]),
            Json::Object(vec![Field::new ("under_one".to_string(), Json::Null )])
        ]);
        assert_eq!(r#"[{"under_one":null},{"under_one":null},{"under_one":null}]"#, arr.to_string());
    }
}