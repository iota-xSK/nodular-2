// mod app;
// mod automaton;
// mod clipboard;
mod app;
mod automaton;
mod graph;
mod vec2;
use graph::Node;
// use automaton::*;
use raylib::prelude::{Camera2D, Vector2};

use crate::app::App;
use crate::automaton::{Automaton, Pattern, Rule, Ruleset};
// use crate::app::*;
use crate::graph::Graph;
use crate::vec2::Vec2;
fn main() -> Result<(), ()> {
    let wildcard = Pattern::Wildcard;
    let turn_on = Pattern::Or(
        Box::new(Pattern::Equal {
            state: 0,
            number: 1,
        }),
        Box::new(Pattern::Equal {
            state: 0,
            number: 2,
        }),
    );
    let mut graph = Graph::new();

    let automaton = Automaton::new(
        Ruleset::new(
            vec![
                vec![
                    Rule::new(turn_on.clone(), 0),
                    Rule::new(wildcard.clone(), 1),
                ],
                vec![Rule::new(turn_on.clone(), 0), Rule::new(wildcard, 1)],
            ],
            vec!["electron".to_string(), "wire".to_string()],
        )
        .unwrap(),
        graph,
    );

    let mut app = App::new(
        automaton,
        Camera2D {
            offset: Vector2::zero(),
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: 1.0,
        },
    );

    app.run();

    let mut graph = Graph::new();

    graph.add_node(Node::new(0, 0, vec![], Vec2::new(0.0, 0.0)));
    graph.add_node(Node::new(1, 0, vec![], Vec2::new(0.0, 0.0)));
    graph.add_node(Node::new(1, 0, vec![], Vec2::new(0.0, 0.0)));
    graph.add_node(Node::new(1, 0, vec![], Vec2::new(0.0, 0.0)));
    graph.add_node(Node::new(1, 0, vec![], Vec2::new(0.0, 0.0)));

    graph.add_edge(0, 1);
    graph.add_edge(0, 2);
    graph.add_edge(0, 3);
    graph.add_edge(0, 4);

    println!("{:?}", graph);
    println!("{:?}", graph);

    Ok(())
}
