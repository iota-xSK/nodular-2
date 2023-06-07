mod automaton;
mod graph;
mod vec2;
use automaton::*;

use crate::graph::Graph;
fn main() -> Result<(), ()> {
    println!("Hello, world!");

    let pattern = Pattern::Wildcard;

    let mut graph = Graph::new();

    graph.add_node(0);
    graph.add_node(0);
    graph.add_node(0);
    graph.add_node(0);

    graph.add_edge(0, 1);
    graph.add_edge(1, 2);

    let mut automaton = Automaton::new(
        Ruleset::new(vec![vec![Rule::new(pattern, 0)]]).unwrap(),
        graph,
    );

    Ok(())
}
