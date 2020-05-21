use clap::{Arg, App, ArgMatches};
use crate::generator::generators::read_file_into_string;
use crate::parser::{Json, parse_json, ParserError};

mod parser;
mod generator;
mod sender;

fn main() {
    let args = App::new("json-generator")
        .version("0.1.0")
        .author("Boris Zhguchev <zhguchev@gmail.com>")
        .about("generate json documents based on example including predefined functions")
        .arg(Arg::with_name("json-file")
            .short('f')
            .long("json-file")
            .takes_value(true)
            .about("example to generate"))
        .arg(Arg::with_name("json-body")
            .short('b')
            .long("json-body")
            .takes_value(true)
            .about("json body to generate"))
        .arg(Arg::with_name("repeater")
            .short('r')
            .long("repeat")
            .takes_value(true)
            .about("how many values needs to generate"))
        .arg(Arg::with_name("to-curl")
            .long("to-curl")
            .takes_value(true)
            .about("send the request through curl using this param and adding json body "))
        .arg(Arg::with_name("to-folder")
            .long("to-folder")
            .takes_value(true)
            .about("save jsons as separated files"))
        .arg(Arg::with_name("to-file")
            .long("to-file")
            .takes_value(true)
            .about("save jsons to file"))
        .arg(Arg::with_name("pretty-js")
            .long("pretty")
            .about("formatting"))
        .get_matches();


    let json = parse_text(find_json_text(&args).as_str());
    let rep: usize = args.value_of('r').unwrap_or("1").parse()
        .expect("repeat operator should be a number more then 0");


}


fn find_json_text(args: &ArgMatches) -> String {
    match (args.value_of('b'), args.value_of('f')) {
        (Some(body), _) => String::from(body),
        (None, Some(file)) => read_file_into_string(file)
            .expect("the input file or body containing json should be provided!"),
        (None, None) => panic!("the input file or body containing json should be provided!")
    }
}

fn parse_text(txt: &str) -> Json {
    match parse_json(txt) {
        Ok(json) => json,
        Err(e) => panic!("error while parsing json : {:?}", e),
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn find_json_text() {}
}