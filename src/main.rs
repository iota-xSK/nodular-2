mod app;
mod automaton;
mod graph;
mod vec2;
use automaton::*;
use raylib::prelude::{Camera2D, Vector2};

use crate::app::*;
use crate::graph::Graph;
use crate::vec2::Vec2;
fn main() -> Result<(), ()> {
    println!("Hello, world!");

    let pattern = Pattern::Wildcard;

    let mut graph = Graph::new();

    graph.add_node(0);
    graph.add_node(1);
    graph.add_node(2);

    graph.add_edge(0, 1);
    graph.add_edge(1, 2);

    let mut automaton = Automaton::new(
        Ruleset::new(
            vec![vec![Rule::new(pattern, 0)]],
            vec!["hello world".to_string()],
        )
        .unwrap(),
        graph,
    );

    let mut app = App::new(
        VisualAutomaton::new(
            automaton,
            vec![
                Vec2::new(500.0, 120.0),
                Vec2::new(110.0, 160.0),
                Vec2::new(0.0, 10.0),
            ],
        ),
        Camera2D {
            offset: Vector2::new(0.0, 0.0),
            target: Vector2::new(0.0, 0.0),
            rotation: 0.0,
            zoom: 1.0,
        },
    );

    app.run();

    Ok(())
}
