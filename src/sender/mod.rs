use crate::parser::Json;

#[cfg(windows)]
const S: &'static str = "\r\n";
#[cfg(not(windows))]
const S: &'static str = "\n";


pub struct PrettyJson {
    delegate: Json
}

impl PrettyJson {
    pub fn new(delegate: Json) -> PrettyJson {
        PrettyJson { delegate }
    }
}

impl ToString for PrettyJson {
    fn to_string(&self) -> String {
        match &self.delegate {
            obj @ Json::Object(_) => nl_curly_bracket(nl_semicolon(obj.to_string())),
            arr @ Json::Array(_) => nl_square_bracket(nl_semicolon(arr.to_string())),
            prm @ _ => prm.to_string(),
        }
    }
}

fn nl_semicolon(str: String) -> String {
    let mut res = str;
    res = res.replace(",", format!(",{}", S).as_str());
    res.push_str(S);
    res
}

fn nl_curly_bracket(str: String) -> String {
    let mut res = str;
    res = res.replace("{", format!("{{{}", S).as_str());
    res = res.replace("}", format!("{}}}{}", S, S).as_str());
    res
}

fn nl_square_bracket(str: String) -> String {
    let mut res = str;
    res = res.replace("[", format!("[{}", S).as_str());
    res = res.replace("]", format!("{}]{}", S, S).as_str());
    res
}

#[cfg(test)]
mod tests {
    use crate::parser::{Json, Field, parse_json, ParserError};
    use crate::sender::{PrettyJson, nl_semicolon, nl_curly_bracket};

    #[test]
    fn pretty_to_string_test() {
        let json = Json::Object(vec![
            Field::new("name".to_string(), Json::Null),
        ]);
        let pj = PrettyJson { delegate: json };
        let string = pj.to_string();
        print!("{}", string);
        assert_eq!("{\r\n\"name\":null\r\n}\r\n\r\n", string);

        let js = r#"
        {
  "person": {
    "id": 1,
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
}
        "#;
        match parse_json(js) {
            Ok(json) => {
                let pretty_js = PrettyJson::new(json).to_string();
                print!("{}",pretty_js);
                assert_eq!("",pretty_js)
            },
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn add_nl_after_semicolon_test() {
        let str = "a,b,c".to_string();
        let res_str = nl_semicolon(str);
        assert_eq!(res_str, "a,\r\nb,\r\nc\r\n");
    }

    #[test]
    fn add_nl_after_bracket_test() {
        let str = "{a{b},c}".to_string();
        let res_str = nl_curly_bracket(str);
        assert_eq!(res_str, "{\r\na{\r\nb\r\n}\r\n,c\r\n}\r\n");
    }
}