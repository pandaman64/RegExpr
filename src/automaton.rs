use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::io::Write;

use parser::RegExpr;

pub struct NodeAllocator {
    new_id: usize,
}

impl NodeAllocator {
    pub fn new() -> NodeAllocator {
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

pub struct Node {
    id: usize,
    successors: HashMap<Option<char>, Vec<Rc<RefCell<Node>>>>,
    pub is_end: bool,
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

    pub fn dotty_print<W: Write + ?Sized>(&self,writer: &mut W) {
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

pub struct Graph {
    pub start: Rc<RefCell<Node>>,
    pub end: Rc<RefCell<Node>>,
}

impl Graph {
    fn concatenated(self, other: Graph) -> Graph {
        (*self.end).borrow_mut().successors.entry(None).or_insert(vec![]).push(other.start);
        Graph {
            start: self.start.clone(),
            end: other.end.clone(),
        }
    }

    pub fn dotty_print<W: Write + ?Sized>(&self,writer: &mut W) {
        self.start.borrow().dotty_print(writer);
    }
}

pub fn build_nfa(expr: &RegExpr, alloc: &mut NodeAllocator) -> Graph {
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

pub fn merge_by_epsilon(graph: &Graph, alloc: &mut NodeAllocator) -> Rc<RefCell<Node>> {
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
