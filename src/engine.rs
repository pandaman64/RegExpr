use automaton::DFA;
use automaton::DFANode;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub struct Engine{
    start: DFANode,
    edges: BTreeMap<DFANode,HashMap<char,DFANode>>
}

impl Engine{
    pub fn new(dfa: DFA) -> Engine{
        let mut edges: BTreeMap<DFANode,HashMap<char,DFANode>> = BTreeMap::new();
        for edge in dfa.edges.into_iter(){
            assert!(edges.entry(edge.from).or_insert(HashMap::new()).insert(edge.condition,edge.to).is_none());
        }

        Engine{ start: dfa.start, edges: edges }
    }

    pub fn match_string(&self,s: &str) -> bool{
        let mut current = self.start.clone();
        let mut iter = s.chars();
        loop{
            match iter.next(){
                None => { return current.is_acceptor; },
                Some(c) => {
                   match self.edges.get(&current){
                       None => { return false; },
                       Some(edge) => {
                           match edge.get(&c){
                               None => { return false; },
                               Some(to) => { current = to.clone(); }
                           }
                       }
                   }
                }
            }
        }
    }
}

