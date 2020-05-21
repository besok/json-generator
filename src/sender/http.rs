use std::process::{Command, Output};
use std::io;
use crate::sender::{Sender, PrettyJson};
use crate::parser::Json;
use std::io::Error;


pub struct CurlSender {
    cmd: String
}

impl Sender for CurlSender {
    fn send(&self, json: String) -> Result<String, String> {
        match curl(self.cmd.as_str(), json.as_str()) {
            Ok(o) => Ok(format!("{:?}", o)),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn curl(cmd: &str, json: &str) -> io::Result<Output> {
    let mut args: Vec<&str> = cmd.split_whitespace().collect();

    args.push("-d");
    args.push(json);

    Command::new("curl")
        .args(args)
        .output()
}

#[cfg(test)]
mod tests {
    use crate::sender::http::curl;
    use std::process::Output;
    use std::io::Error;

    #[test]
    fn simple_test() {
        let res = curl(
            r#"-X POST 127.0.0.1:7878 -H Content-Type: application/json"#,
            r#"{"key1":"value1", "key2":"value2"}"#).expect("no error");
        match res {
            Output { status, stderr, stdout } => {
                println!("status : {}", status);
                println!("{}", std::str::from_utf8(&stdout).expect("no error"));
                println!("{}", std::str::from_utf8(&stderr).expect("no error"));
            }
        }
    }
}