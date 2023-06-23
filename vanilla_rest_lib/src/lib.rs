
#![allow(dead_code)]

use std::{
    io::{prelude::*, BufReader, ErrorKind},
    net::TcpStream, sync::{Arc, Mutex},
};

pub const MAX_REQ_SIZE:usize = 256;

pub struct Connection<T> {
    stream: TcpStream,
    pub req: Request,
    pub state: Option<Arc<Mutex<T>>>,
    handlers: Vec<fn(&Self) -> Response>,
}

impl<T> Connection<T> {
    pub fn new(mut _stream: TcpStream) -> Connection<T> {
        let mut buf =[b'\0'; MAX_REQ_SIZE];
        BufReader::new(&mut _stream).read(&mut buf).unwrap();

        Connection {
            stream: _stream,
            req: Request::from_utf8(&buf).unwrap(),
            state: None,
            handlers: Vec::new(),
        }
    }

    pub fn mount_handlers(&mut self, handlers: Vec<fn(&Self) -> Response>) -> &mut Self {
        for handler in handlers {
            self.handlers.push(handler);
        }
        self
    }

    pub fn mount_state(&mut self, state: Arc<Mutex<T>>) -> &mut Self {
        self.state = Some(state);
        self
    }

    pub fn serve(&mut self) {
        let mut res = Response::empty();
        for handler in self.handlers.clone() {
            let _res = handler(&self);
            if !_res.status_line.is_empty() {
                res = _res;
                break;
            }
        }

        if res.status_line.is_empty() {
            res = Response {
                status_line: "HTTP/1.1 404 NOT FOUND".to_string(),
                headers: Vec::new(),
                body: Vec::new(),
            }
        }

        self.stream.write_all(res.as_bytes().as_slice()).unwrap();
    }
}


#[derive(Debug, Clone)]
pub struct Request {
    pub request_line: String,
    pub headers : Vec<(String, String)>,
    pub body: Vec<u8>
}

impl Request {
    pub fn from_utf8(bytes: &[u8]) -> Result<Request, Box<dyn std::error::Error>> {
        let mut req = Request { request_line: String::new(), headers: Vec::new(), body: Vec::new() };

        let mut beginning = 0;  
        let mut index = 0;
        while index < bytes.len()-1 {
            if bytes[index] == b'\r' && bytes[index +1] == b'\n' {
                let line = String::from_utf8(bytes[beginning..index].to_vec()).unwrap();

                if req.request_line.is_empty() && beginning == 0 && !line.is_empty() {
                    req.request_line = line;
                } else if !line.is_empty() {
                    let mut split = line.split(':').map(|s| s.trim().to_string());
                    req.headers.push((split.next().unwrap(), split.next().unwrap()));
                }
                beginning = index +2;
            }
            index += 1;
        }

        if req.request_line.is_empty() { return Err(Box::new(std::io::Error::from(ErrorKind::InvalidData))); }

        if let Some((_, _len)) = req.headers.iter().find(|(s, _)| s == "Content-Length") {
            let len = _len.parse::<usize>()?;
            if len != 0 {
                req.body  = bytes[beginning..(beginning+len)].to_vec();
            }
        }

        Ok(req)
    }

    pub fn empty() -> Self {
        Request { request_line: String::new(), headers: Vec::new(), body: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status_line: String,
    pub headers : Vec<(String, String)>,
    pub body: Vec<u8>
}

impl Response {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::from(self.status_line.as_bytes());
        buf.push(b'\r');
        buf.push(b'\n');
        for (key, value) in self.headers.as_slice() {
            buf.append(&mut key.as_bytes().to_vec());
            buf.push(b':');
            buf.push(b' ');
            buf.append(&mut value.as_bytes().to_vec());
            buf.push(b'\r');
            buf.push(b'\n');
        }
        buf.push(b'\r');
        buf.push(b'\n');
        buf.append(&mut self.body.clone());
        buf
    }

    pub fn empty() -> Self {
        Response { status_line: String::new(), headers: Vec::new(), body: Vec::new() }
    }
}
