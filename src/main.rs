use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::BorrowMut;
use std::collections::HashSet;

enum RegExpr {
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
struct ParseError;

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
                    level -= 1
                }
            }
            Some('(') => level += 1,
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

fn parse<T: Iterator<Item = char>>(input: &mut T) -> Result<RegExpr, ParseError> {
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

struct NodeAllocator {
    new_id: usize,
}

impl NodeAllocator {
    fn new() -> NodeAllocator {
        NodeAllocator { new_id: 0 }
    }

    fn next_id(&mut self) -> usize {
        let id = self.new_id;
        self.new_id += 1;
        id
    }
}

struct Node {
    id: usize,
    successors: HashMap<Option<char>, Vec<Rc<RefCell<Node>>>>,
}

impl Node {
    fn new(alloc: &mut NodeAllocator) -> Node {
        Node {
            id: alloc.next_id(),
            successors: HashMap::new(),
        }
    }

    fn traverse<F>(&self, f: &Box<F>, visited: &mut HashSet<usize>)
        where F: Fn(&Self)
    {
        if visited.contains(&self.id) {
            return;
        }

        f(&self);

        visited.insert(self.id);

        for (_, nodes) in &self.successors {
            for node in nodes {
                node.borrow().traverse(&f, visited);
            }
        }
    }
}

struct Graph {
    start: Rc<RefCell<Node>>,
    end: Rc<RefCell<Node>>,
}

impl Graph {
    fn concatenated(self, other: Graph) -> Graph {
        (*self.end).borrow_mut().successors.entry(None).or_insert(vec![]).push(other.start);
        Graph {
            start: self.start.clone(),
            end: other.end.clone(),
        }
    }

    fn dotty_print(&self) {
        let mut visited = HashSet::new();

        println!("digraph g{{");

        self.start.borrow().traverse(&Box::new(|this: &Node| {
                                         for (condition, nodes) in &this.successors {
                                             for node in nodes {
                                                 if let &Some(c) = condition {
                                                     println!("\t{} -> {} [ label = \"{}\" ];",
                                                              this.id,
                                                              node.borrow().id,
                                                              c);
                                                 } else {
                                                     println!("\t{} -> {} [ label = \"ε\" ];",
                                                              this.id,
                                                              node.borrow().id);
                                                 }
                                             }
                                         }
                                     }),
                                     &mut visited);

        println!("}}");
    }
}

fn build_nfa(expr: &RegExpr, alloc: &mut NodeAllocator) -> Graph {
    // All subgraphs must end with Graph::end
    match *expr {
        RegExpr::Character(c) => {
            let end = Rc::new(RefCell::new(Node::new(alloc)));
            let mut start = Node::new(alloc);
            start.successors.insert(Some(c), vec![end.clone()]);
            Graph {
                start: Rc::new(RefCell::new(start)),
                end: end,
            }
        }
        RegExpr::Sequence(ref car, ref cdr) => {
            let car = build_nfa(&car, alloc);
            let cdr = build_nfa(&cdr, alloc);
            let intermediate = car.concatenated(cdr);
            intermediate
        }
        RegExpr::Branch(ref lhs, ref rhs) => {
            let end = Rc::new(RefCell::new(Node::new(alloc)));
            let mut start = Node::new(alloc);
            let lhs = build_nfa(&lhs, alloc);
            let rhs = build_nfa(&rhs, alloc);
            start.successors.insert(None, vec![lhs.start.clone(), rhs.start.clone()]);
            (*lhs.end).borrow_mut().successors.entry(None).or_insert(vec![]).push(end.clone());
            (*rhs.end).borrow_mut().successors.entry(None).or_insert(vec![]).push(end.clone());

            Graph {
                start: Rc::new(RefCell::new(start)),
                end: end,
            }
        }
        RegExpr::Range(ref range) => {
            let end = Rc::new(RefCell::new(Node::new(alloc)));
            let mut start = Node::new(alloc);
            for c in range {
                start.successors.insert(Some(*c), vec![end.clone()]);
            }
            Graph {
                start: Rc::new(RefCell::new(start)),
                end: end,
            }
        }
        RegExpr::Repeation(ref expr) => {
            let intermediate = build_nfa(&expr, alloc);
            (*intermediate.end)
                .borrow_mut()
                .successors
                .entry(None)
                .or_insert(vec![])
                .push(intermediate.start.clone());

            intermediate
        }
    }
}

fn main() {
    let input = "a*i(u|e)*o".to_owned();
    let expression = parse(&mut input.chars());

    let mut alloc = NodeAllocator::new();
    let nfa = build_nfa(&expression.unwrap(), &mut alloc);
    nfa.dotty_print();
}
