use std::{
    char,
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Get => "GET",
                Method::Post => "POST",
            }
        )
    }
}

#[derive(Debug)]
pub struct MethodParseError(String);
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

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Request {
    pub fn read(stream: &TcpStream) -> Self {
        let mut stream = stream.bytes().map(|x| x.unwrap_or(0));

        let line = read_line(&mut stream);
        let first: Vec<&str> = line.splitn(3, ' ').collect();
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
            method: first[0].parse().unwrap(),
            endpoint: first[1].trim().to_owned(),
            headers,
            body,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    status: usize,
    headers: HashMap<String, String>,
    body: String,
}

impl Response {
    pub fn new<S: Into<String>>(status: usize, body: S) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: body.into(),
        }
    }
    pub fn with_header<S0: Into<String>, S1: Into<String>>(self, key: S0, val: S1) -> Self {
        let mut headers = self.headers;
        headers.insert(key.into(), val.into());
        Self {
            status: self.status,
            headers,
            body: self.body,
        }
    }
    pub fn write(&self, s: &mut TcpStream) -> Result<(), std::io::Error> {
        s.write_all(self.to_string().as_bytes())?;
        s.flush()
    }
}
impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "HTTP/1.1 {}", self.status)?;
        self.headers
            .iter()
            .try_fold((), |_, (key, value)| writeln!(f, "{key}: {value}"))?;
        write!(f, "\n{}", self.body)
    }
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
