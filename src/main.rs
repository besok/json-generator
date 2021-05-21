use clap::{App, Arg, ArgMatches};
use simplelog::*;

use crate::generator::generators::read_file_into_string;
use crate::sender::{ConsoleSender, Sender};
use crate::sender::file::{FileSender, FolderSender};
use crate::sender::http::CurlSender;
use crate::json_template::JsonTemplate;
use crate::generator::GeneratorFunc;

#[macro_use]
extern crate log;
extern crate simplelog;

#[macro_use]
mod r#macro;

mod parser;
mod generator;
mod sender;
mod json_template;
mod error;


fn main() {
    let args = get_args();
    if args.is_present("logs") {
        SimpleLogger::init(LevelFilter::Debug, Config::default()).unwrap()
    }

    generate(
        &mut json(&args),
        r(&args),
        args.is_present("pretty-js"),
        &mut output(&args),
    )
}

fn get_args() -> ArgMatches {
    App::new("json-generator")
        .version("0.2")
        .author("Boris Zhguchev <zhguchev@gmail.com>")
        .about("The json generator with ability to generate dynamic fields.")
        .arg(
            Arg::with_name("json-file")
                .short('f')
                .long("json-file")
                .takes_value(true)
                .allow_hyphen_values(true)
                .conflicts_with("json-body")
                .about("the file containing the json template"))
        .arg(
            Arg::with_name("json-body")
                .short('b')
                .long("json-body")
                .takes_value(true)
                .allow_hyphen_values(true)
                .conflicts_with("json-file")
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
        .get_matches()
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

fn generate(json: &mut JsonTemplate, rep: usize, pretty: bool, outputs: &mut Vec<Box<dyn Sender>>) -> () {
    debug!("generate the {} repetitions. ", rep);
    for _ in 0..rep {
        for mut v in outputs.iter_mut() {
            match v.send_with_pretty(json.next_value(), pretty) {
                Ok(res) => info!("sending json, success : {}", res),
                Err(e) => error!("sending json, error : {}", e)
            }
        }
    }
}

fn json(args: &ArgMatches) -> JsonTemplate {
    debug!("try to parse the json template...");
    let txt = match (args.value_of("json-body"), args.value_of("json-file")) {
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

#[cfg(test)]
mod tests {
    #[test]
    fn find_json_text() {}
}
