use clap::{App, Arg, ArgMatches};
use simplelog::*;
use serde_json::Value;
use json_generator::generate;
use json_generator::json_template::JsonTemplate;
use json_generator::sender::{Sender, ConsoleSender};
use json_generator::sender::file::{FileSender, FolderSender};
use json_generator::sender::http::CurlSender;
use json_generator::generator::generators::read_file_into_string;

#[macro_use]
pub extern crate log;

fn main() {
    let args = get_args();
    if args.is_present("logs") {
        SimpleLogger::init(LevelFilter::Debug, Config::default()).unwrap()
    }

    generate_from_args(&args);
}

fn create_args<'b>() -> App<'b> {
    App::new("json-generator")
        .version("0.2")
        .author("Boris Zhguchev <zhguchev@gmail.com>")
        .about("The json generator with ability to generate dynamic fields.")
        .arg(
            Arg::with_name("jt-file")
                .short('f')
                .long("file")
                .takes_value(true)
                .allow_hyphen_values(true)
                .conflicts_with("jt-body")
                .about("the file containing the json template"))
        .arg(
            Arg::with_name("jt-body")
                .short('b')
                .long("body")
                .takes_value(true)
                .allow_hyphen_values(true)
                .conflicts_with("jt-file")
                .about("the text representation containing the json template"))
        .arg(
            Arg::with_name("repeater")
                .short('r')
                .long("repeat")
                .takes_value(true)
                .about("how many repetition needs to perform"))
        .arg(
            Arg::with_name("indicator")
                .short('i')
                .long("indicator")
                .takes_value(true)
                .about("the prefix signalling the field contains a generator"))
        .arg(
            Arg::with_name("to-curl")
                .long("to-curl")
                .takes_value(true)
                .allow_hyphen_values(true)
                .about("to send the request through the curl utility using this param and adding json body (curl utility needs to be installed)"))
        .arg(
            Arg::with_name("to-folder")
                .long("to-folder")
                .takes_value(true)
                .allow_hyphen_values(true)
                .about("to save the generated jsons in the folder"))
        .arg(
            Arg::with_name("to-file")
                .long("to-file")
                .takes_value(true)
                .allow_hyphen_values(true)
                .about("save the generated jsons to the file"))
        .arg(
            Arg::with_name("to-console")
                .long("to-cmd")
                .about("to display the generated jsons in the console(by default if outputs array is empty)"))
        .arg(
            Arg::with_name("pretty-js")
                .long("pretty")
                .about("to format the generated json into the readable view"))
        .arg(
            Arg::with_name("logs")
                .long("logs")
                .about("to print extra logs"))
}

fn get_args() -> ArgMatches {
    create_args().get_matches()
}

fn r(args: &ArgMatches) -> usize {
    args
        .value_of("repeater")
        .unwrap_or("1")
        .parse()
        .expect("the repetition number should be a positive integer, greater than zero")
}

fn output(args: &ArgMatches) -> Vec<Box<dyn Sender>> {
    let mut outputs: Vec<Box<dyn Sender>> = vec![];
    if let Some(str) = args.value_of("to-file") {
        debug!("new output to the file: {}", str);
        outputs.push(Box::new(FileSender::new(str.to_string())))
    }
    if let Some(str) = args.value_of("to-folder") {
        debug!("new output to the folder: {}", str);
        outputs.push(Box::new(FolderSender::new(str.to_string())))
    }
    if let Some(str) = args.value_of("to-curl") {
        debug!("new output to the server: {}", str);
        outputs.push(Box::new(CurlSender::new(str.to_string())))
    }
    if args.is_present("to-console") {
        debug!("new output to the console");
        outputs.push(Box::new(ConsoleSender {}))
    }
    if outputs.is_empty() {
        debug!("set the output to the console");
        outputs.push(Box::new(ConsoleSender {}))
    }
    outputs
}

fn json_template(args: &ArgMatches) -> JsonTemplate {
    debug!("try to parse the json template...");
    let txt = match (args.value_of("jt-body"), args.value_of("jt-file")) {
        (Some(body), _) => {
            debug!("ready to obtain the json template from the body {}", body);
            String::from(body)
        }
        (None, Some(file)) => {
            debug!("ready to obtain the json template from the file {}", file);
            read_file_into_string(file)
                .expect("exception with the processing the file!")
        }
        (None, None) => panic!("the input file or body containing the json template should be provided!")
    };
    let indicator = args.value_of("indicator").unwrap_or("|");
    debug!("the json template with indicator[{}] {}", indicator, txt);
    match JsonTemplate::from_str(txt.as_str(), indicator) {
        Ok(t) => t,
        Err(e) => panic!("error while parsing json : {:?}", e),
    }
}

fn generate_from_args(args: &ArgMatches) -> Vec<Value> {
    generate(&mut json_template(&args), r(&args), args.is_present("pretty"), &mut output(&args))
}

#[cfg(test)]
mod tests {
    use crate::{create_args, generate_from_args};

    #[test]
    fn find_json_text() {
        let jt_body = r#"{"|id": "uuid()"}"#;
        let args =
            create_args()
                .get_matches_from(
                    vec![
                        "",
                        format!("--body={}", jt_body).as_str(),
                        "--pretty",
                    ]);
        let res = generate_from_args(&args);
        assert_eq!(res.len(), 1);
        assert_eq!(res.get(0)
                       .and_then(|v| v.as_object())
                       .unwrap()
                       .get("id")
                       .and_then(|e| e.as_str())
                       .unwrap().len(), 36);
    }
}
