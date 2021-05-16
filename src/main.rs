//! # json generator
//! The console utility to generate JSON items according to the provided example composing JSON body
//! and a set of functions defining the logic to generate new items.
//! The utility allows delivering the generated JSON to different sources such as HTTP server, folder or file
//! ## Example
//! ```commandline
//! json-generator.exe  -f "file path" -r 10  --pretty --print --to-folder folder--to-curl '-X POST ip'
//! ```
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


fn main() {
    let args = get_args();
    if args.is_present("print") {
        SimpleLogger::init(LevelFilter::Debug, Config::default()).unwrap()
    }

    generate(&mut json(&args), r(&args), args.is_present("pretty-js"), &mut output(&args))
}

fn get_args() -> ArgMatches {
    App::new("json-generator")
        .version("0.1.0")
        .author("Boris Zhguchev <zhguchev@gmail.com>")
        .about("generate json documents based on example including predefined functions")
        .arg(Arg::with_name("json-file")
            .short('f')
            .long("json-file")
            .takes_value(true)
            .allow_hyphen_values(true)
            .conflicts_with("json-body")
            .about("example to generate"))
        .arg(Arg::with_name("json-body")
            .short('b')
            .long("json-body")
            .takes_value(true)
            .allow_hyphen_values(true)
            .conflicts_with("json-file")
            .about("json body to generate"))
        .arg(Arg::with_name("repeater")
            .short('r')
            .long("repeat")
            .takes_value(true)
            .about("how many values needs to generate"))
        .arg(Arg::with_name("to-curl")
            .long("to-curl")
            .takes_value(true)
            .allow_hyphen_values(true)
            .about("send the request through curl using this param and adding json body "))
        .arg(Arg::with_name("to-folder")
            .long("to-folder")
            .takes_value(true)
            .allow_hyphen_values(true)
            .about("save jsons as separated files"))
        .arg(Arg::with_name("to-file")
            .long("to-file")
            .takes_value(true)
            .allow_hyphen_values(true)
            .about("save jsons to file"))
        .arg(Arg::with_name("to-console")
            .long("to-cmd")
            .about("show json in console(by default if outputs array is empty)"))
        .arg(Arg::with_name("pretty-js")
            .long("pretty")
            .about("formatting"))
        .arg(Arg::with_name("print")
            .short('p')
            .long("print")
            .about("print logs"))
        .get_matches()
}

fn r(args: &ArgMatches) -> usize {
    args.value_of("repeater").unwrap_or("1").parse()
        .expect("repeat operator should be a number more then 0")
}

fn output(args: &ArgMatches) -> Vec<Box<dyn Sender>> {
    let mut outputs: Vec<Box<dyn Sender>> = vec![];
    if let Some(str) = args.value_of("to-file") {
        info!("set the output to file: {}", str);
        outputs.push(Box::new(FileSender::new(str.to_string())))
    }
    if let Some(str) = args.value_of("to-folder") {
        info!("set the output to folder: {}", str);
        outputs.push(Box::new(FolderSender::new(str.to_string())))
    }
    if let Some(str) = args.value_of("to-curl") {
        info!("set the output to server: {}", str);
        outputs.push(Box::new(CurlSender::new(str.to_string())))
    }
    if args.is_present("to-console") {
        info!("set the output to console");
        outputs.push(Box::new(ConsoleSender {}))
    }
    if outputs.is_empty() {
        info!("set the output to console");
        outputs.push(Box::new(ConsoleSender {}))
    }
    outputs
}

fn generate(json: &mut JsonTemplate, rep: usize, pretty: bool, outputs: &mut Vec<Box<dyn Sender>>) -> () {
    for _ in 0..rep {
        for mut v in outputs.iter_mut() {
            let res = if pretty {
                v.send_pretty(json.next_value())
            } else {
                v.send(json.next_value().to_string())
            };
            match res {
                Ok(res) => info!("sending json, success : {}", res),
                Err(e) => error!("sending json, error : {}", e)
            }
        }
    }
}

fn json(args: &ArgMatches) -> JsonTemplate {
    info!("parsing json:");
    let txt = match (args.value_of("json-body"), args.value_of("json-file")) {
        (Some(body), _) => String::from(body),
        (None, Some(file)) => read_file_into_string(file)
            .expect("the input file or body containing he json template should be provided!"),
        (None, None) => panic!("the input file or body containing the json template should be provided!")
    };
    info!("got the json template {}", txt);
    match JsonTemplate::from_str(txt.as_str(), "|") {
        Ok(t) => t,
        Err(e) => panic!("error while parsing json : {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn find_json_text() {}
}
