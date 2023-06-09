use crate::{app::UiState, vec2::Vec2};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub read: u32,
    pub write: u32,
    pub edges: Vec<usize>,
    pub position: Vec2,
}

impl Node {
    pub fn new(read: u32, write: u32, edges: Vec<usize>, position: Vec2) -> Self {
        Self {
            read,
            write,
            edges,
            position,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

impl Graph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn copy(&self, selection: &[usize]) -> Self {
        // let mut selection = Vec::from(selection);

        // selection.sort();

        let mut indexes = vec![None; self.nodes.len()];
        let mut new_graph = Graph::new();

        for selected in selection {
            let mut new_node = self.nodes[*selected].clone();
            new_node.edges = vec![];
            new_graph.add_node(new_node);
            indexes[*selected] = Some(new_graph.nodes.len() - 1)
        }
        println!("{:?}", indexes);
        for (i, &selected) in selection.iter().enumerate() {
            for j in &self.nodes[selected].edges {
                if let Some(new_index) = indexes[*j] {
                    println!("{i}");
                    new_graph.add_edge(i, new_index);
                }
            }
        }
        new_graph
    }

    // pub fn from(nodes_read: &[u32], nodes_write: &[u32], edges: &[&[usize]]) -> Self {
    //     Self {
    //         nodes_write: nodes_write.iter().map(|a| Some(*a)).collect(),
    //         nodes_read: nodes_read.iter().map(|a| Some(*a)).collect(),
    //         edges: edges.iter().map(|a| Vec::from(*a)).collect(),
    //     }
    // }
    // pub fn copy(&self, selected: &[usize]) -> Self {
    //     let mut new = self.clone();
    //     for i in 0..self.nodes_read.len() {
    //         if !selected.contains(&i) {
    //             new.remove_node(i)
    //         }
    //     }
    //     new
    // }
    // pub fn paste(&self, app: &mut App) {
    //     let mut new_indexes = vec![None; self.nodes_read.len()];
    //
    //     for i in 0..new_indexes.len() {
    //         if let Some(node) = self.nodes_write[i] {
    //             new_indexes[i] = Some(app.automaton.automaton.graph.add_node(node));
    //         }
    //     }
    //     for i in &new_indexes {
    //         if let Some(i) = i {
    //             let mut new = vec![];
    //             for j in &self.edges[*i] {
    //                 if let Some(new_edge) = new_indexes[*j] {
    //                     new.push(new_edge)
    //                 }
    //             }
    //             app.automaton.automaton.graph.edges[*i] = new;
    //         }
    //     }
    // }
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node)
    }
    pub fn remove_node_from_app(&mut self, idx: usize, ui: &mut UiState) {
        self.nodes.swap_remove(idx);
        let len = self.nodes.len();
        for node in self.nodes.iter_mut() {
            node.edges.retain(|a| *a != idx);
            node.edges = node
                .edges
                .iter()
                .map(|a| if *a == len { idx } else { *a })
                .collect();
        }
        ui.selected = ui
            .selected
            .iter()
            .map(|a| if *a == len { idx } else { *a })
            .collect();
    }
    pub fn remove_node(&mut self, idx: usize) {
        self.nodes.swap_remove(idx);
        let len = self.nodes.len();
        for node in self.nodes.iter_mut() {
            node.edges.retain(|a| *a != idx);
            node.edges = node
                .edges
                .iter()
                .map(|a| if *a == len { idx } else { *a })
                .collect();
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) -> bool {
        if !self.nodes[u].edges.contains(&v) {
            self.nodes[u].edges.push(v);
            true
        } else {
            false
        }
    }

    pub fn remove_edge(&mut self, u: usize, v: usize) {
        self.nodes[u].edges.retain(|a| *a != v);
    }
}
