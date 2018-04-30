use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*; // to read from TCP stream
use std::string::String;
extern crate regex;
use regex::Regex;
use std::i64;
use std::num::ParseIntError;

const MAX_SIZE_IN_BYTES: usize = 65536;

// curl -H "Expect:" -H "Content-Type: application/json" -X POST -d "@big_to_send" 127.0.0.1:8080
// add expect to CURL to avoid the continue BS


fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
        handle_connection(stream);
    }
}


fn get_rgb(hex_color: &str) -> (Result<i64, ParseIntError>, Result<i64, ParseIntError>, Result<i64, ParseIntError>){
    let r;
    let g;
    let b ;
    if hex_color.chars().count() == 3 {
        r = i64::from_str_radix(&format!("{}{}", &hex_color[0..1], &hex_color[0..1]), 16);
        g = i64::from_str_radix(&format!("{}{}", &hex_color[1..2], &hex_color[1..2]), 16);
        b = i64::from_str_radix(&format!("{}{}", &hex_color[2..3], &hex_color[2..3]), 16);
    } else {
        r = i64::from_str_radix(&hex_color[0..2], 16);
        g = i64::from_str_radix(&hex_color[2..4], 16);
        b = i64::from_str_radix(&hex_color[4..6], 16);
    }
    return (r, g, b); // maybe no return statement needed?

}

fn get_ppm_file(input: &str) {
    let pixels = input.split(",");

    let mut max = 0;

    let mut counter = 0;
    for p in pixels {
        println!("{}", p);

        let t = get_rgb(&p);
        println!("{:?}", t.0);
        println!("{:?}", t.1);
        println!("{:?}", t.2);

        if counter == 12 {
            break;
        }
        counter = counter + 1;
    }

}

fn send_response(mut stream: TcpStream, code: &str, data: &str) {
    let response = format!("HTTP/1.1 {}\r\n\r\n\n{}\n", code, data);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; MAX_SIZE_IN_BYTES]; // making the buffer 512 bytes in size
    stream.read(&mut buffer).unwrap();
    let payload = String::from_utf8_lossy(&buffer[..]); // from utf8 converts it to String instead of COW


    println!("payload: {}", payload);
    let re = Regex::new(r"\n\*\*cols=(?P<cols>[0-9]+),(?P<data>([0-9a-zA-Z,])+)\*\*").unwrap();
    let cap = re.captures(&payload);

    match cap {
        None => {
            send_response(stream, "400 BAD", "payload error");
        },
        _ => {
            let caps = cap.unwrap();
            println!("cols={}", &caps["cols"]);
            let data = &caps["data"];
            get_ppm_file(data);
            send_response(stream, "200 OK", data);
        }
    }

}