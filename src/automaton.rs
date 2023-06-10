use crate::{
    app::App,
    graph::{Graph, Node},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Automaton {
    pub rules: Ruleset,
    pub graph: Graph,
}

impl Automaton {
    pub fn new(rules: Ruleset, graph: Graph) -> Self {
        Self { rules, graph }
    }
    pub fn step(&mut self) {
        for node in self.graph.nodes.iter_mut() {
            std::mem::swap(&mut node.read, &mut node.write);
        }

        for node in 0..self.graph.nodes.len() {
            self.rules.apply(node, &mut self.graph).unwrap();
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Rule {
    pattern: Pattern,
    replacement: u32,
}

impl Rule {
    pub fn new(pattern: Pattern, replacement: u32) -> Self {
        Self {
            pattern,
            replacement,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum Pattern {
    Equal { state: u32, number: u32 },
    Gth { state: u32, number: u32 },
    Lth { state: u32, number: u32 },
    Geq { state: u32, number: u32 },
    Leq { state: u32, number: u32 },
    Or(Box<Pattern>, Box<Pattern>),
    And(Box<Pattern>, Box<Pattern>),
    Not(Box<Pattern>),
    Wildcard,
}

impl Pattern {
    pub fn pattern_match(&self, node: usize, graph: &Graph) -> bool {
        match self {
            Pattern::Equal { state, number } => {
                graph.nodes[node]
                    .edges
                    .iter()
                    .filter(|a| graph.nodes[**a].read == *state)
                    .count() as u32
                    == *number
            }
            Pattern::Gth { state, number } => {
                graph.nodes[node]
                    .edges
                    .iter()
                    .filter(|a| graph.nodes[**a].read == *state)
                    .count() as u32
                    > *number
            }
            Pattern::Lth { state, number } => {
                (graph.nodes[node]
                    .edges
                    .iter()
                    .filter(|a| graph.nodes[**a].read == *state)
                    .count() as u32)
                    < *number
            }
            Pattern::Geq { state, number } => {
                (graph.nodes[node]
                    .edges
                    .iter()
                    .filter(|a| graph.nodes[**a].read == *state)
                    .count() as u32)
                    >= *number
            }
            Pattern::Leq { state, number } => {
                (graph.nodes[node]
                    .edges
                    .iter()
                    .filter(|a| graph.nodes[**a].read == *state)
                    .count() as u32)
                    <= *number
            }
            Pattern::Or(a, b) => a.pattern_match(node, graph) || b.pattern_match(node, graph),
            Pattern::And(a, b) => a.pattern_match(node, graph) && b.pattern_match(node, graph),
            Pattern::Not(a) => !a.pattern_match(node, graph),
            Pattern::Wildcard => true,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Ruleset {
    pub names: Vec<String>,
    pub rules: Vec<Vec<Rule>>,
}

impl Ruleset {
    pub fn new(rules: Vec<Vec<Rule>>, names: Vec<String>) -> Option<Self> {
        for state in &rules {
            if !state.iter().any(|a| a.pattern == Pattern::Wildcard) {
                return None;
            }
        }
        Some(Ruleset { rules, names })
    }

    pub fn apply(&self, idx: usize, graph: &mut Graph) -> Option<()> {
        for rule in &self.rules[graph.nodes[idx].read as usize] {
            if rule.pattern.pattern_match(idx, graph) {
                graph.nodes[idx].write = rule.replacement;
                return Some(());
            }
        }
        None
    }
}
