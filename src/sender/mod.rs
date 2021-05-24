//! ### Senders
//! The module which is responsible to send generated jsons to different sources
//! It provides the trait Sender to work with the generated jsons.
//! There are following implementations exist:
//! * `ConsoleSender` sends the generated json to the console
//! * 'CurlSender' sends the generated json to the remote server according to the given command
//! * 'FolderSender' saves the generated jsons to the folder in the filesystem
//! * 'FileSender' saves the generated jsons to the file, appending it

use serde_json::{Value, to_string_pretty};
use crate::error::GenError;

pub mod http;
pub mod file;

#[cfg(windows)]
const S: &'static str = "\r\n";
#[cfg(not(windows))]
const S: &str = "\n";

/// the function that pursues to beautify the generated json, according to the flag `pretty`.
pub fn string_from(json: &Value, pretty: bool) -> Result<String, GenError> {
    if pretty { to_string_pretty(json) } else { Ok(json.to_string()) }
        .map_err(|e| GenError::new_with_in_sender(e.to_string().as_str()))
}

/// The trait denoting the behaviour which is needed to be performed with the generated jsons.
pub trait Sender {
    /// send to the pointed destination.
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

