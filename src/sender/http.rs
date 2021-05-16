use std::process::{Command, Output, Child};
use std::io;
use crate::sender::{Sender, S};
use std::io::Error;

/// the struct which implements the Sender trait and allows
/// to send a json to the server, using curl utility
pub struct CurlSender {
    pub cmd: String,
}

impl CurlSender {
    pub fn new(cmd: String) -> Self {
        CurlSender {
            cmd
        }
    }
}


fn out_to_str(out: &Output) -> String {
    format!("| status:{}{} | stdout:{}{}{} | stderr:{}{}",
            out.status, S,
            S,std::str::from_utf8(out.stdout.as_slice()).expect("no error"), S,
            S,std::str::from_utf8(out.stderr.as_slice()).expect("no error"),
    )
}

impl Sender for CurlSender {
    fn send(&mut self, json: String) -> Result<String, String> {
        match curl(self.cmd.as_str(), json.as_str()) {
            Ok(o) => Ok(
                format!("sending item to curl: {} - input: {}{} {}",
                        S, self.cmd, S, out_to_str(&o))
            ),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// the function using the curl from os
/// ```
///  let res = curl(
///                r#"-X POST 127.0.0.1:7878 -H Content-Type:application/json"#,
///                r#"{"key1":"value1", "key2":"value2"}"#).expect("no error");
/// ```
/// todo move to nonblocking output
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
    use std::process::{Output, Command, Child};
    use std::io::Error;

    #[test]
    fn simple_test() {
        for _ in 1..5 {
            let res = curl(
                r#"-X POST 127.0.0.1:7878 -H Content-Type:application/json"#,
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
}