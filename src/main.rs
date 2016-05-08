use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::io::Write;

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

trait Traversable {
    fn traverse<F>(&self, f: &mut F, visited: &mut HashSet<usize>) where F: FnMut(&Self);
}

struct Node {
    id: usize,
    successors: HashMap<Option<char>, Vec<Rc<RefCell<Node>>>>,
    is_end: bool,
}

impl Node {
    fn new(alloc: &mut NodeAllocator) -> Node {
        Node {
            id: alloc.next_id(),
            successors: HashMap::new(),
            is_end: false,
        }
    }

    fn new_with_end(is_end: bool, alloc: &mut NodeAllocator) -> Node {
        Node {
            id: alloc.next_id(),
            successors: HashMap::new(),
            is_end: is_end,
        }
    }

    fn dotty_print<W: Write + ?Sized>(&self,writer: &mut W) {
        let mut visited = HashSet::new();

        writeln!(writer,"digraph g{{").unwrap();

        self.traverse(&mut |this: &Node| {
                          if this.is_end{
                              writeln!(writer,"{} [ style = \"bold\" ];",this.id).unwrap();
                          }
                          for (condition, nodes) in &this.successors {
                              for node in nodes {
                                  if let &Some(c) = condition {
                                      writeln!(writer,"\t{} -> {} [ label = \"{}\" ];",
                                             this.id,
                                             node.borrow().id,
                                             c).unwrap();
                                  } else {
                                      writeln!(writer,"\t{} -> {} [ label = \"ε\" ];",
                                             this.id,
                                             node.borrow().id).unwrap();
                                  }
                              }
                          }
                      },
                      &mut visited);

        writeln!(writer,"}}").unwrap();
    }
}

impl Traversable for Node {
    fn traverse<F>(&self, f: &mut F, visited: &mut HashSet<usize>)
        where F: FnMut(&Self)
    {
        if visited.contains(&self.id) {
            return;
        }

        f(&self);

        visited.insert(self.id);

        for (_, nodes) in &self.successors {
            for node in nodes {
                node.borrow().traverse(f, visited);
            }
        }
    }
}

impl Traversable for Rc<RefCell<Node>> {
    fn traverse<F>(&self, f: &mut F, visited: &mut HashSet<usize>)
        where F: FnMut(&Self)
    {
        if visited.contains(&self.borrow().id) {
            return;
        }

        f(&self);

        visited.insert(self.borrow().id);

        for (_, nodes) in &self.borrow().successors {
            for node in nodes {
                node.traverse(f, visited);
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

    fn dotty_print<W: Write + ?Sized>(&self,writer: &mut W) {
        self.start.borrow().dotty_print(writer);
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
            (*intermediate.start)
                .borrow_mut()
                .successors
                .entry(None)
                .or_insert(vec![])
                .push(intermediate.end.clone());
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

fn merge_by_epsilon(graph: &Graph, alloc: &mut NodeAllocator) -> Rc<RefCell<Node>> {
    let mut indexer: HashMap<usize, Rc<RefCell<Node>>> = HashMap::new();

    graph.start.traverse(&mut |this: &Rc<RefCell<Node>>|{
        println!("reached id: {}, is_end: {}",this.borrow().id,this.borrow().is_end);
        indexer.insert(this.borrow().id,Rc::new(RefCell::new(Node::new_with_end(this.borrow().is_end,alloc))));
    },&mut HashSet::new());

    let mut count = 0;
    graph.start.traverse(&mut |this: &Rc<RefCell<Node>>|{
        for (condition, successors) in &this.borrow().successors{
            for successor in successors{
                println!("edge ({:?}) -> {}", condition, successor.borrow().id);
                match condition{
                    &Some(c) => {
                        (*indexer[&this.borrow().id]).borrow_mut()
                            .successors.entry(Some(c)).or_insert(vec![])
                            .push(indexer[&successor.borrow().id].clone());
                    }
                    &None => {
                        let mut joined_successors: HashMap<Option<char>,Vec<Rc<RefCell<Node>>>> = HashMap::new();
                        {
                            let joined_mut = joined_successors.borrow_mut();
                            for (c,succs) in &indexer[&successor.borrow().id].borrow().successors{
                                for succ in succs{
                                    joined_mut.entry(*c).or_insert(vec![]).push(succ.clone());
                                }
                            }
                        }
                        (*indexer[&this.borrow().id]).borrow_mut().successors.extend(joined_successors);
                        if indexer[&successor.borrow().id].borrow().is_end {
                            (*indexer[&this.borrow().id]).borrow_mut().is_end = true;
                        }
                        *indexer.get_mut(&successor.borrow().id).unwrap() = indexer[&this.borrow().id].clone();
                    }
                }
            }
        }
        count += 1;
        println!("iter{} reached",count);
    use std::fs::File;
    use std::io::Write;
        indexer[&graph.start.borrow().id].borrow().dotty_print(&mut File::create(format!("iter{}.dot",count)).unwrap());
    },&mut HashSet::new());

    indexer[&graph.start.borrow().id].clone()
}

fn main() {
    use std::fs::File;
    let input = "(a|bc)*d".to_owned();
    let expression = parse(&mut input.chars());

    let mut alloc = NodeAllocator::new();
    let nfa = build_nfa(&expression.unwrap(), &mut alloc);
    (*nfa.end).borrow_mut().is_end = true;
    nfa.dotty_print(&mut File::create("nfa.dot").unwrap());

    let merged = merge_by_epsilon(&nfa, &mut alloc);
    merged.borrow().dotty_print(&mut File::create("dfa.dot").unwrap());
}
