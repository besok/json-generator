pub mod parser;

#[derive(Clone, Debug,  PartialEq)]
enum Value {
    Num(i64),
    Str(String),
    Bool(bool),
    Null,
}
impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Num(el) => format!("{}", el),
            Value::Str(el) => format!(r#""{}""#, el),
            Value::Bool(el) => format!("{}", el),
            Value::Null => String::from("null"),
        }
    }
}

#[derive(Clone)]
enum Bounder {
    Default
}

#[derive(Clone)]
enum Json {
    Field(String, Value, Bounder),
    Array(String, Vec<Value>, Bounder),
    Object(String, Vec<Json>),
}

enum JsonObject {
    Entity(Json),
    Array(Vec<Json>),
}

impl ToString for JsonObject {
    fn to_string(&self) -> String {
        match self {
            JsonObject::Entity(e) => format!("{{{}}}", e.to_string()),
            JsonObject::Array(v) =>
                format!("[{}]",
                        vec_to_string(&v
                            .iter()
                            .map(|e| JsonObject::Entity(e.clone()).to_string())
                            .collect()
                        ))
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

impl ToString for Json {
    fn to_string(&self) -> String {
        match self {
            Json::Field(n, v, _) => return format!(r#""{}":{}"#, n, v.to_string()),
            Json::Array(n, v, _) => format!(r#""{}":[{}]"#, n, vec_to_string(v)),
            Json::Object(n, v) => format!(r#""{}":{{{}}}"#, n, vec_to_string(v)),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::{Value, Json, Bounder, JsonObject};

    #[test]
    fn to_string_val() {
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
        assert_eq!(Value::Str(String::from("string")).to_string(), r#""string""#);
        assert_eq!(Value::Str(String::from("")).to_string(), r#""""#);
        assert_eq!(Value::Num(-1).to_string(), "-1");
        assert_eq!(Value::Num(0).to_string(), "0");
    }

    #[test]
    fn to_string_json() {
        let field = Json::Field(String::from("name"), Value::Null, Bounder::Default);
        assert_eq!(field.to_string(), "\"name\":null");

        let field = Json::Field(String::from("name"), Value::Str(String::from("string")), Bounder::Default);
        assert_eq!(field.to_string(), format!(r#""name":"string""#));

        let field = Json::Field(String::from("name"), Value::Num(10), Bounder::Default);
        assert_eq!(field.to_string(), format!(r#""name":10"#));

        let field = Json::Field(String::from("name"), Value::Bool(true), Bounder::Default);
        assert_eq!(field.to_string(), format!(r#""name":true"#));

        let field = Json::Array(String::from("name"), vec![], Bounder::Default);
        assert_eq!(field.to_string(), format!(r#""name":[]"#));

        let field = Json::Array(String::from("name"), vec![Value::Num(10), Value::Num(11), Value::Num(12)], Bounder::Default);
        assert_eq!(field.to_string(), format!(r#""name":[10,11,12]"#));


        let field = Json::Object(String::from("name"),
                                 vec![
                                     Json::Field(String::from("name"), Value::Str(String::from("string")), Bounder::Default),
                                     Json::Array(String::from("children"), vec![Value::Num(10), Value::Num(11), Value::Num(12)], Bounder::Default),
                                     Json::Field(String::from("null"), Value::Null, Bounder::Default),
                                     Json::Field(String::from("age"), Value::Num(10), Bounder::Default),
                                     Json::Object(String::from("addr"), vec![
                                         Json::Field(String::from("str"), Value::Str(String::from("string")), Bounder::Default),
                                         Json::Field(String::from("city"), Value::Str(String::from("string")), Bounder::Default),
                                     ]),
                                 ],
        );
        assert_eq!(field.to_string(), String::from(r#""name":{"name":"string","children":[10,11,12],"null":null,"age":10,"addr":{"str":"string","city":"string"}}"#));
    }

    #[test]
    fn to_string_json_object() {
        let field = Json::Object(String::from("name"),
                                 vec![
                                     Json::Field(String::from("name"), Value::Str(String::from("string")), Bounder::Default),
                                     Json::Array(String::from("children"), vec![Value::Num(10), Value::Num(11), Value::Num(12)], Bounder::Default),
                                     Json::Field(String::from("null"), Value::Null, Bounder::Default),
                                     Json::Field(String::from("age"), Value::Num(10), Bounder::Default),
                                     Json::Object(String::from("addr"), vec![
                                         Json::Field(String::from("str"), Value::Str(String::from("string")), Bounder::Default),
                                         Json::Field(String::from("city"), Value::Str(String::from("string")), Bounder::Default),
                                     ]),
                                 ],
        );

        let js = JsonObject::Entity(field);
        assert_eq!(js.to_string(), String::from(r#"{"name":{"name":"string","children":[10,11,12],"null":null,"age":10,"addr":{"str":"string","city":"string"}}}"#));

        let field1 = Json::Object(String::from("name"),
                                  vec![
                                      Json::Field(String::from("name"), Value::Str(String::from("string")), Bounder::Default),
                                      Json::Array(String::from("children"), vec![Value::Num(10), Value::Num(11), Value::Num(12)], Bounder::Default),
                                      Json::Field(String::from("null"), Value::Null, Bounder::Default),
                                      Json::Field(String::from("age"), Value::Num(10), Bounder::Default),
                                      Json::Object(String::from("addr"), vec![
                                          Json::Field(String::from("str"), Value::Str(String::from("string")), Bounder::Default),
                                          Json::Field(String::from("city"), Value::Str(String::from("string")), Bounder::Default),
                                      ]),
                                  ],
        );
        let field2 = Json::Object(String::from("name"),
                                  vec![
                                      Json::Field(String::from("name"), Value::Str(String::from("string")), Bounder::Default),
                                      Json::Array(String::from("children"), vec![Value::Num(10), Value::Num(11), Value::Num(12)], Bounder::Default),
                                      Json::Field(String::from("null"), Value::Null, Bounder::Default),
                                      Json::Field(String::from("age"), Value::Num(10), Bounder::Default),
                                      Json::Object(String::from("addr"), vec![
                                          Json::Field(String::from("str"), Value::Str(String::from("string")), Bounder::Default),
                                          Json::Field(String::from("city"), Value::Str(String::from("string")), Bounder::Default),
                                      ]),
                                  ],
        );

        let js = JsonObject::Array(vec![field1, field2]);
        assert_eq!(js.to_string(), String::from(r#"[{"name":{"name":"string","children":[10,11,12],"null":null,"age":10,"addr":{"str":"string","city":"string"}}},{"name":{"name":"string","children":[10,11,12],"null":null,"age":10,"addr":{"str":"string","city":"string"}}}]"#));
    }
}