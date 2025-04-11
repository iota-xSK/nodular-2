// mod app;
// mod automaton;
// mod clipboard;
mod app;
mod automaton;
mod graph;
mod vec2;
use graph::Node;
use midir::MidiOutput;
use raylib::prelude::{Camera2D, Vector2};

use crate::app::App;
use crate::automaton::{Automaton, Pattern, Rule, Ruleset};
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
    let turn_off = Pattern::Or(
        Box::new(Pattern::Equal {
            state: 0,
            number: 2,
        }),
        Box::new(Pattern::Equal {
            state: 0,
            number: 3,
        }),
    );
    let graph = Graph::new();

    let automaton = Automaton::new(
        Ruleset::new(
            vec![
                vec![
                    Rule::new(turn_on, 1),
                    Rule::new(wildcard.clone(), 1),
                ],
                vec![Rule::new(turn_off, 0), Rule::new(wildcard, 1)],
            ],
            vec!["electron".to_string(), "wire".to_string()],
        )
        .unwrap(),
        graph,
    );

    let mut app = App::new(automaton);

    app.run();

    Ok(())
}
