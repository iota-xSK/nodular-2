use crate::{app::App, graph::Graph, vec2::Vec2};

#[derive(Clone, Debug)]
pub struct Clipboard {
    pub nodes_read: Vec<Option<u32>>,
    pub nodes_write: Vec<Option<u32>>,
    pub edges: Vec<Vec<usize>>,
    pub positions: Vec<Vec2>,
}

impl Clipboard {
    pub fn copy(graph: &Graph, selected: &[usize], positions: &[Vec2]) -> Self {
        let mut new = graph.clone();
        for i in 0..graph.nodes_read.len() {
            if !selected.contains(&i) {
                new.remove_node(i)
            }
        }

        Clipboard {
            nodes_read: new.nodes_read,
            nodes_write: new.nodes_write,
            edges: new.edges,
            positions: positions.iter().map(|a| *a).collect(),
        }
    }

    pub fn paste(&self, app: &mut App) {
        let mut new_indexes = vec![None; self.nodes_read.len()];

        for i in 0..new_indexes.len() {
            if let Some(node) = self.nodes_write[i] {
                new_indexes[i] = Some(app.automaton.automaton.graph.add_node(node));
                if new_indexes.len() <= app.automaton.node_possions.len() {
                    app.automaton.node_possions.push(Vec2::zero())
                }
            }
        }
        app.ui_state.selected = new_indexes.iter().filter_map(|a| *a).collect();
        for (i, new_index) in new_indexes.iter().enumerate() {
            if let Some(new_index) = new_index {
                let mut new = vec![];
                for j in &self.edges[i] {
                    if let Some(new_edge) = new_indexes[*j] {
                        new.push(new_edge)
                    }
                }
                app.automaton.automaton.graph.edges[*new_index] = new;
                app.automaton.node_possions[*new_index] = self.positions[i] + Vec2::new(50.0, 50.0);
            }
        }
    }
}
