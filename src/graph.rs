#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Graph {
    pub nodes_read: Vec<Option<u32>>,
    pub nodes_write: Vec<Option<u32>>,
    pub edges: Vec<Option<Vec<usize>>>,
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
            edges: edges.iter().map(|a| Some(Vec::from(*a))).collect(),
        }
    }
    pub fn add_node(&mut self, state: u32) {
        for i in 0..self.nodes_read.len() {
            if let None = self.nodes_write[i] {
                self.nodes_write[i] = Some(state);
                self.nodes_read[i] = Some(state);
                return;
            }
        }
        self.nodes_write.push(Some(state))
    }
    pub fn remove_node(&mut self, idx: usize) {
        self.nodes_write[idx] = None;
    }
}
