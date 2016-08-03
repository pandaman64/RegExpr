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
    acceptors: HashSet<Node>,
}

struct PowerSet{
    all_nodes: BTreeSet<Node>,
    current: Vec<Node>
}

fn power_set(nodes: BTreeSet<Node>) -> PowerSet{
    PowerSet{ all_nodes: nodes, current: vec![] }
}

impl PowerSet{
    fn min_node(&self) -> Node{
        *self.all_nodes.iter().next().unwrap()
    }

    fn max_node(&self) -> Node{
        *self.all_nodes.iter().last().unwrap()
    }

    fn next_node(&self,node: &Node) -> Node{
        *self.all_nodes.iter().skip_while(|n| *n <= node).next().unwrap()
    }
}

impl Iterator for PowerSet{
    type Item = HashSet<Node>;

    fn next(&mut self) -> Option<HashSet<Node>>{
       if self.current.is_empty(){
           self.current = vec![self.min_node()];
           return Some(self.current.iter().cloned().collect());
       }

       let mut next = self.current.clone();

       for (idx,node) in self.current.iter().enumerate(){
            if *node != self.max_node(){
                next[idx] = self.next_node(&node);
                break;
            }
            else{
                next[idx] = self.min_node();
                if next.len() < self.all_nodes.len(){
                    next.push(self.min_node());
                    break;
                }
                else{
                    return None;
                }
            }
       }

       self.current = next;

       Some(self.current.iter().cloned().collect())
    }
}

pub struct DFAAllocator{
    nodes: HashMap<DFANode,HashSet<Node>>,
//    edges: HashMap<usize,HashSet<DFAEdge>>
}

impl DFAAllocator{
    pub fn new() -> DFAAllocator{
        DFAAllocator{ nodes: HashMap::new() }
    }

    fn new_node(&mut self,nodes: HashSet<Node>) -> DFANode{
        if let Some((node,_)) = self.nodes.iter().find(|&(_,v)| *v == nodes){
            return *node;
        }
        let len = self.nodes.len();
        let node = DFANode{ id: len };
        self.nodes.insert(node,nodes);
        node
    }

    fn pretty_name(&self, node: &DFANode) -> String{
        format!("\"{{ {} }}\"", self.nodes[node].iter().map(|node| format!("{}",node.id)).collect::<Vec<_>>().join(","))
    }
}

#[derive(Debug,PartialEq,Eq,Hash,Clone,Copy)]
pub struct DFANode{
    id: usize
}

#[derive(Debug,Hash,PartialEq,Eq)]
pub struct DFAEdge{
    condition: char,
    from: DFANode,
    to: DFANode
}

#[derive(Debug)]
pub struct DFA{
    start: DFANode,
    edges: HashSet<DFAEdge>,
    acceptors: HashSet<DFANode>
}

impl DFA{
    pub fn dotty_print<W: Write + ?Sized>(&self, alloc: &DFAAllocator, writer: &mut W){
        writeln!(writer, "digraph g{{").unwrap();

        for acceptor in &self.acceptors {
            writeln!(writer, "{} [ style = \"bold\" ];", alloc.pretty_name(acceptor)).unwrap();
        }

        for edge in &self.edges{
            writeln!(writer,"\t{} -> {} [ label = \"{}\" ];",alloc.pretty_name(&edge.from),alloc.pretty_name(&edge.to),edge.condition).unwrap();
        }
        
        writeln!(writer, "}}").unwrap();
    }
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

    fn traverse_node<F: FnMut(&Node)>(&self,
                                      f: &mut F,
                                      current: &Node,
                                      visited: &mut HashSet<Node>) {
        if visited.contains(current) {
            return;
        }

        visited.insert(*current);

        f(current);

        for edge in self.edges.iter().filter(|edge| edge.from == *current) {
            self.traverse_node(f, &edge.to, visited);
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
        RegExpr::Sequence(ref v) => {
            let start = Node::new(alloc);
            let nfas: Vec<Graph>;
            {
                nfas = v.iter().map(|e| build_nfa(e, alloc)).collect();
            }
            let end = Node::new(alloc);

            let mut current_end: HashSet<Node> = [start].iter().cloned().collect();
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

fn reachable_through_epsilon(through_epsilon: &HashMap<Node, HashSet<Node>>,
                             current: &Node,
                             visited: &mut HashSet<Node>)
                             -> HashSet<Node> {
    if visited.contains(current) {
        return HashSet::new();
    }

    visited.insert(*current);

    if let Some(nodes) = through_epsilon.get(current) {
        let mut accumulator: HashSet<Node> = nodes.clone();
        for to in nodes.iter() {
            let extension = reachable_through_epsilon(through_epsilon, to, visited);
            accumulator.extend(extension);
        }

        accumulator
    } else {
        HashSet::new()
    }
}

pub fn merge_by_epsilon(graph: &Graph, _: &mut NodeAllocator) -> Graph {
    let mut successors_through_epsilon: HashMap<Node, HashSet<Node>> = HashMap::new();
    let mut successors_through_non_epsilon: HashMap<Node, HashSet<(char, Node)>> = HashMap::new();

    let mut ret = Graph::new(graph.start);

    for edge in &graph.edges {
        match edge.condition {
            None => {
                successors_through_epsilon.entry(edge.from)
                                          .or_insert(HashSet::new())
                                          .insert(edge.to);
            }
            Some(c) => {
                successors_through_non_epsilon.entry(edge.from)
                                              .or_insert(HashSet::new())
                                              .insert((c, edge.to));
                ret.edges.insert(edge.clone());
            }
        }
    }

    graph.traverse_node(&mut |node| {
                            let through_epsilon =
                                reachable_through_epsilon(&successors_through_epsilon,
                                                          node,
                                                          &mut HashSet::new());
                            println!("{} -> {:?}", node.id, through_epsilon);

                            for another_start in &through_epsilon {
                                if let Some(edges) =
                                       successors_through_non_epsilon.get(another_start) {
                                    for &(c, to) in edges {
                                        ret.edges.insert(Edge {
                                            condition: Some(c),
                                            from: *node,
                                            to: to,
                                        });
                                    }
                                }
                                if graph.acceptors.contains(another_start){
                                    ret.acceptors.insert(*node);
                                }
                            }

                            if graph.acceptors.contains(node){
                                ret.acceptors.insert(*node);
                                ret.acceptors.extend(through_epsilon);
                            }
                        },
                        &graph.start,
                        &mut HashSet::new());
    ret
}


fn succeeding_subsets(start: HashSet<Node>,edges: &BTreeSet<Edge>,all_nodes: BTreeSet<Node>,alloc: &mut DFAAllocator) -> HashMap<char,HashSet<DFANode>>{
    let mut subsets: HashMap<char,HashSet<DFANode>> = HashMap::new();

    for edge in edges{
        if start.contains(&edge.from){
            subsets.entry(edge.condition.unwrap()).or_insert(HashSet::new()).extend(power_set(all_nodes.clone()).filter(|nodes| nodes.contains(&edge.to)).map(|nodes| alloc.new_node(nodes)));
        }
    }
   
    subsets
}

pub fn build_dfa(graph: &Graph, alloc: &mut DFAAllocator) -> DFA{
    let all_nodes: HashSet<Node>;
    {
        let mut all_nodes_mut = HashSet::new();
        for edge in &graph.edges{
            all_nodes_mut.insert(edge.from);
            all_nodes_mut.insert(edge.to);
        }
        all_nodes = all_nodes_mut;
    }

    let start = alloc.new_node([graph.start].iter().cloned().collect());
    let mut visited: HashSet<DFANode> = HashSet::new();
    let mut queue: HashSet<DFANode> = [start].iter().cloned().collect();

    let mut dfa = DFA{ start: start, edges: HashSet::new(), acceptors: HashSet::new() };

    while !queue.is_empty(){
        let successor_map = succeeding_subsets(alloc.nodes[&start].clone(),&graph.edges,all_nodes.iter().cloned().collect(),alloc);
        let mut new_queue = HashSet::new();

        for (c,successors) in &successor_map{
            for successor in successors{
                dfa.edges.insert(DFAEdge{ condition:*c, from: start, to:*successor });
                if !visited.contains(&successor){
                    new_queue.insert(*successor);
                    visited.insert(*successor);
                }
            }
        }
    
        queue = new_queue;
    }

    //TODO: add acceptors

    dfa
}
