use std::fmt;

enum RegExpr{
    Character(char),
    Range(Vec<char>),
    Repeation(Box<RegExpr>),
    Branch(Box<RegExpr>,Box<RegExpr>),
    Sequence(Box<RegExpr>,Box<RegExpr>)
}

impl fmt::Debug for RegExpr{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match *self{
            RegExpr::Character(ref c) => write!(f, "{}", c),
            RegExpr::Range(ref range) => write!(f, "({:?})",range),
            RegExpr::Repeation(ref expr) => write!(f, "({:?}*)", expr),
            RegExpr::Branch(ref lhs,ref rhs) => write!(f, "({:?}|{:?})", lhs, rhs),
            RegExpr::Sequence(ref car,ref cdr) => write!(f, "({:?}{:?})", car, cdr)
        }
    }
}

#[derive(Debug)]
struct ParseError;

fn range<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
    let mut buffer = Vec::new();
    loop{
        match input.next(){
            Some(']') => break,
            Some(c) => buffer.push(c),
            None => return Err(ParseError{})
        }
    }
    Ok(RegExpr::Range(buffer))
}

fn paren<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
	let mut level = 0;
	let mut buffer: Vec<char> = Vec::new();
	loop{
		match input.next(){
			Some(')') => {
				if level == 0 {
					break;
				}
				else{
					level -= 1
				}
			},
			Some('(') => { 
				level += 1
			},
			Some(c) => buffer.push(c),
			None => return Err(ParseError)
		}
	}
	parse(&mut buffer.into_iter())
}

fn parse_one<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
	match input.next(){
        Some('[') => range(input),
		Some('(') => paren(input),
        Some(c) if c != '*' && c != '|' => Ok(RegExpr::Character(c)),
        _ => Err(ParseError{})
    }
}

fn parse<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
	match parse_one(input){
		e @ Err(_) => e,
		Ok(expr) => {
			match input.next(){
				Some('|') => {
					match parse(input){
						Ok(rhs) => Ok(RegExpr::Branch(Box::new(expr),Box::new(rhs))),
						e @ Err(_) => e
					}
				},
				Some('*') => {
					let expr = RegExpr::Repeation(Box::new(expr));
					match input.next(){
						Some(c) => {
							match parse(&mut Some(c).into_iter().chain::<Box<Iterator<Item=char>>>(Box::new(input))){
								Ok(rhs) => Ok(RegExpr::Sequence(Box::new(expr),Box::new(rhs))),
								e @ Err(_) => e
							}
						},
						None => Ok(expr) 
					}
				},
				Some(c) => {
					match parse(&mut Some(c).into_iter().chain::<Box<Iterator<Item=char>>>(Box::new(input))){
						Ok(rhs) => Ok(RegExpr::Sequence(Box::new(expr),Box::new(rhs))),
						e @ Err(_) => e
					}
				},
				None => Ok(expr)
			}
		}
	}
}

fn main() {
    let input = "a*i(u|e)*o".to_owned();
    let expression = parse(&mut input.chars());
    
	println!("{}",input);
    println!("{:?}",expression);
}
