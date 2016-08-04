use std::collections::HashSet;
use std::collections::HashMap;
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

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Debug)]
pub struct Edge {
    condition: Option<char>,
    from: Node,
    to: Node,
}

#[derive(Debug)]
pub struct Graph {
    start: Node,
    edges: BTreeSet<Edge>,
    acceptors: BTreeSet<Node>,
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone)]
pub struct DFANode {
    pub nodes: BTreeSet<Node>,
    pub is_acceptor: bool,
}
impl DFANode {
    fn new(nodes: BTreeSet<Node>, graph: &Graph) -> DFANode {
        let is_acceptor = nodes.intersection(&graph.acceptors).next().is_some();
        DFANode {
            nodes: nodes,
            is_acceptor: is_acceptor,
        }
    }

    fn pretty_name(&self) -> String {
        format!("\"{{ {} }}\"",
                self.nodes.iter().map(|node| format!("{}", node.id)).collect::<Vec<_>>().join(","))
    }
}

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct DFAEdge {
    pub condition: char,
    pub from: DFANode,
    pub to: DFANode,
}

#[derive(Debug)]
pub struct DFA {
    pub start: DFANode,
    pub edges: BTreeSet<DFAEdge>,
}

impl DFA {
    fn new(start: DFANode) -> DFA {
        DFA {
            start: start,
            edges: BTreeSet::new(),
        }
    }

    pub fn dotty_print<W: Write + ?Sized>(&self, writer: &mut W) {
        writeln!(writer, "digraph g{{").unwrap();

        for edge in &self.edges {
            writeln!(writer,
                     "\t{} -> {} [ label = \"{}\" ];",
                     edge.from.pretty_name(),
                     edge.to.pretty_name(),
                     edge.condition)
                .unwrap();
            if edge.from.is_acceptor {
                writeln!(writer,
                         "\t{} [ style = \"bold\" ];",
                         edge.from.pretty_name())
                    .unwrap();
            }
            if edge.to.is_acceptor {
                writeln!(writer, "\t{} [ style = \"bold\" ];", edge.to.pretty_name()).unwrap();
            }
        }

        writeln!(writer, "}}").unwrap();
    }
}

impl Graph {
    fn new(start: Node) -> Graph {
        Graph {
            start: start,
            edges: BTreeSet::new(),
            acceptors: BTreeSet::new(),
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
    // fn traverse_node<F: FnMut(&Node)>(&self,
    // f: &mut F,
    // current: &Node,
    // visited: &mut HashSet<Node>) {
    // if visited.contains(current) {
    // return;
    // }
    //
    // visited.insert(*current);
    //
    // f(current);
    //
    // for edge in self.edges.iter().filter(|edge| edge.from == *current) {
    // self.traverse_node(f, &edge.to, visited);
    // }
    // }
    //
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
        RegExpr::Sequence(ref v) => {
            let start = Node::new(alloc);
            let nfas: Vec<Graph>;
            {
                nfas = v.iter().map(|e| build_nfa(e, alloc)).collect();
            }
            let end = Node::new(alloc);

            let mut current_end: BTreeSet<Node> = [start].iter().cloned().collect();
            let mut ret = Graph::new(start);

            for nfa in nfas {
                ret.edges.extend(nfa.edges);
                for acceptor in current_end {
                    ret.add_edge(None, acceptor, nfa.start);
                }
                current_end = nfa.acceptors;
            }

            for acceptor in current_end {
                ret.add_edge(None, acceptor, end)
            }
            ret.acceptors.insert(end);
            ret
            // let mut car = build_nfa(&car, alloc);
            // let cdr = build_nfa(&cdr, alloc);
            // car.edges.extend(car.acceptors.iter().map(|acceptor| {
            // Edge {
            // condition: None,
            // from: acceptor.clone(),
            // to: cdr.start,
            // }
            // }));
            // Graph {
            // start: car.start,
            // edges: car.edges.union(&cdr.edges).cloned().collect(),
            // acceptors: cdr.acceptors,
            // }
            //
        }
        RegExpr::Branch(ref lhs, ref rhs) => {
            use std::iter::Iterator;
            let lhs = build_nfa(&lhs, alloc);
            let rhs = build_nfa(&rhs, alloc);
            let start = Node::new(alloc);
            let end = Node::new(alloc);
            println!("start -> {}", start.id);
            println!("lhs.start -> {}", lhs.start.id);
            println!("rhs.start -> {}", rhs.start.id);

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

fn reachable_through_epsilon(graph: &Graph, nodes: &BTreeSet<Node>) -> BTreeSet<Node> {
    let mut ret: BTreeSet<Node> = nodes.clone();
    for node in nodes {
        graph.traverse(&mut |edge| {
                           if edge.condition.is_none() && ret.contains(&edge.from) {
                               ret.insert(edge.to);
                           }
                       },
                       &node,
                       &mut HashSet::new());
    }
    ret
}

pub fn build_dfa(graph: &Graph) -> DFA {
    let mut target = DFANode::new(reachable_through_epsilon(graph,
                                                            &[graph.start]
                                                                 .iter()
                                                                 .cloned()
                                                                 .collect()),
                                  graph);
    let mut ret: DFA = DFA::new(target.clone());
    let mut dfa_nodes: BTreeSet<DFANode> = BTreeSet::new();
    dfa_nodes.insert(target.clone());
    let mut processed_nodes: BTreeSet<DFANode> = BTreeSet::new();
    loop {
        println!("target: {:?}", target);
        let mut successors: HashMap<char, BTreeSet<Node>> = HashMap::new();
        for &Edge { from: _, to, condition } in graph.edges.iter().filter(|edge| {
            target.nodes.contains(&edge.from)
        }) {
            match condition {
                Some(c) => {
                    successors.entry(c).or_insert(BTreeSet::new()).insert(to);
                }
                None => {}
            }
        }
        {
            successors = successors.iter()
                                   .map(|(c, nodes)| {
                                       (*c, reachable_through_epsilon(graph, &nodes))
                                   })
                                   .collect();
        }
        println!("succs: {:?}", successors);
        for (c, successor) in successors {
            let node = DFANode::new(successor, graph);
            if !processed_nodes.contains(&node) && !dfa_nodes.contains(&node) {
                dfa_nodes.insert(node.clone());
            }
            ret.edges.insert(DFAEdge {
                condition: c,
                from: target.clone(),
                to: node,
            });
        }
        dfa_nodes.remove(&target);
        processed_nodes.insert(target);
        println!("processed: {:?}", processed_nodes);
        println!("not processed: {:?}", dfa_nodes);
        match dfa_nodes.iter().next() {
            Some(node) => {
                target = node.clone();
            }
            None => {
                println!("no set remain");
                break;
            }
        }
    }
    ret
}
