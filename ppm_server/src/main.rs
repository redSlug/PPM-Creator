use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
use std::string::String;
use std::str;
extern crate regex;
use regex::Regex;
use std::fs::File;
use std::io;
use std::path::Path;
use std::io::BufReader;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};


#[derive(PartialEq)]
enum Method {
    Get,
    Post,
}

impl Method {
    fn from_string(s: &str) -> io::Result<Method> {
        match s {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            &_ => Err(Error::new(ErrorKind::Other, format!("{} Unsupported Method", s))),
        }
    }
}

struct Request {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    fn from_stream(stream: &mut TcpStream) -> io::Result<Request> {
        let mut reader = BufReader::new(stream);
        let (method, path) = Request::read_status(&mut reader)?;
        let headers = Request::read_headers(&mut reader)?;
        let body = Request::read_body(&mut reader, Request::get_content_length(&headers))?;

        // returns
        Ok(Request {
            method,
            path,
            headers,
            body,
        })
    }

    fn get_content_length(headers: &HashMap<String, String>) -> usize{
        let content_length = "Content-Length";
        if !headers.contains_key(content_length) {
            return 0;
        }
        return headers.get(content_length).unwrap().parse::<usize>().unwrap();
    }

    fn read_line(reader: &mut BufRead) -> io::Result<String> {
        let mut buffer = Vec::new();
        reader.read_until(b'\n', &mut buffer).unwrap();
        let s = std::str::from_utf8(&buffer).unwrap();
        return Ok(s.to_string());
    }

    fn read_status(reader: &mut BufRead) -> io::Result<(Method, String)> {
        let data = Request::read_line(reader).unwrap();
        let v: Vec<&str> = data.split(' ').collect();
        return Ok((Method::from_string(v[0]).unwrap(), v[1].to_string()));
    }

    fn read_headers(reader: &mut BufRead) -> io::Result<HashMap<String, String>> {
        let mut headers = HashMap::new();
        loop {
            let l = Request::read_line(reader)?;
            let v: Vec<&str> = l.split(":").collect();
            if v.len() < 2 {
                // No more headers to parse
                return Ok(headers);
            }
            headers.insert(v[0].trim().to_string(), v[1].trim().to_string());
        }
    }

    fn read_body(reader: &mut BufRead, content_length: usize) -> io::Result<String> {
        if content_length == 0 {
            return Ok("no payload".to_string());
        }
        let mut buffer = vec![0u8; content_length];
        reader.read_exact(&mut buffer).unwrap();
        let res = str::from_utf8(&buffer).unwrap();
        return Ok(res.to_string());
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn get_rgb(hex_color: &str) -> Vec<u8>{
    let mut ret = Vec::new();
    ret.push(u8::from_str_radix(&hex_color[0..2], 16).unwrap());
    ret.push(u8::from_str_radix(&hex_color[2..4], 16).unwrap());
    ret.push(u8::from_str_radix(&hex_color[4..6], 16).unwrap());
    return ret;
}

fn get_ppm_file(input: &str, cols: i32) {
    let pixels = input.split("#");
    let path = Path::new("result.ppm");
    let mut file = match File::create(&path) {
        Err(_) => panic!("couldn't create file"),
        Ok(file) => file,
    };
    let rows = 16;
    let max_value = 255;
    let header = format!("P6\n{} {}\n{}\n", cols, rows, max_value);
    let mut count = rows * cols;
    let mut data = Vec::new();
    for pixel in pixels {
        if count == 0 {
            break;
        }
        data.append(&mut get_rgb(pixel));
        count -= 1;
    }
    match file.write_all(header.as_bytes()) {
        Err(_) => panic!("couldn't write header"),
        Ok(_) => println!("wrote header")
    };
    match file.write_all(&data) {
        Err(_) => panic!("couldn't write data"),
        Ok(_) => println!("wrote data")
    };
}

fn handle_connection(mut stream: TcpStream) {
    println!("connection");
    let r = Request::from_stream(&mut stream).unwrap();
    let mut response = format!("HTTP/1.1 {}\r\n\r\n\n{}\n", "400 BAD", "unimplemented");
    if r.method == Method::Get {
        println!("get request");
        let mut f = File::open("src/index.html").expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the file");
        response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}\r\n", contents);
    } else if r.method == Method::Post {
        println!("post request!");
        let body = r.body;
        println!("body={}", body);
        println!("before regex");
        let data_re = Regex::new(r"cols=(?P<cols>[0-9]+)\#(?P<data>([0-9a-zA-Z\#])+)").unwrap();
        let cap = data_re.captures(&body);
        match cap {
            None => {
                response = format!("HTTP/1.1 {}\r\n\r\n\n{}\n", "400 BAD", "payload error");
            },
            _ => {
                let caps = cap.unwrap();
                let cols = &caps["cols"];
                let cols_int = cols.parse().unwrap();
                let data = &caps["data"];
                get_ppm_file(data, cols_int);
                response = format!("HTTP/1.1 {}\r\n\r\n\n{}\n", "200 OK", data);
            }
        }
    }
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}