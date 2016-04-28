use std::fmt;

enum RegExpr{
    Character(char),
    Range(char,char),
    Repeation(Box<RegExpr>),
    Branch(Box<RegExpr>,Box<RegExpr>),
    Sequence(Vec<Box<RegExpr>>)
}

impl fmt::Debug for RegExpr{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match *self{
            RegExpr::Character(ref c) => write!(f, "{}", c),
            RegExpr::Range(ref beg,ref end) => write!(f, "([{}-{}])",beg,end),
            RegExpr::Repeation(ref expr) => write!(f, "({:?}*)", expr),
            RegExpr::Branch(ref lhs,ref rhs) => write!(f, "({:?}|{:?})", lhs, rhs),
            RegExpr::Sequence(ref exprs) => write!(f, "({:?})", exprs)
        }
    }
}

fn main() {
    let expr1 =
        Box::new(RegExpr::Repeation(
            Box::new(RegExpr::Branch(
                Box::new(RegExpr::Character('a')),
                Box::new(RegExpr::Range('A','Z'))
            ))
        ));
    let expr2 =
        Box::new(RegExpr::Branch(
            Box::new(RegExpr::Character('b')),
            Box::new(RegExpr::Character('B'))
        ));
    let expr = RegExpr::Sequence(vec![expr1,expr2]);
    println!("{:?}",expr);
}
