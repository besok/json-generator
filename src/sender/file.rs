use std::fs::{metadata,  File, OpenOptions, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::sender::{Sender, string_from};
use crate::error::GenError;
use serde_json::Value;

/// the struct which implements the Sender trait and allows
/// to save a generated json to folder
/// It includes the internal state to generate the index of the files
pub struct FolderSender {
    path: String,
    idx: usize,
}

impl FolderSender {
    pub fn new(path: String) -> Self {
        match metadata(path.clone()) {
            Ok(m) => {
                if !m.is_dir() {
                    panic!(format!("the output path {} to the file should point to a folder.", path));
                }
            }
            Err(e) =>
                if !Path::new(path.as_str()).exists() {
                    match create_dir_all(path.as_str()) {
                        Ok(_) => (),
                        Err(e) => panic!("error occurred while creating or open the file:{}", e.to_string()),
                    }
                } else { panic!("the error occurred with the output file: {}", e.to_string()) }
        }
        debug!("the folder sender with the path {} has been created successfully", path);
        FolderSender { path, idx: 0 }
    }
}

impl Sender for FolderSender {
    fn send(&mut self, json: &Value, pretty: bool) -> Result<String, GenError> {
        let mut pb = PathBuf::new();
        pb.push(self.path.as_str());
        pb.push(format!("json_{}.json", self.idx).as_str());

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(pb.as_path())
            .expect("problem with a file");


        let js =  string_from(json, pretty)?;
        if let Err(e) = file.write_all(js.into_bytes().as_slice()) {
            Err(GenError::new_with_in_sender(format!("error while appending to a file: {}", e.to_string()).as_str()))
        } else {
            let res = format!("the item {} has been saved in the folder: {}", self.idx, self.path);
            self.idx += 1;
            Ok(res)
        }
    }
}


/// the struct which implements the Sender trait and allows
/// to append a generated json to file
pub struct FileSender {
    path: String
}


impl FileSender {
    pub fn new(path: String) -> Self {
        match metadata(path.clone()) {
            Ok(m) => {
                if m.is_dir() {
                    panic!(format!("the output path {} should point to a file not to a folder.", path));
                }
            }
            Err(_) => match create_file(path.as_str()) {
                Ok(_) => (),
                Err(str) => panic!("the error: {} while creating a file {}",str.to_string(), path),
            }
        }

        debug!("the file sender with the path {} has been created successfully", path);
        FileSender { path }
    }
}

fn create_file(path: &str) -> Result<(), GenError> {
    if !Path::new(path).exists() {
        match File::create(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(GenError::new_with_in_sender(
                format!("error occurred while creating or open the file:{}", e.to_string())
                    .as_str())),
        }
    } else { Ok(()) }
}
impl Sender for FileSender {
    fn send(&mut self, json: &Value, pretty: bool) -> Result<String, GenError> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(self.path.as_str())
            .expect("the file to append should be there");

        let js = string_from(json, pretty)?;
        if let Err(e) = file.write_all(js.into_bytes().as_slice()) {
            Err(GenError::new_with_in_sender(format!("error occurred while appending to the file: {}", e.to_string())
                .as_str()))
        } else {
            Ok(format!("the item has been saved to the file: {}", self.path))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sender::file::{FileSender, FolderSender};
    use crate::sender::Sender;
    use serde_json::Value;
    use crate::error::GenError;
    use std::path::Path;
    use std::fs::{remove_file, remove_dir};

    fn rem_file(path: &str) -> Result<(), GenError> {
        if !Path::new(path).exists() {
            Err(GenError::new_with_in_sender(format!("the path {} does not exist", path).as_str()))
        } else {
            match remove_file(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(GenError::new_with_in_sender(
                    format!("error occurred while remove the file:{}", e.to_string()).as_str())),
            }
        }
    }
    fn rem_folder(path: &str) -> Result<(), GenError> {
        if !Path::new(path).exists() {
            Err(GenError::new_with_in_sender(format!("the folder {} does not exist", path).as_str()))
        } else {
            match remove_dir(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(GenError::new_with_in_sender(
                    format!("error occurred while remove the file:{}", e.to_string()).as_str())),
            }
        }
    }



    #[test]
    fn file_sender_test() {
        let file = "jsons/temp/file.json".to_string();

        match FileSender::new(file.clone())
            .send(&Value::Null, false) {
            Ok(_) => assert!(rem_file(file.as_str()).is_ok()),
            Err(e) => panic!("error : {}", e),
        }
    }

    #[test]
    fn folder_sender_test() {
        let file = "jsons/temp/temp".to_string();
        match FolderSender::new(file.clone()).send(&Value::Null, false) {
            Ok(_) => {
                assert!(rem_file(format!("{}/{}", file, "json_0.json").as_str()).is_ok());
                assert!(rem_folder(file.as_str()).is_ok());
            }
            Err(e) => panic!("error : {}", e),
        }
    }
}