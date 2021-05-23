use serde_json::Value;
use crate::generator::GeneratorFunc;
use crate::json_template::JsonTemplate;
use crate::sender::Sender;

#[macro_use]
pub extern crate log;
pub extern crate simplelog;

#[macro_use]
mod r#macro;

mod parser;
pub mod generator;
pub mod sender;
pub mod json_template;
mod error;

pub fn generate(json: &mut JsonTemplate, rep: usize, pretty: bool, outputs: &mut Vec<Box<dyn Sender>>) -> Vec<Value> {
    debug!("generate the {} repetitions. ", rep);
    let mut res = vec![];
    for _ in 0..rep {
        let value = json.next_value();
        res.push(value.clone());
        for mut v in outputs.iter_mut() {
            match v.send_with_pretty(&value, pretty) {
                Ok(res) => info!("sending json, success : {}", res),
                Err(e) => error!("sending json, error : {}", e)
            }
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::generate;
    use crate::json_template::JsonTemplate;
    use serde_json::Value;
    use chrono::Utc;

    #[test]
    fn base_test() {
        let jt_body = r#"{"|id": "uuid()"}"#;
        let mut js_template = JsonTemplate::from_str(jt_body, "|").unwrap();
        if_let!(
            generate(&mut js_template,1,true,&mut vec![]).get(0)
                => Some(Value::Object(map))
                => assert!(map.get("id").and_then(|e|e.as_str()).unwrap().len() == 36 ));
    }

    #[test]
    fn full_test() {
        let jt_body = r#"
{
  "description": "the example how to create a json template to generate new jsons",
  "note": "the prefix | in a field name signals that the field carries a function to generate data.",
  "record": {
    "|type": "str_from_list(business,technical,analytical)",
    "technical":{
      "|id": "uuid()",
      "|index": "seq()",
      "|update_tm": "dt()",
      "|created_tm": "dt(%Y-%m-%d %H:%M:%S)"
    },
    "|is_active": "bool()",
    "|name": "str(10,customer)",
    "|email": "str(5,,@gmail.com)",
    "|code": "str(5,,'(code)')",
    "|dsc": "str(20)",
    "geo": {
      "|country": "str_from_file(jsons/countries,,)",
      "|city": "str_from_file(jsons/cities,\n)",
      "|street": "str(10,,-street)",
      "|house": "int(1,1000)"
    },
    "|id_parent": "int_from_list(1,2,3,4,5,6,7)",
    "|related_records": "int(1,1000) -> array(5)"

  }
}
            "#;
        let mut js_template = JsonTemplate::from_str(jt_body, "|").unwrap();
        let json = generate(&mut js_template,1,true,&mut vec![]);
        println!("{}",json.get(0).unwrap().to_string());
        if_let!(
            json.get(0)
                => Some(Value::Object(values)) => {
                    if_let!(values.get("description") => Some(Value::String(v))
                        => assert_eq!(v,"the example how to create a json template to generate new jsons"));
                    if_let!(values.get("record") => Some(Value::Object(rec_values)) => {
                            if_let!(rec_values.get("type") => Some(Value::String(v))
                                => assert!("businesstechnicalanalytical".contains(v)));
                             if_let!(rec_values.get("is_active") => Some(Value::Bool(_)) => ());
                             if_let!(rec_values.get("name") => Some(Value::String(v)) => {
                                assert_eq!(v.len(),18);
                                assert!(v.starts_with("customer"))
                             });
                             if_let!(rec_values.get("email") => Some(Value::String(v)) => {
                                assert_eq!(v.len(),15);
                                assert!(v.ends_with("@gmail.com"))
                             });
                             if_let!(rec_values.get("code") => Some(Value::String(v)) => {
                                assert_eq!(v.len(),11);
                                assert!(v.ends_with("(code)"))
                             });
                              if_let!(rec_values.get("dsc") => Some(Value::String(v)) => assert_eq!(v.len(),20));
                              if_let!(rec_values.get("id_parent") => Some(Value::Number(n)) => {
                                assert!(vec![1,2,3,4,5,6,7].contains(&n.as_i64().unwrap()))
                              });
                              if_let!(rec_values.get("related_records") => Some(Value::Array(elems)) => {
                                assert_eq!(elems.len(),5);
                                for el in elems.iter(){
                                    let el = el.as_i64().unwrap();
                                    assert!(el > 0 && el < 1000)
                                }
                              });
                              if_let!(rec_values.get("geo") => Some(Value::Object(elems)) => {
                                if_let!(elems.get("country") => Some(Value::String(v))
                                    => assert!("USAEnglandIrelandGermanyRussiaJapanAustralia".contains(v)));
                                if_let!(elems.get("city") => Some(Value::String(v))
                                    => assert!("BerlinPragueMoscowLondonHelsinkiRomeBarcelonaViennaAmsterdamDublin".contains(v)));
                                if_let!(elems.get("house") => Some(n) => {
                                    let el = n.as_i64().unwrap();
                                    assert!(el > 0 && el < 1000)
                                });
                              });
                              if_let!(rec_values.get("technical") => Some(Value::Object(elems)) => {
                                if_let!(elems.get("id") => Some(Value::String(v)) => assert_eq!(v.len(), 36));
                                if_let!(elems.get("index") => Some(n) => {
                                     let el = n.as_i64().unwrap();
                                     assert_eq!(el,1)
                                });
                                if_let!(elems.get("update_tm") => Some(Value::String(v)) => {
                                     let time = Utc::now();
                                     assert_eq!(v,&time.format("%Y-%m-%d %H:%M:%S").to_string())
                                });
                              });

                        }
                    )

                }

                );
    }
}