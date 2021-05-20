//! The module which is responsible to send generated jsons to different sources
//! It provides the trait Sender to work with
use serde_json::{Value, to_string_pretty};

pub mod http;
pub mod file;

#[cfg(windows)]
const S: &'static str = "\r\n";
#[cfg(not(windows))]
const S: &'static str = "\n";

pub trait Sender {
    fn send(&mut self, json: String) -> Result<String, String>;
    fn send_pretty(&mut self, delegate: Value) -> Result<String, String> {
        self.send(to_string_pretty(&delegate).map_err(|e| e.to_string())?)
    }
}

pub struct ConsoleSender {}

impl Sender for ConsoleSender {
    fn send(&mut self, json: String) -> Result<String, String> {
        println!("{}", json);
        Ok("item has been sent to console".to_string())
    }
}