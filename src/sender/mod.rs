//! The module which is responsible to send generated jsons to different sources
//! It provides the trait Sender to work with
use serde_json::{Value, to_string_pretty};
use crate::error::GenError;

pub mod http;
pub mod file;

#[cfg(windows)]
const S: &'static str = "\r\n";
#[cfg(not(windows))]
const S: &'static str = "\n";

pub trait Sender {
    fn send(&mut self, json: String) -> Result<String, GenError>;
    fn send_with_pretty(&mut self, delegate: &Value, pretty: bool) -> Result<String, GenError> {
        self.send(if pretty { to_string_pretty(delegate)? } else { delegate.to_string() })
    }
}

pub struct ConsoleSender {}

impl Sender for ConsoleSender {
    fn send(&mut self, json: String) -> Result<String, GenError> {
        debug!("to send to the console");
        println!("{}", json);
        Ok("the item has been sent to the console".to_string())
    }
}

