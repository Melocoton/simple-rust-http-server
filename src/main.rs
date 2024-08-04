use std::{fs, io};
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::process::exit;

fn main() {
    let listener_result = start_socket(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9001));
    let listener: TcpListener;
    match listener_result {
        Ok(v) => {
            listener = v;
            let p = &listener;
            println!("Started socket: {p:?}");
        },
        Err(e) => {
            println!("Error: {e:?}");
            exit(1);
        }
    }
    
    let paths = fs::read_dir("./").unwrap();
    for path in paths {
        println!("Files: {:?}", path.unwrap().path().display());
    }

    loop {
        let mut connection = listener.accept().unwrap();
        println!("Connection estabilshed");
        
        let buf_reader = BufReader::new(&connection.0);
        let request: Vec<String> = buf_reader.lines().map(|result| result.unwrap()).take_while(|line| !line.is_empty()).collect();
        
        println!("Data {request:?}");
        println!("Data size {:?} bytes", data_size(&request));

        let file_data: Vec<u8>;
        if request.len() > 0 {
            let path: String = parse_request(&request[0]);
            let final_path: String = String::from(".")+ &*path;
            let file_read: io::Result<Vec<u8>>;
            
            if path == "/" {
                file_read = fs::read("./index.html");
            } else {
                file_read = fs::read(final_path);
            }
            
            match file_read {
                Ok(data) => file_data = data,
                Err(e) => {
                    match e.kind() { 
                        ErrorKind::NotFound => {
                            file_data = vec![];
                        },
                        _ => {
                            println!("Error reading file {e:?}");
                            let response = "HTTP/1.1 500 Internal Server Error\r\nServer: custom\r\n\r\n<h1 style=\"color: red\">Could not read file<h1>\r\n".as_bytes();
                            connection.0.write_all(response).unwrap();
                            continue;
                        }
                    }
                }
            }
        } else {
            file_data = vec![];
        }
        
        let response: Vec<u8>;
        if request.len() > 0 && file_data.len() > 0 {
            let path: String = parse_request(&request[0]);
            println!("Requested path is: {path}");
            let header = "HTTP/1.1 200 OK\r\nServer: custom\r\n\r\n".as_bytes().to_vec();
            response = [header, file_data].concat();
        } else {
            response = "HTTP/1.1 404 Not Found\r\nServer: custom\r\n\r\n<h1 style=\"color: red\">Not found<h1>\r\n".as_bytes().to_vec();
        }
        
        connection.0.write_all(&response).unwrap();
        println!("Response sent, size: {:?} bytes", response.len())
    }
    
}

fn parse_request(request: &String) -> String {
    let parts = request.split(" ").collect::<Vec<&str>>();
    String::from(parts[1])
}

fn data_size(data: &Vec<String>) -> usize {
    let mut size: usize = 0;
    for line in data {
        size += line.len();
    }
    size
}

fn start_socket(socket_addr: SocketAddrV4) -> Result<TcpListener, Error> {
    let addr = socket_addr;
    let listener = TcpListener::bind(addr);
    return listener;
}
