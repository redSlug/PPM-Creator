use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
use std::string::String;
extern crate regex;
use regex::Regex;
use std::fs::File;
use std::path::Path;

const MAX_SIZE_IN_BYTES: usize = 65536;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}


fn get_rgb(hex_color: &str) -> Vec<u8>{
    let r;
    let g;
    let b ;
    if hex_color.chars().count() == 3 {
        // 1 byte colors
        r = u8::from_str_radix(&format!("{}{}", &hex_color[0..1], &hex_color[0..1]), 16);
        g = u8::from_str_radix(&format!("{}{}", &hex_color[1..2], &hex_color[1..2]), 16);
        b = u8::from_str_radix(&format!("{}{}", &hex_color[2..3], &hex_color[2..3]), 16);
    } else {
        // 2 byte colors
        r = u8::from_str_radix(&hex_color[0..2], 16);
        g = u8::from_str_radix(&hex_color[2..4], 16);
        b = u8::from_str_radix(&hex_color[4..6], 16);
    }
    // Need it to be a vector so get append to work
    let mut ret = Vec::new();
    ret.push(r.unwrap());
    ret.push(g.unwrap());
    ret.push(b.unwrap());
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

    let mut buffer = [0; MAX_SIZE_IN_BYTES];
    stream.read(&mut buffer).unwrap();
    let payload = String::from_utf8_lossy(&buffer[..]); // from utf8 converts it to String instead of COW
    let response;

    let mut f = File::open("src/index.html").expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    if payload.contains("POST"){
        println!("post request!");
        println!("payload={}", &payload); // data in paload missing
        println!("before regex");
        let data_re = Regex::new(r"cols=(?P<cols>[0-9]+)\#(?P<data>([0-9a-zA-Z\#])+)").unwrap();
        let cap = data_re.captures(&payload);

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
    } else if payload.contains("GET"){
        response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}\r\n", contents);

    } else {
        response = format!("HTTP/1.1 {}\r\n\r\n\n{}\n", "400 BAD", "nope!");
    }

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}