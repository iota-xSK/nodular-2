use crate::graph::Graph;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Automaton {
    rules: Ruleset,
    graph: Graph,
}

impl Automaton {
    fn step(&mut self) {}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Rule {
    pattern: Pattern,
    replacement: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum Pattern {
    Equal { state: State, number: u32 },
    Gth { state: State, number: u32 },
    Lth { state: State, number: u32 },
    Geq { state: State, number: u32 },
    Leq { state: State, number: u32 },
    Or(Box<Pattern>, Box<Pattern>),
    And(Box<Pattern>, Box<Pattern>),
    Not(Box<Pattern>),
    Wildcard,
}

impl Pattern {
    fn pattern_match(&self, node: usize, graph: &Graph) -> bool {
        let nbh = graph.nbh[node].iter().map(|i| graph.nodes_read[*i]);

        match self {
            Pattern::Equal { state, number } => {
                if nbh.filter(|a| a == state).count() as u32 == *number {
                    return true;
                } else {
                    return false;
                }
            }
            Pattern::Gth { state, number } => {
                if nbh.filter(|a| a == state).count() as u32 > *number {
                    return true;
                } else {
                    return false;
                }
            }
            Pattern::Lth { state, number } => {
                if (nbh.filter(|a| a == state).count() as u32) < *number {
                    return true;
                } else {
                    return false;
                }
            }
            Pattern::Geq { state, number } => {
                if nbh.filter(|a| a == state).count() as u32 >= *number {
                    return true;
                } else {
                    return false;
                }
            }
            Pattern::Leq { state, number } => {
                if nbh.filter(|a| a == state).count() as u32 <= *number {
                    return true;
                } else {
                    return false;
                }
            }
            Pattern::Or(left, right) => {
                return left.pattern_match(node, graph) || right.pattern_match(node, graph)
            }
            Pattern::And(left, right) => {
                return left.pattern_match(node, graph) && right.pattern_match(node, graph)
            }
            Pattern::Not(u) => return !u.pattern_match(node, graph),
            Pattern::Wildcard => return true,
        }
    }
}

struct Ruleset {
    rules: Vec<Vec<Rule>>,
}

impl Ruleset {
    fn new(rules: Vec<Vec<Rule>>) -> Option<Self> {
        for state in self.rules {
            if !state.iter().any(|a| a.pattern == Pattern::Wildcard) {
                return None;
            }
        }
        Some(Ruleset { rules })
    }

    fn apply(&self, graph: &mut graph, idx: usize) -> Option<u32> {
        for rule in self.rules[graph.nodes_read[idx]?].iter() {
            if rule.pattern_match(idx, graph) {
                graph.node_write[idx] = Some(rule.replacement)
            }
        }
    }
}
