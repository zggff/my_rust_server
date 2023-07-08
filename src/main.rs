use std::{
    char,
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::Read,
    net::{TcpListener, TcpStream},
    str::FromStr,
};

#[derive(Debug)]
enum Method {
    Get,
    Post,
}

#[derive(Debug)]
struct MethodParseError(String);
impl Display for MethodParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot parse {} as POST method", self.0)
    }
}
impl Error for MethodParseError {}
impl FromStr for Method {
    type Err = MethodParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            a => Err(MethodParseError(a.to_owned())),
        }
    }
}

#[derive(Debug)]
struct Request {
    method: Method,
    endpoint: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Request {
    pub fn from_stream(stream: &TcpStream) -> Self {
        let mut stream = stream.bytes().map(|x| x.unwrap_or(0));

        let line = read_line(&mut stream);
        let (method, endpoint) = line.split_once(' ').unwrap();
        let mut headers = HashMap::new();

        loop {
            let line = read_line(&mut stream);
            if line.trim().is_empty() {
                break;
            }
            let (name, value) = line.split_once(':').unwrap();
            headers.insert(name.trim().to_owned(), value.trim().to_owned());
        }

        let body = headers
            .get("Content-Length")
            .map(|x| x.parse::<usize>().unwrap_or(0))
            .and_then(|l| {
                if l == 0 {
                    return None;
                }
                Some(String::from_utf8_lossy(&stream.take(l).collect::<Vec<u8>>()).to_string())
            });

        Request {
            method: method.parse().unwrap(),
            endpoint: endpoint.trim().to_owned(),
            headers,
            body,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;
    for stream in listener.incoming() {
        let request = Request::from_stream(&stream?);
        dbg!(request);
    }
    Ok(())
}


fn read_line<I: Iterator<Item = u8>>(i: &mut I) -> String {
    let mut s = String::new();
    for c in i {
        if c == b'\n' {
            return s;
        }
        s.push(char::from_u32(c as u32).unwrap());
    }
    s
}
