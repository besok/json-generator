//! The module which is responsible to send generated jsons to different sources
//! It provides the trait Sender to work with
use crate::parser::{Json, Field};

pub mod http;
pub mod file;

#[cfg(windows)]
const S: &'static str = "\r\n";
#[cfg(not(windows))]
const S: &'static str = "\n";

/// The basic trait using to send the json to a specific location related to the implementation
pub trait Sender {
    fn send(&mut self, json: String) -> Result<String, String>;
    fn send_pretty(&mut self, delegate: Json) -> Result<String, String> {
        self.send(PrettyJson { delegate }.to_string())
    }
}

trait PrettyString {
    fn to_pretty_string(&self, level: i16) -> String;
}

impl PrettyString for Json {
    fn to_pretty_string(&self, level: i16) -> String {
        match self {
            Json::Object(fields) => format!("{}", fields.to_pretty_string(level + 1)),
            Json::Array(values) => format!("{}", values.to_pretty_string(level + 1)),
            el @ _ => el.to_string(),
        }
    }
}

impl PrettyString for Vec<Json> {
    fn to_pretty_string(&self, level: i16) -> String {
        format!("[{}{}{}]",
                self.iter()
                    .map(|j| j.to_pretty_string(level + 1))
                    .fold(String::new(),
                          |a, b|
                              match (a.as_str(), b.as_str()) {
                                  ("", b) => format!("{}{}{}", S, spaces(level), b),
                                  (a, "") => format!("{}", a),
                                  (a, b) => format!("{},{}{}{}", a, S, spaces(level), b),
                              }), S, spaces(level - 2)
        )
    }
}

impl PrettyString for Vec<Field> {
    fn to_pretty_string(&self, level: i16) -> String {
        format!("{{{}{}{}}}",
                self.iter()
                    .map(|j| j.to_pretty_string(level + 1))
                    .fold(String::new(),
                          |a, b|
                              match (a.as_str(), b.as_str()) {
                                  ("", b) => format!("{}", b),
                                  (a, "") => format!("{}", a),
                                  (a, b) => format!("{},{}", a, b),
                              }), S, spaces(level - 2)
        )
    }
}

impl PrettyString for Field {
    fn to_pretty_string(&self, level: i16) -> String {
        format!(r#"{}{}"{}": {}"#,
                S, spaces(level), self.name,
                self.value.to_pretty_string(level + 1))
    }
}

fn spaces(l: i16) -> String {
    let mut s = String::new();
    if l < 0 {
        return s
    }
    for _ in 0..l {
        s.push(' ');
    }
    s
}

/// The struct to transform  a raw json(which is essentially a string)
/// to the string including some elements of formatting.
pub struct PrettyJson {
    delegate: Json
}

pub struct ConsoleSender {}

impl Sender for ConsoleSender {
    fn send(&mut self, json: String) -> Result<String, String> {
        println!("{}", json);
        Ok("item has been sent to console".to_string())
    }
}

impl PrettyJson {
    pub fn new(delegate: Json) -> Self {
        PrettyJson { delegate }
    }
}

impl ToString for PrettyJson {
    fn to_string(&self) -> String {
        self.delegate.to_pretty_string(0)
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::{Json, Field, parse_json, ParserError};
    use crate::sender::PrettyJson;

    //todo  tests will occur to fall in linux.
    #[test]
    fn pretty_to_string_test() {
        let js = r#"{"person":{"id":{"f":100},"name":"Eli\"za\"beth","surname":"E","age":10,"children":[3,6],"address":{"street":"Grip","house":10,"city":"Berlin"}}} "#;
        match parse_json(js) {
            Ok(json) => {
                let pretty_js = PrettyJson::new(json).to_string();
                let pretty_js = pretty_js.replace("\r\n","\n");
                assert_eq!(pretty_js,
                r#"{
  "person": {
     "id": {
        "f": 100
     },
     "name": "Eli\"za\"beth",
     "surname": "E",
     "age": 10,
     "children": [
       3,
       6
     ],
     "address": {
        "street": "Grip",
        "house": 10,
        "city": "Berlin"
     }
  }
}"#);
            }
            Err(e) => panic!("{}", e),
        };
    }
}