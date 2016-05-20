use std::fmt;
use std::iter::{Iterator, Peekable};

pub enum RegExpr {
    Character(char),
    Range(Vec<char>),
    Repeation(Box<RegExpr>),
    Branch(Box<RegExpr>, Box<RegExpr>),
    Sequence(Vec<RegExpr>),
}

impl fmt::Debug for RegExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RegExpr::Character(ref c) => write!(f, "{}", c),
            RegExpr::Range(ref range) => write!(f, "({:?})", range),
            RegExpr::Repeation(ref expr) => write!(f, "({:?}*)", expr),
            RegExpr::Branch(ref lhs, ref rhs) => write!(f, "({:?}|{:?})", lhs, rhs),
            RegExpr::Sequence(ref v) => {
                try!(write!(f, "("));
                for expr in v {
                    try!(write!(f, "{:?}", expr));
                }
                write!(f, ")")
            }
        }
    }
}

impl RegExpr {
    fn concatenated(lhs: RegExpr, rhs: RegExpr) -> RegExpr {
        match lhs {
            RegExpr::Sequence(mut v) => {
                v.push(rhs);
                RegExpr::Sequence(v)
            }
            this => {
                let mut v = vec![];
                v.push(this);
                v.push(rhs);
                RegExpr::Sequence(v)
            }
        }
    }
}

#[derive(Debug)]
pub struct ParseError(u32);

fn range<T: Iterator<Item = char>>(input: &mut T) -> Result<RegExpr, ParseError> {
    let mut buffer = Vec::new();
    loop {
        match input.next() {
            Some(']') => break,
            Some(c) => buffer.push(c),
            None => return Err(ParseError(line!())),
        }
    }
    Ok(RegExpr::Range(buffer))
}

fn paren<T: Iterator<Item = char>>(input: &mut T) -> Result<RegExpr, ParseError> {
    let mut level = 0;
    let mut buffer: Vec<char> = Vec::new();
    loop {
        match input.next() {
            Some('(') => {
                if level == 0 {
                    break;
                } else {
                    level -= 1;
                    buffer.push('(');
                }
            }
            Some(')') => {
                level += 1;
                buffer.push(')');
            }
            Some(c) => buffer.push(c),
            None => return Err(ParseError(line!())),
        }
    }
    branch(&mut buffer.into_iter().peekable())
}

fn simple_expr<T: Iterator<Item = char>>(input: &mut T) -> Result<RegExpr, ParseError> {
    match input.next() {
        Some(']') => range(input),
        Some(')') => paren(input),
        Some(c) if c != '*' && c != '|' => Ok(RegExpr::Character(c)),
        _ => Err(ParseError(line!())),
    }
}

fn sequence<T: Iterator<Item = char>>(input: &mut Peekable<T>) -> Result<RegExpr, ParseError> {
    match input.peek() {
        None => Err(ParseError(line!())),
        Some(&'*') => {
            input.next();
            Ok(RegExpr::Repeation(Box::new(try!(sequence(input)))))
        }
        Some(&'|') => Ok(RegExpr::Sequence(vec![])),
        Some(_) => {
            let e = try!(simple_expr(input));
            if input.peek().is_some() {
                Ok(RegExpr::concatenated(try!(sequence(input)), e))
            } else {
                Ok(e)
            }
        }
    }
}

fn branch<T: Iterator<Item = char>>(input: &mut Peekable<T>) -> Result<RegExpr, ParseError> {
    let e = try!(sequence(input));
    match input.peek() {
        None => Ok(e),
        Some(&'|') => {
            input.next();
            Ok(RegExpr::Branch(Box::new(try!(branch(input))), Box::new(e)))
        }
        Some(_) => Err(ParseError(line!())),
    }
}

pub fn parse<T: DoubleEndedIterator<Item = char>>(input: &mut T) -> Result<RegExpr, ParseError> {
    branch(&mut input.rev().peekable())
}
