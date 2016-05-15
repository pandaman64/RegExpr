mod parser;
use parser::parse;

mod automaton;
use automaton::NodeAllocator;
use automaton::build_nfa;
use automaton::merge_by_epsilon;

fn main() {
    use std::fs::File;
    let input = "(a|bc)*d".to_owned();
    let expression = parse(&mut input.chars());

    let mut alloc = NodeAllocator::new();
    let nfa = build_nfa(&expression.unwrap(), &mut alloc);
    nfa.dotty_print(&mut File::create("nfa.dot").unwrap());
    // nfa.dotty_print(&mut std::io::stdout());

    let merged = merge_by_epsilon(&nfa, &mut alloc);
    merged.dotty_print(&mut File::create("dfa.dot").unwrap());
}
