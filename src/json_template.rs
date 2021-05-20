use serde_json::{Value, Map};
use crate::generator::{Generator, GeneratorFunc};
use crate::json_template::JsonTemplate::{Plain, Array, Object, Gen};
use crate::parser::generator::{atomic_generator, generator};
use crate::parser::GenError;

#[derive(Debug)]
pub enum JsonTemplate {
    Object(Vec<(String, JsonTemplate)>),
    Array(Vec<JsonTemplate>),
    Plain(Value),
    Gen(Generator),
}

impl ToString for JsonTemplate {
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

fn parse_generator(gen_str: &str) -> Result<Generator, GenError> {
    generator(gen_str)
}

impl JsonTemplate {
    pub fn new(value: Value, indicator: &str) -> Result<Self, GenError> {
        match value {
            Value::Object(pairs) => {
                let mut res_pairs = vec![];
                for (k, v) in pairs.iter() {
                    if k.starts_with(indicator) {
                        match v {
                            Value::String(gen_str) => {
                                res_pairs.push((
                                    k.strip_prefix(indicator).ok_or(GenError::new_with("unreachable".to_string()))?.to_string(),
                                    Gen(parse_generator(gen_str)?)
                                ))
                            }
                            _ => return Err(GenError::new_with(format!("Error for field '{}' : a generator function should be a string.", k)))
                        }
                    } else {
                        res_pairs.push((k.clone(), JsonTemplate::new(v.clone(), indicator)?))
                    }
                }
                Ok(Object(res_pairs))
            }
            Value::Array(elems) => {
                let mut res_elems = vec![];
                for e in elems.iter() {
                    res_elems.push(JsonTemplate::new(e.clone(), indicator)?)
                }
                Ok(Array(res_elems))
            }
            plain => Ok(Plain(plain))
        }
    }
    pub fn from_str(json: &str, indicator: &str) -> Result<Self, GenError> {
        let value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        JsonTemplate::new(value, indicator)
    }
}

impl GeneratorFunc for JsonTemplate {
    fn next_value(&mut self) -> Value {
        match self {
            Object(gen_pairs) => {
                let mut fields = serde_json::Map::new();
                for (k, t) in gen_pairs.into_iter() {
                    fields.insert(k.clone(), t.next_value());
                }
                Value::from(fields)
            }

            Array(elems) => Value::Array(elems.into_iter().map(|t| t.next_value()).collect()),
            Plain(v) => v.clone(),
            Gen(generator) => generator.next()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::json_template::JsonTemplate;
    use serde_json::{json, Value};

    #[test]
    fn simple_test() {
        let json = json!({
            "|field": "uuid()",
            "num" : 1
        });
        let res = JsonTemplate::new(json, "|");
        assert!(res.is_ok());
        println!("{}", res.unwrap().to_string());
    }

    #[test]
    fn simple_failed_test() {
        let json = json!({
            "|field": "uuidds()",
            "num" : 1
        });
        let res = JsonTemplate::new(json, "|");
        println!("{}", res.err().unwrap());
    }

    #[test]
    fn from_str_test() {
        let res = JsonTemplate::from_str(
            r#"{"|field": "uuid()","num" : 1}"#, "|",
        );
        assert!(res.is_ok());
        println!("{}", res.unwrap().to_string());
    }
}
