use failure::format_err;
use failure::Error;
use std::io::{BufRead, BufReader, Lines};
use std::str::FromStr;
use std::net::TcpStream;
use std::cell::RefCell;

const SIMPLE_STRINGS: u8 = b'$';
const NUMBER: u8 = b':';
const BULK_STRINGS: u8 = b'+';
const ERRORS: u8 = b'-';
const ARRAYS: u8 = b'*';

#[derive(Debug)]
pub enum Type {
    Array(Vec<String>),
    Integer(Vec<u8>),
    Status(Vec<u8>),
    Error(Vec<u8>),
    String(String),
    None,
}

pub struct Decode<'a> {
    stream: & 'a TcpStream,
    result: Type,
}

impl<'a> Decode<'a> {
    pub fn new(stream: & 'a TcpStream) -> Decode<'a> {
        Decode {
            stream: stream,
            result: Type::None,
        }
    }

    pub fn parse(&mut self) -> Result<(), Error> {
        let buff: BufReader<& 'a TcpStream> = BufReader::new(self.stream);
        let mut lines: Lines<BufReader<& 'a TcpStream>> = buff.lines();
        match lines.next() {
            Some(line) => match line {
                Ok(line_string) => {
                    let bytes: &[u8] = line_string.as_bytes();
                    if bytes.is_empty() {
                        Err(format_err!("it's empty"))
                    } else {
                        match bytes[0] {
                            BULK_STRINGS => {
                                self.result = Self::parse_status(&bytes[1..]);
                            }
                            NUMBER => {
                                self.result = Self::parse_integer(&bytes[1..]);
                            }
                            ERRORS => {
                                self.result = Self::parse_error(&bytes[1..]);
                            }
                            SIMPLE_STRINGS => {
                                self.result = Self::parse_string(&bytes[1..], &mut lines)?;
                            }
                            ARRAYS => {
                                self.result = Self::parse_array(&bytes[1..], &RefCell::new(lines))?;
                            }
                            _ => {}
                        }
                        Ok(())
                    }
                }
                Err(e) => Err(format_err!("{}", e.to_string())),
            },
            None => Err(format_err!("")),
        }
    }

    fn parse_simple_string(&self) {}

    fn parse_status(bytes: &[u8]) -> Type {
        Type::Status(bytes.to_vec())
    }

    fn parse_integer(bytes: &[u8]) -> Type {
        Type::Integer(bytes.to_vec())
    }

    fn parse_error(bytes: &[u8]) -> Type {
        Type::Error(bytes.to_vec())
    }

    fn parse_string(bytes: &[u8], lines: &mut Lines<BufReader<& 'a TcpStream>>) -> Result<Type, Error>
    {
        // let length: i64 = bytes.iter().fold(0i64, |prev: i64, item: &u8| {
        //     prev = (prev * 10i64) + (*item).into();
        //     prev
        // });

        match lines.next() {
            Some(item) => {
                match item {
                    Ok(result) => Ok(Type::String(result)),
                    Err(e) => Err(format_err!("{:?}", e.to_string()))
                }
            },
            None => {
                Err(format_err!("{:?}", "empty"))
            }
        }
    }

    fn parse_array(bytes: &[u8], lines: &RefCell<Lines<BufReader<& 'a TcpStream>>>) -> Result<Type, Error> {
        let length: i64 = bytes.iter().fold(0i64, |prev: i64, item: &u8| {
            (prev * 10i64) + (*item as i8) as i64
        });

        let mut array: Vec<String> = Vec::new();

        for line in 0..length {
            match lines.borrow_mut().next() {
                Some(line) => {
                    match line {
                        Ok(line_string) => {
                            let bytes: &[u8] = line_string.as_bytes();
                            if bytes.len() == 0 {
                                return Err(format_err!("it's empty"));
                            } else {
                                match bytes[0] {
                                    SIMPLE_STRINGS => {
                                        match Self::parse_string(&bytes[1..], &mut lines.borrow_mut()) {
                                            Ok(result) => {
                                                match result {
                                                    Type::String(s) => {
                                                        array.push(s);
                                                    },
                                                    _ => {

                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                return Err(format_err!("{:?}", e.to_string()))
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        },
                        Err(e) => {
                            return Err(format_err!("{:?}", e.to_string()));
                        }
                    }
                },
                None => {
                    return Err(format_err!("array empty"));
                }
            }
        }
        Ok(Type::Array(array))
    }

    pub fn get_result(&self) -> &Type {
        &self.result
    }
}

#[test]
fn test_parse_status() {
    use std::io::Write;
    use std::net::{Shutdown};
    let mut stream = TcpStream::connect("127.0.0.1:6379").unwrap();
    stream.write("*3\r\n$3\r\nSET\r\n$1\r\nh\r\n$3\r\n123\r\n".as_bytes()).unwrap();

    let mut decode: Decode<'_> = Decode::new(&stream);

    decode.parse().unwrap();
    match decode.get_result() {
        Type::Status(item) => {assert_eq!(item, &vec![b'O',b'K']);},
        _ => {},
    }

    
    stream.shutdown(Shutdown::Both).unwrap();
}

#[test]
fn test_parse_string() {
    use std::io::Write;
    use std::net::{Shutdown};
    let mut stream = TcpStream::connect("127.0.0.1:6379").unwrap();
    stream.write("*2\r\n$3\r\nGET\r\n$1\r\nh\r\n".as_bytes()).unwrap();
    let mut decode: Decode<'_> = Decode::new(&stream);
    decode.parse().unwrap();
    match decode.get_result() {
        Type::String(item) => {assert_eq!(item, &String::from("123"));},
        _ => {},
    }

    stream.shutdown(Shutdown::Both).unwrap();
}

#[test]
fn test_parse_array() {
    
}