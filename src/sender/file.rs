use std::fs::{metadata, Metadata, File, OpenOptions, create_dir_all};
use std::io::{Error, Write};
use std::path::{Path, PathBuf};
use crate::sender::Sender;
use std::time::{SystemTime, UNIX_EPOCH};


pub struct FolderSender {
    path: String,
    idx: usize,
}

impl FolderSender {
    pub fn new(path: String) -> Self {
        match metadata(path.clone()) {
            Ok(m) => {
                if !m.is_dir() {
                    panic!("the output path to file should point out to a folder.");
                }
            }
            Err(e) =>
                if !Path::new(path.as_str()).exists() {
                    match create_dir_all(path.as_str()) {
                        Ok(_) => (),
                        Err(e) => panic!(
                            format!("error occuring while creating or open the file:{}",
                                    e.to_string())),
                    }
                } else { panic!(format!("the error occurs with output file: {}", e.to_string())) }
        }

        FolderSender { path, idx: 0 }
    }
}

impl Sender for FolderSender {
    fn send(&mut self, json: String) -> Result<String, String> {
        let mut pb = PathBuf::new();
        pb.push(self.path.as_str());
        pb.push(format!("json_{}.json", self.idx).as_str());

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(pb.as_path())
            .expect("problem with a file");


        if let Err(e) = file.write_all(json.into_bytes().as_slice()) {
            Err(format!("error while appending to a file: {}", e.to_string()))
        } else {
            let res = format!("the item {} has been sent to folder: {}", self.idx, self.path);
            self.idx += 1;
            Ok(res)
        }
    }
}


fn current_ts() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("error with time")
        .as_millis()
}

pub struct FileSender {
    path: String
}


impl FileSender {
    pub fn new(path: String) -> Self {
        match metadata(path.clone()) {
            Ok(m) => {
                if m.is_dir() {
                    panic!("the output path to file should point out to file not a folder.");
                }
            }
            Err(_) => match create_file(path.as_str()) {
                Ok(_) => (),
                Err(str) => panic!(str),
            }
        }

        FileSender { path }
    }
}

fn create_file(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        match File::create(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(
                format!("error occuring while creating or open the file:{}",
                        e.to_string())),
        }
    } else { Ok(()) }
}

impl Sender for FileSender {
    fn send(&mut self, json: String) -> Result<String, String> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(self.path.as_str())
            .expect("the file to append should be there");


        if let Err(e) = file.write_all(json.into_bytes().as_slice()) {
            Err(format!("error while appendiong a file: {}", e.to_string()))
        } else {
            let res = format!("the item has been sent to file: {}", self.path);
            Ok(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sender::file::{FileSender, FolderSender};
    use crate::sender::Sender;

    #[test]
    fn file_sender_test() {
        match FileSender::new(r#"C:\projects\json-generator\jsons\test.txt"#.to_string())
            .send("test".to_string()) {
            Ok(_) => (),
            Err(_) => panic!("!"),
        }
    }

    #[test]
    fn folder_sender_test() {
        match FolderSender::new(r#"C:\projects\json-generator\jsons\t"#.to_string())
            .send("test!!".to_string()) {
            Ok(_) => (),
            Err(_) => panic!("!"),
        }
    }
}