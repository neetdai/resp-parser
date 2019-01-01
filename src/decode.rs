use failure::format_err;
use failure::Error;
use std::cell::RefCell;
use std::io::{BufReader, Lines, Read};
use std::net::TcpStream;
use std::iter::FromIterator;
use std::str::FromStr;

const SIMPLE_STRINGS: char = '+';
const NUMBER: char = ':';
const BULK_STRINGS: char = '$';
const ERRORS: char = '-';
const ARRAYS: char = '*';

#[derive(Debug)]
pub enum Type {
    Array(Vec<String>),
    Integer(String),
    Status(String),
    Error(String),
    String(String),
    None,
}

pub struct Decode<'a, R>
where
    R: 'a + Read,
{
    stream: &'a mut R,
    result: Type,
}

impl<'a, R> Decode<'a, R>
where
    R: 'a + Read,
{
    pub fn new(stream: &'a mut R) -> Decode<'a, R> {
        Decode {
            stream: stream,
            result: Type::None,
        }
    }

    pub fn parse(&mut self) -> Result<(), Error> {
        let mut buff: String = String::new();
        self.stream.read_to_string(&mut buff);

        if buff.len() == 0 {
            return Err(format_err!("buff is empty"));
        } else {
            match buff.remove(0) {
                NUMBER => {
                    buff.trim_end_matches("\r\n");
                    self.result = Self::parse_integer(buff);
                },
                BULK_STRINGS => {
                	let mut isize: String = std::str::FromStr::from_str(&buff.chars().take_while(|b| {
                		b != &'\n'
                	}).collect::<String>().trim_end_matches("\r").to_string())?;

                },
                ERRORS => {
                    buff.trim_end_matches("\r\n");
                    self.result = Self::parse_error(buff);
                },
                SIMPLE_STRINGS => {
                    buff.trim_end_matches("\r\n");
                    self.result = Self::parse_status(buff);
                },
                ARRAYS => {

                },
                _ => {
                    return Err(format_err!("undefine sign"))
                }
            }
        }
        Ok(())
    }

    fn parse_status(buff: String) -> Type {
        Type::Status(buff)
    }

    fn parse_integer(buff: String) -> Type {
        Type::Integer(buff)
    }

    fn parse_error(buff: String) -> Type {
        Type::Error(buff)
    }

    fn parse_string(
        bytes: &[u8],
        lines: &mut Lines<BufReader<&'a TcpStream>>,
    ) -> Result<Type, Error> {
        // let length: i64 = bytes.iter().fold(0i64, |prev: i64, item: &u8| {
        //     prev = (prev * 10i64) + (*item).into();
        //     prev
        // });

        match lines.next() {
            Some(item) => match item {
                Ok(result) => Ok(Type::String(result)),
                Err(e) => Err(format_err!("{:?}", e.to_string())),
            },
            None => Err(format_err!("{:?}", "empty")),
        }
    }

    fn parse_array(
        bytes: &[u8],
        lines: &RefCell<Lines<BufReader<&'a TcpStream>>>,
    ) -> Result<Type, Error> {
        let length: i64 = bytes.iter().fold(0i64, |prev: i64, item: &u8| {
            (prev * 10i64) + (*item as i8) as i64
        });

        let mut array: Vec<String> = Vec::new();

        for line in 0..length {
            match lines.borrow_mut().next() {
                Some(line) => match line {
                    Ok(line_string) => {
                        let bytes: &[u8] = line_string.as_bytes();
                        if bytes.len() == 0 {
                            return Err(format_err!("it's empty"));
                        } else {
                            // match bytes[0] {
                            //     SIMPLE_STRINGS => {
                            //         match Self::parse_string(&bytes[1..], &mut lines.borrow_mut()) {
                            //             Ok(result) => match result {
                            //                 Type::String(s) => {
                            //                     array.push(s);
                            //                 }
                            //                 _ => {}
                            //             },
                            //             Err(e) => return Err(format_err!("{:?}", e.to_string())),
                            //         }
                            //     }
                            //     _ => {}
                            // }
                        }
                    }
                    Err(e) => {
                        return Err(format_err!("{:?}", e.to_string()));
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
    use std::net::Shutdown;
    let mut stream = TcpStream::connect("127.0.0.1:6379").unwrap();
    stream
        .write("*3\r\n$3\r\nSET\r\n$1\r\nh\r\n$3\r\n123\r\n".as_bytes())
        .unwrap();

    let mut decode: Decode<'_, _> = Decode::new(&mut stream);

    decode.parse().unwrap();
    match decode.get_result() {
        Type::Status(item) => {
            assert_eq!(item, &String::from("OK"));
        }
        _ => {}
    }

    stream.shutdown(Shutdown::Both).unwrap();
}

#[test]
fn test_parse_string() {
    use std::io::Write;
    use std::net::Shutdown;
    let mut stream = TcpStream::connect("127.0.0.1:6379").unwrap();
    stream
        .write("*2\r\n$3\r\nGET\r\n$1\r\nh\r\n".as_bytes())
        .unwrap();
    let mut decode: Decode<'_, _> = Decode::new(&mut stream);
    decode.parse().unwrap();
    match decode.get_result() {
        Type::String(item) => {
            assert_eq!(item, &String::from("123"));
        }
        _ => {}
    }
    stream.write(b"*2\r\n$3\r\ndel\r\n*1\rnh\r\n").unwrap();
    stream.shutdown(Shutdown::Both).unwrap();
}

#[test]
fn test_parse_array() {
    use std::io::Write;
    use std::net::Shutdown;
    let mut stream = TcpStream::connect("127.0.0.1:6379").unwrap();
    stream
        .write(b"*3\r\n$5\r\nLPUSH\r\n$1\r\na\r\n$1\r\n1\r\n")
        .unwrap();
    let mut decode: Decode<'_, _> = Decode::new(&mut stream);
    decode.parse().unwrap();
    stream
        .write(b"*4\r\n$6\r\nlrange\r\n$1\r\na\r\n$1\r\n0\r\n$2\r\n-1\r\n")
        .unwrap();
    let mut decode: Decode<'_, _> = Decode::new(&mut stream);

    decode.parse().unwrap();
    match decode.get_result() {
        Type::Array(item) => {
            assert_eq!(item, &vec![String::from("2")]);
        }
        Type::Integer(item) => {
            assert_eq!(item, &String::from("1"));
        }
        Type::Status(item) => {
            assert_eq!(item, &String::from("3"));
        }
        Type::Error(item) => {
            assert_eq!(item, &String::from("4"));
        }
        Type::String(item) => {
            assert_eq!(item, &String::from("5"));
        }
        Type::None => {}
    }
    stream.write(b"*2\r\n$3\r\ndel\r\n*1\r\na\r\n").unwrap();
    stream.shutdown(Shutdown::Both);
}
