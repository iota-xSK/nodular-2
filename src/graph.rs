use crate::app::App;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Graph {
    pub nodes_read: Vec<Option<u32>>,
    pub nodes_write: Vec<Option<u32>>,
    pub edges: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes_read: vec![],
            nodes_write: vec![],
            edges: vec![],
        }
    }

    pub fn from(nodes_read: &[u32], nodes_write: &[u32], edges: &[&[usize]]) -> Self {
        Self {
            nodes_write: nodes_write.iter().map(|a| Some(*a)).collect(),
            nodes_read: nodes_read.iter().map(|a| Some(*a)).collect(),
            edges: edges.iter().map(|a| Vec::from(*a)).collect(),
        }
    }
    pub fn copy(&self, selected: &[usize]) -> Self {
        let mut new = self.clone();
        for i in 0..self.nodes_read.len() {
            if !selected.contains(&i) {
                new.remove_node(i)
            }
        }
        new
    }
    pub fn paste(&self, app: &mut App) {
        let mut new_indexes = vec![None; self.nodes_read.len()];

        for i in 0..new_indexes.len() {
            if let Some(node) = self.nodes_write[i] {
                new_indexes[i] = Some(app.automaton.automaton.graph.add_node(node));
            }
        }
        for i in &new_indexes {
            if let Some(i) = i {
                let mut new = vec![];
                for j in &self.edges[*i] {
                    if let Some(new_edge) = new_indexes[*j] {
                        new.push(new_edge)
                    }
                }
                app.automaton.automaton.graph.edges[*i] = new;
            }
        }
    }
    pub fn add_node(&mut self, state: u32) -> usize {
        for i in 0..self.nodes_read.len() {
            if let None = self.nodes_write[i] {
                self.nodes_write[i] = Some(state);
                self.nodes_read[i] = Some(state);
                return i;
            }
        }
        self.nodes_write.push(Some(state));
        self.nodes_read.push(Some(state));
        self.edges.push(vec![]);
        self.nodes_write.len() - 1
    }
    pub fn remove_node(&mut self, idx: usize) {
        self.nodes_write[idx] = None;
        self.nodes_read[idx] = None;
        self.edges[idx] = vec![];

        for edge in self.edges.iter_mut() {
            edge.retain(|a| *a != idx)
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        if !self.edges[u].contains(&v) {
            self.edges[u].push(v)
        }
    }
    pub fn remove_edge(&mut self, u: usize, v: usize) {
        self.edges[u].retain(|a| *a != v);
    }
}
