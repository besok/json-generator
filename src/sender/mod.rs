use crate::parser::Json;

mod http;

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
            obj @ Json::Object(_) =>
                sqr(crly(smcln(obj.to_string()))),
            prm @ _ => prm.to_string(),
        }
    }
}

fn smcln(str: String) -> String {
    let mut res = str;
    res = res.replace(",", format!(",{}", S).as_str());
    res
}

fn crly(str: String) -> String {
    let mut res = str;
    res = res.replace("{", format!("{{{}", S).as_str());
    res = res.replace("}", format!("{}}}", S).as_str());
    res
}

fn sqr(str: String) -> String {
    let mut res = str;
    res = res.replace("[", format!("[{}", S).as_str());
    res = res.replace("]", format!("{}]", S).as_str());
    res
}

#[cfg(test)]
mod tests {
    use crate::parser::{Json, Field, parse_json, ParserError};
    use crate::sender::{PrettyJson, smcln, crly};

    //todo  tests will occur to fall in linux.

    #[test]
    fn pretty_to_string_test() {
        let js = r#"{"person":{"id":1,"name":"Eli\"za\"beth","surname":"E","age":10,"children":[3,6],"address":{"street":"Grip","house":10,"city":"Berlin"}}} "#;
        match parse_json(js) {
            Ok(json) => {
                let pretty_js = PrettyJson::new(json).to_string();
                assert_eq!("{\r\n\"person\":{\r\n\"id\":1,\r\n\"name\":\"Eli\\\"za\\\"beth\",\r\n\"surname\":\"E\",\r\n\"age\":10,\r\n\"children\":[\r\n3,\r\n6\r\n],\r\n\"address\":{\r\n\"street\":\"Grip\",\r\n\"house\":10,\r\n\"city\":\"Berlin\"\r\n}\r\n}\r\n}",
                           pretty_js)
            }
            Err(e) => panic!("{}", e),
        };
    }

    #[test]
    fn add_nl_after_semicolon_test() {
        let str = "a,b,c".to_string();
        let res_str = smcln(str);
        assert_eq!(res_str, "a,\r\nb,\r\nc");
    }

    #[test]
    fn add_nl_after_bracket_test() {
        let str = "{a{b},c}".to_string();
        let res_str = crly(str);
        assert_eq!(res_str, "{\r\na{\r\nb\r\n}\r\n,c\r\n}\r\n");
    }
}