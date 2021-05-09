use serde_json::Value;
use crate::generator::{Generator};
use std::iter::Map;
use crate::parser::structure::JsonGen::{Plain, Array, Object, Gen};
use crate::parser::generator::generator;

#[derive(Debug)]
enum JsonGen {
    Object(Vec<(String, JsonGen)>),
    Array(Vec<JsonGen>),
    Plain(Value),
    Gen(Generator),
}

impl ToString for JsonGen {
    fn to_string(&self) -> String {
        match self {
            Object(pairs) => {
                let mut res = "{".to_string();
                for (k, v) in pairs.iter() {
                    res.push_str(k.as_str());
                    res.push_str(":");
                    res.push_str(v.to_string().as_str());
                    res.push_str(",")
                }
                res.push_str("}");
                res
            }
            Array(elems) => {
                let mut res = "[".to_string();

                for e in elems.iter() {
                    res.push_str(e.to_string().as_str());
                    res.push_str(",")
                }
                res.push_str("]");
                res
            }
            Plain(v) => v.to_string(),
            Gen(g) => g.to_string(),
        }
    }
}

impl JsonGen {
    fn new(value: Value, indicator: &str) -> Result<Self, String> {
        match value {
            Value::Object(pairs) => {
                let mut res_pairs = vec![];
                for (k, v) in pairs.iter() {
                    if k.starts_with(indicator) {
                        match v {
                            Value::String(gen_str) => {
                                let internal_gen = generator(gen_str)
                                    .map(|e| e.1)
                                    .map_err(|e| e.to_string())?;
                                let new_k = String::from(k.strip_prefix(indicator).ok_or("unreachable")?);
                                res_pairs.push((new_k, Gen(internal_gen)))
                            }
                            _ => return Err(format!("Error for field '{}' : a generator function should be a string.", k))
                        }
                    } else {
                        res_pairs.push((k.clone(), JsonGen::new(v.clone(), indicator)?))
                    }
                }
                Ok(Object(res_pairs))
            }
            Value::Array(elems) => {
                let mut res_elems = vec![];
                for e in elems.iter() {
                    res_elems.push(JsonGen::new(e.clone(), indicator)?)
                }
                Ok(Array(res_elems))
            }
            plain => Ok(Plain((plain)))
        }
    }
    fn from_str(json: &str, indicator: &str) -> Result<Self, String> {
        let value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        JsonGen::new(value, indicator)
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::structure::JsonGen;
    use serde_json::{json, Value};

    #[test]
    fn simple_test() {
        let json = json!({
            "|field": "uuid()",
            "num" : 1
        });
        let res = JsonGen::new(json, "|");
        assert!(res.is_ok());
        println!("{}", res.unwrap().to_string());
    }
    #[test]
    fn simple_failed_test() {
        let json = json!({
            "|field": "uuidds()",
            "num" : 1
        });
        let res = JsonGen::new(json, "|");
        println!("{}", res.err().unwrap());
    }

    #[test]
    fn from_str_test() {
        let res = JsonGen::from_str(
            r#"{"|field": "uuid()","num" : 1}"#, "|",
        );
        assert!(res.is_ok());
        println!("{}", res.unwrap().to_string());
    }
}
