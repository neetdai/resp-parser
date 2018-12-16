use failure::format_err;
use failure::Error;
use std::cell::RefCell;
use std::io::{BufRead, BufReader, Lines, Read};
use std::str::FromStr;

const BULK_STRINGS: u8 = b'$';
const NUMBER: u8 = b':';
const SIMPLE_STRINGS: u8 = b'+';
const ERRORS: u8 = b'-';
const ARRAYS: u8 = b'*';

#[derive(Debug)]
pub enum Type {
    Array(Vec<String>),
    Integer(Vec<u8>),
    Status(Vec<u8>),
    Error(Vec<u8>),
    String(String),
}

pub struct Decode {
    result: Option<Type>,
}

impl Decode {
    pub fn parse<R>(&mut self, reader: R) -> Result<(), Error>
    where
        R: Read,
    {
        let buff: BufReader<R> = BufReader::new(reader);
        let mut lines: Lines<BufReader<R>> = buff.lines();
        match lines.next() {
            Some(line) => match line {
                Ok(line_string) => {
                    let bytes: &[u8] = line_string.as_bytes();
                    if bytes.is_empty() {
                        Err(format_err!("it's empty"))
                    } else {
                        match bytes[0] {
                            SIMPLE_STRINGS => {
                                self.result = Some(self.parse_status(&bytes[1..]));
                            }
                            NUMBER => {
                                self.result = Some(self.parse_integer(&bytes[1..]));
                            }
                            ERRORS => {
                                self.result = Some(self.parse_error(&bytes[1..]));
                            }
                            BULK_STRINGS => {
                                self.result = Some(self.parse_string(&bytes[1..], &mut lines));
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

    fn parse_status(&self, bytes: &[u8]) -> Type {
        Type::Status(bytes.to_vec())
    }

    fn parse_integer(&self, bytes: &[u8]) -> Type {
        Type::Integer(bytes.to_vec())
    }

    fn parse_error(&self, bytes: &[u8]) -> Type {
        Type::Error(bytes.to_vec())
    }

    fn parse_string<R>(&self, bytes: &[u8], lines: &mut Lines<BufReader<R>>) -> Type
    where
        R: Read,
    {

    }
}

#[test]
fn test_parse() {
    use std::net::TcpStream;
    let stream = TcpStream::connect("127.0.0.1:6379").unwrap();
}
