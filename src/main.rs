
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, prelude::*}; // импортировать все из пакета prelude


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    for stream in listener.incoming(){
        let stream = stream.unwrap();
        con_handle(&stream);
    }
}

#[allow(unused_variables)]
fn con_handle(mut stream: &TcpStream){
    let buf_reader = BufReader::new(stream);
    let requ = buf_reader.lines().next().unwrap().unwrap();

    let (status, filename) = if requ == "GET / HTTP/1.1"{
        ("HTTP/1.1 200 OK", "darkprince.html")
    }else{
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let content = fs::read_to_string(&filename).unwrap();
    let len = content.len();
    let response = format!("{status}\r\nContent-Length: {len}\r\n\r\n{content}");

    stream.write_all(response.as_bytes()).unwrap();
}
