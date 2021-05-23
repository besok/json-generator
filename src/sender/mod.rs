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

pub fn string_from(json: &Value, pretty: bool) -> Result<String, GenError> {
    if pretty { to_string_pretty(json) } else { Ok(json.to_string()) }
        .map_err(|e| GenError::new_with_in_sender(e.to_string().as_str()))
}

pub trait Sender {
    fn send(&mut self, json: &Value, pretty: bool) -> Result<String, GenError>;
}

pub struct ConsoleSender {}

impl Sender for ConsoleSender {
    fn send(&mut self, json: &Value, pretty: bool) -> Result<String, GenError> {
        debug!("send to the console");
        println!("{}",  string_from(json, pretty)?);
        Ok("the item has been sent to the console".to_string())
    }
}

