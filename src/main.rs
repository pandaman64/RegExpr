use std::fmt;
use std::io;
use std::str;

enum RegExpr{
    Character(char),
    Range(Vec<char>),
    Repeation(Box<RegExpr>),
    Branch(Box<RegExpr>,Box<RegExpr>),
    Sequence(Vec<Box<RegExpr>>)
}

impl fmt::Debug for RegExpr{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match *self{
            RegExpr::Character(ref c) => write!(f, "{}", c),
            RegExpr::Range(ref range) => write!(f, "({:?})",range),
            RegExpr::Repeation(ref expr) => write!(f, "({:?}*)", expr),
            RegExpr::Branch(ref lhs,ref rhs) => write!(f, "({:?}|{:?})", lhs, rhs),
            RegExpr::Sequence(ref exprs) => write!(f, "({:?})", exprs)
        }
    }
}

#[derive(Debug)]
struct ParseError;

fn character<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
   match input.next(){
       Some(c) => Ok(RegExpr::Character(c)),
       None => Err(ParseError{}) 
   }
}

fn range<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
    unimplemented!()
}

fn repeation<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
    unimplemented!()
}

fn branch<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
    unimplemented!()
}

fn sequence<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
    unimplemented!()
}

fn parse<T: Iterator<Item=char>>(input: &mut T) -> Result<RegExpr,ParseError>{
    let mut input = input.peekable();
    character(&mut input)
}

fn main() {
    /*
    let expr1 =
        Box::new(RegExpr::Repeation(
            Box::new(RegExpr::Branch(
                Box::new(RegExpr::Character('a')),
                Box::new(RegExpr::Range(vec!['A','B','C','Z']))
            ))
        ));
    let expr2 =
        Box::new(RegExpr::Branch(
            Box::new(RegExpr::Character('b')),
            Box::new(RegExpr::Character('B'))
        ));
    let expr = RegExpr::Sequence(vec![expr1,expr2]);
    */
    let mut input = String::new();
    let expression = match io::stdin().read_line(&mut input){
        Ok(_) => parse(&mut input.chars()),
        Err(error) => panic!("err {}", error)
    };
    println!("{:?}",expression);
}
