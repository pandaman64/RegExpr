mod parser;
use parser::parse;

mod automaton;
use automaton::NodeAllocator;
use automaton::build_nfa;
use automaton::build_dfa;

fn main() {
    use std::fs::File;
    let input = "(a|b)*cde".to_owned();
    let expression = parse(&mut input.chars());

    println!("{:?}", expression);

    let mut alloc = NodeAllocator::new();
    let nfa = build_nfa(&expression.unwrap(), &mut alloc);
    nfa.dotty_print(&mut File::create("nfa.dot").unwrap());
    // nfa.dotty_print(&mut std::io::stdout());

    let dfa = build_dfa(&nfa);
    dfa.dotty_print(&mut File::create("dfa.dot").unwrap());
}
