#[derive(Debug)]
enum Value{
    Num(i64),
    Str(String),
    Bool(bool),
    Null,
}

impl ToString for Value{
    fn to_string(&self) -> String {
        match self {
            Value::Num(el) => format!("{}",el),
            Value::Str(el) => format!(r#""{}""#,el),
            Value::Bool(el) => format!("{}",el),
            Value::Null => String::from("null"),
        }
    }
}


enum Bounder{
    Null
}

enum Json{
    Field(String, Value,Bounder),
    Array(String, Vec<Value>,Bounder),
    Object(String, Vec<Json>),
}


fn vecToString(arr: &Vec<Value>) -> String {
    arr.iter().map(ToString::to_string).fold(String::new(),|a,b| if a == "".to_string() {format!("{}",b)} else {format!("{},{}",a,b)})
}

impl ToString for Json{
    fn to_string(&self) -> String {
        match self {
            Json::Field(n, v, _) => return format!(r#""{}":{}"#,n,v.to_string()),
            Json::Array(n, v, _) => format!(r#""{}":[{}]"#,n,vecToString(v)),
            Json::Object(n, v) => String::from(""),
        }
    }
}


#[cfg(test)]
mod tests{
    use crate::parser::{Value, Json, Bounder};

    #[test]
    fn to_string_val(){
        assert_eq!(Value::Null.to_string(),"null");
        assert_eq!(Value::Bool(true).to_string(),"true");
        assert_eq!(Value::Bool(false).to_string(),"false");
        assert_eq!(Value::Str(String::from("string")).to_string(),r#""string""#);
        assert_eq!(Value::Str(String::from("")).to_string(),r#""""#);
        assert_eq!(Value::Num(-1).to_string(),"-1");
        assert_eq!(Value::Num(0).to_string(),"0");
    }

    #[test]
    fn to_string_json(){
        let field = Json::Field(String::from("name"), Value::Null,Bounder::Null);
        assert_eq!(field.to_string(), "\"name\":null");

        let field = Json::Field(String::from("name"), Value::Str(String::from("string")),Bounder::Null);
        assert_eq!(field.to_string(), format!(r#""name":"string""#));

        let field = Json::Field(String::from("name"), Value::Num(10),Bounder::Null);
        assert_eq!(field.to_string(), format!(r#""name":10"#));

        let field = Json::Field(String::from("name"), Value::Bool(true),Bounder::Null);
        assert_eq!(field.to_string(), format!(r#""name":true"#));

        let field = Json::Array(String::from("name"),vec![],Bounder::Null);
        assert_eq!(field.to_string(), format!(r#""name":[]"#));

        let field = Json::Array(String::from("name"),vec![Value::Num(10),Value::Num(11),Value::Num(12)],Bounder::Null);
        assert_eq!(field.to_string(), format!(r#""name":[10,11,12]"#));

    }

}