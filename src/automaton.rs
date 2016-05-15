use std::collections::HashSet;
use std::collections::BTreeSet;
use std::io::Write;

use parser::RegExpr;

pub struct NodeAllocator {
    nodes: Vec<Node>,
}

impl NodeAllocator {
    pub fn new() -> NodeAllocator {
        NodeAllocator { nodes: vec![] }
    }
}

#[derive(PartialEq,Eq,PartialOrd,Ord,Debug,Clone,Copy,Hash)]
pub struct Node {
    id: usize,
}

impl Node {
    fn new(alloc: &mut NodeAllocator) -> Node {
        let id = alloc.nodes.len();
        alloc.nodes.push(Node { id: id });
        alloc.nodes[id]
    }
}

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone)]
pub struct Edge {
    condition: Option<char>,
    from: Node,
    to: Node,
}

pub struct Graph {
    start: Node,
    edges: BTreeSet<Edge>,
    acceptors: HashSet<Node>,
}

impl Graph {
    fn new(start: Node) -> Graph {
        Graph {
            start: start,
            edges: BTreeSet::new(),
            acceptors: HashSet::new(),
        }
    }

    fn add_edge(&mut self, condition: Option<char>, from: Node, to: Node) {
        self.edges.insert(Edge {
            condition: condition,
            from: from,
            to: to,
        });
    }

    pub fn dotty_print<W: Write + ?Sized>(&self, writer: &mut W) {
        writeln!(writer, "digraph g{{").unwrap();

        for acceptor in &self.acceptors {
            writeln!(writer, "{} [ style = \"bold\" ];", acceptor.id).unwrap();
        }

        self.traverse(&mut |edge| {
                          if let Some(c) = edge.condition {
                              writeln!(writer,
                                       "\t{} -> {} [ label = \"{}\" ];",
                                       edge.from.id,
                                       edge.to.id,
                                       c)
                                  .unwrap();
                          } else {
                              writeln!(writer,
                                       "\t{} -> {} [ label = \"ε\" ];",
                                       edge.from.id,
                                       edge.to.id)
                                  .unwrap();
                          }
                      },
                      &self.start,
                      &mut HashSet::new());

        writeln!(writer, "}}").unwrap();

    }

    fn traverse<F: FnMut(&Edge)>(&self, f: &mut F, current: &Node, visited: &mut HashSet<Node>) {
        if visited.contains(current) {
            return;
        }

        visited.insert(*current);

        for edge in self.edges.iter().filter(|edge| edge.from == *current) {
            f(edge);
        }

        for edge in self.edges.iter().filter(|edge| edge.from == *current) {
            self.traverse(f, &edge.to, visited);
        }
    }
}

pub fn build_nfa(expr: &RegExpr, alloc: &mut NodeAllocator) -> Graph {
    match *expr {
        RegExpr::Character(c) => {
            let start = Node::new(alloc);
            let end = Node::new(alloc);
            let mut graph = Graph::new(start);
            graph.add_edge(Some(c), start, end);
            graph.acceptors.insert(end);
            graph
        }
        RegExpr::Sequence(ref car, ref cdr) => {
            let mut car = build_nfa(&car, alloc);
            let cdr = build_nfa(&cdr, alloc);
            car.edges.extend(car.acceptors.iter().map(|acceptor| {
                Edge {
                    condition: None,
                    from: acceptor.clone(),
                    to: cdr.start,
                }
            }));
            Graph {
                start: car.start,
                edges: car.edges.union(&cdr.edges).cloned().collect(),
                acceptors: cdr.acceptors,
            }
        }
        RegExpr::Branch(ref lhs, ref rhs) => {
            use std::iter::Iterator;
            let lhs = build_nfa(&lhs, alloc);
            let rhs = build_nfa(&rhs, alloc);
            let start = Node::new(alloc);
            let end = Node::new(alloc);

            let mut graph = Graph::new(start);
            graph.acceptors = [end].iter().map(|node| *node).collect();
            graph.edges = lhs.edges.union(&rhs.edges).cloned().collect();
            graph.edges.insert(Edge {
                condition: None,
                from: start,
                to: lhs.start,
            });
            graph.edges.insert(Edge {
                condition: None,
                from: start,
                to: rhs.start,
            });
            graph.edges.extend(lhs.acceptors.iter().map(|acceptor| {
                Edge {
                    condition: None,
                    from: *acceptor,
                    to: end,
                }
            }));
            graph.edges.extend(rhs.acceptors.iter().map(|acceptor| {
                Edge {
                    condition: None,
                    from: *acceptor,
                    to: end,
                }
            }));

            graph
        }
        RegExpr::Range(ref range) => {
            let start = Node::new(alloc);
            let end = Node::new(alloc);

            Graph {
                start: start,
                edges: range.iter()
                            .map(|&c| {
                                Edge {
                                    condition: Some(c),
                                    from: start,
                                    to: end,
                                }
                            })
                            .collect(),
                acceptors: [end].iter().cloned().collect(),
            }

        }
        RegExpr::Repeation(ref expr) => {
            let mut graph = build_nfa(&expr, alloc);
            graph.acceptors.insert(graph.start);
            let new_edges: Vec<Edge>;
            {
                new_edges = graph.acceptors
                                 .iter()
                                 .flat_map(|acceptor| {
                                     vec![Edge {
                                              condition: None,
                                              from: graph.start,
                                              to: *acceptor,
                                          },
                                          Edge {
                                              condition: None,
                                              from: *acceptor,
                                              to: graph.start,
                                          }]
                                 })
                                 .collect();
            }
            graph.edges.extend(new_edges);
            graph

        }
    }
}

pub fn merge_by_epsilon(graph: &Graph, alloc: &mut NodeAllocator) -> Graph {
    //assume (a,b) is arranged in dictionary order where a is compared first.
    let epsilon_edges: BTreeSet<(Node,Node)> = graph.edges.iter().filter(|edge| edge.condition.is_none()).map(|edge| (edge.from,edge.to)).collect();

    let dfa = Graph{
        start: Node::new(alloc),
        edges: HashSet::new(),
        acceptors: vec![],
    };

    graph.traverse(&mut |edge| {
                
    },graph.start,HashSet::new());
    unimplemented!()
    // let mut indexer: HashMap<usize, Rc<RefCell<Node>>> = HashMap::new();
    //
    // graph.start.traverse(&mut |this: &Rc<RefCell<Node>>|{
    // println!("reached id: {}, is_end: {}",this.borrow().id,this.borrow().is_end);
    // indexer.insert(this.borrow().id,Rc::new(RefCell::new(Node::new_with_end(this.borrow().is_end,alloc))));
    // },&mut HashSet::new());
    //
    // let mut count = 0;
    // graph.start.traverse(&mut |this: &Rc<RefCell<Node>>|{
    // for (condition, successors) in &this.borrow().successors{
    // for successor in successors{
    // println!("edge ({:?}) -> {}", condition, successor.borrow().id);
    // match condition{
    // &Some(c) => {
    // (*indexer[&this.borrow().id]).borrow_mut()
    // .successors.entry(Some(c)).or_insert(vec![])
    // .push(indexer[&successor.borrow().id].clone());
    // }
    // &None => {
    // let mut joined_successors: HashMap<Option<char>,Vec<Rc<RefCell<Node>>>> = HashMap::new();
    // {
    // let joined_mut = joined_successors.borrow_mut();
    // for (c,succs) in &indexer[&successor.borrow().id].borrow().successors{
    // for succ in succs{
    // joined_mut.entry(*c).or_insert(vec![]).push(succ.clone());
    // }
    // }
    // }
    // (*indexer[&this.borrow().id]).borrow_mut().successors.extend(joined_successors);
    // if indexer[&successor.borrow().id].borrow().is_end {
    // (*indexer[&this.borrow().id]).borrow_mut().is_end = true;
    // }
    // indexer.get_mut(&successor.borrow().id).unwrap() = indexer[&this.borrow().id].clone();
    // }
    // }
    // }
    // }
    // count += 1;
    // println!("iter{} reached",count);
    // use std::fs::File;
    // use std::io::Write;
    // indexer[&graph.start.borrow().id].borrow().dotty_print(&mut File::create(format!("iter{}.dot",count)).unwrap());
    // },&mut HashSet::new());
    //
    // indexer[&graph.start.borrow().id].clone()
}
