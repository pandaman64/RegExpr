mod parser;
use parser::parse;

mod automaton;
use automaton::NodeAllocator;
use automaton::DFAAllocator;
use automaton::build_nfa;
use automaton::merge_by_epsilon;
use automaton::build_dfa;

fn main() {
    use std::fs::File;
    let input = "a".to_owned();
    let expression = parse(&mut input.chars());

    println!("{:?}", expression);

    let mut alloc = NodeAllocator::new();
    let nfa = build_nfa(&expression.unwrap(), &mut alloc);
    nfa.dotty_print(&mut File::create("nfa.dot").unwrap());
    // nfa.dotty_print(&mut std::io::stdout());

    let merged = merge_by_epsilon(&nfa, &mut alloc);
    merged.dotty_print(&mut File::create("no_epsilon.dot").unwrap());

    let mut alloc = DFAAllocator::new();
    let dfa = build_dfa(&merged,&mut alloc);
    dfa.dotty_print(&alloc,&mut File::create("dfa.dot").unwrap());
}
