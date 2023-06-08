mod app;
mod automaton;
mod clipboard;
mod graph;
mod vec2;
use automaton::*;
use raylib::prelude::{Camera2D, Vector2};

use crate::app::*;
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

    let mut automaton = Automaton::new(
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
