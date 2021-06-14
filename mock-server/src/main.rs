extern crate simple_server;

use std::str;
use simple_server::Server;

fn main() {
    println!("the simple web-server capable to receive requests on localhost:7878");

    let host = "127.0.0.1";
    let port = "7878";

    let server = Server::new(|request, mut response| {
        let body = str::from_utf8(request.body()).unwrap();
        println!("{}",body);
        Ok(response.body(body.as_bytes().to_vec())?)
    });

    server.listen(host, port);
}