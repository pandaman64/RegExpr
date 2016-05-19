use std::fmt;

pub enum RegExpr {
    Character(char),
    Range(Vec<char>),
    Repeation(Box<RegExpr>),
    Branch(Box<RegExpr>, Box<RegExpr>),
    Sequence(Box<RegExpr>, Box<RegExpr>),
}

impl fmt::Debug for RegExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RegExpr::Character(ref c) => write!(f, "{}", c),
            RegExpr::Range(ref range) => write!(f, "({:?})", range),
            RegExpr::Repeation(ref expr) => write!(f, "({:?}*)", expr),
            RegExpr::Branch(ref lhs, ref rhs) => write!(f, "({:?}|{:?})", lhs, rhs),
            RegExpr::Sequence(ref car, ref cdr) => write!(f, "({:?}{:?})", car, cdr),
        }
    }
}

#[derive(Debug)]
pub struct ParseError;

fn range<T: Iterator<Item = char>>(input: &mut T) -> Result<RegExpr, ParseError> {
    let mut buffer = Vec::new();
    loop {
        match input.next() {
            Some(']') => break,
            Some(c) => buffer.push(c),
            None => return Err(ParseError {}),
        }
    }
    Ok(RegExpr::Range(buffer))
}

fn paren<T: Iterator<Item = char>>(input: &mut T) -> Result<RegExpr, ParseError> {
    let mut level = 0;
    let mut buffer: Vec<char> = Vec::new();
    loop {
        match input.next() {
            Some(')') => {
                if level == 0 {
                    break;
                } else {
                    level -= 1;
                    buffer.push(')');
                }
            }
            Some('(') => {
                level += 1;
                buffer.push('(');
            },
            Some(c) => buffer.push(c),
            None => return Err(ParseError),
        }
    }
    parse(&mut buffer.into_iter())
}

fn parse_one<T: Iterator<Item = char>>(input: &mut T) -> Result<RegExpr, ParseError> {
    match input.next() {
        Some('[') => range(input),
        Some('(') => paren(input),
        Some(c) if c != '*' && c != '|' => Ok(RegExpr::Character(c)),
        _ => Err(ParseError {}),
    }
}

pub fn parse<T: Iterator<Item = char>>(input: &mut T) -> Result<RegExpr, ParseError> {
    parse_one(input).and_then(|expr| {
        match input.next() {
            Some('|') => {
                parse(input).and_then(|rhs| Ok(RegExpr::Branch(Box::new(expr), Box::new(rhs))))
            }
            Some('*') => {
                let expr = RegExpr::Repeation(Box::new(expr));
                match input.next() {
                    Some(c) => {
                        parse(&mut Some(c)
                                       .into_iter()
                                       .chain::<Box<Iterator<Item = char>>>(Box::new(input)))
                            .and_then(|rhs| Ok(RegExpr::Sequence(Box::new(expr), Box::new(rhs))))
                    }
                    None => Ok(expr),
                }
            }
            Some(c) => {
                parse(&mut Some(c)
                               .into_iter()
                               .chain::<Box<Iterator<Item = char>>>(Box::new(input)))
                    .and_then(|rhs| Ok(RegExpr::Sequence(Box::new(expr), Box::new(rhs))))
            }
            None => Ok(expr),
        }
    })
}
