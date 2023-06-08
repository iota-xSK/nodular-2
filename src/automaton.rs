use crate::graph::Graph;

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
        std::mem::swap(&mut self.graph.nodes_read, &mut self.graph.nodes_write);

        for node in 0..self.graph.nodes_write.len() {
            self.rules.apply(&mut self.graph, node);
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
    fn pattern_match(&self, node: usize, graph: &Graph) -> Option<bool> {
        let nbh = graph.edges[node]
            .iter()
            .map(|a| graph.nodes_write[*a])
            .collect::<Option<Vec<u32>>>();
        if let Some(nbh) = nbh {
            match self {
                Pattern::Equal { state, number } => {
                    if nbh.iter().filter(|a| a == &state).count() as u32 == *number {
                        return Some(true);
                    } else {
                        return Some(false);
                    }
                }
                Pattern::Gth { state, number } => {
                    if nbh.iter().filter(|a| a == &state).count() as u32 > *number {
                        return Some(true);
                    } else {
                        return Some(false);
                    }
                }
                Pattern::Lth { state, number } => {
                    if (nbh.iter().filter(|a| a == &state).count() as u32) < *number {
                        return Some(true);
                    } else {
                        return Some(false);
                    }
                }
                Pattern::Geq { state, number } => {
                    if nbh.iter().filter(|a| a == &state).count() as u32 >= *number {
                        return Some(true);
                    } else {
                        return Some(false);
                    }
                }
                Pattern::Leq { state, number } => {
                    if nbh.iter().filter(|a| a == &state).count() as u32 <= *number {
                        return Some(true);
                    } else {
                        return Some(false);
                    }
                }
                Pattern::Or(left, right) => {
                    return Some(
                        left.pattern_match(node, graph)? || right.pattern_match(node, graph)?,
                    )
                }
                Pattern::And(left, right) => {
                    return Some(
                        left.pattern_match(node, graph)? && right.pattern_match(node, graph)?,
                    )
                }
                Pattern::Not(u) => return Some(!u.pattern_match(node, graph)?),
                Pattern::Wildcard => return Some(true),
            }
        };

        unreachable!()
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

    pub fn apply(&self, graph: &mut Graph, idx: usize) -> Option<()> {
        for rule in self.rules[graph.nodes_read[idx]? as usize].iter() {
            if rule.pattern.pattern_match(idx, graph)? {
                graph.nodes_write[idx] = Some(rule.replacement)
            }
        }
        None
    }
}
