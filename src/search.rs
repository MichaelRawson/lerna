use std::collections::{HashMap, HashSet, VecDeque};
use unique::Id;

use crate::collections::IdSet;
use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::score::Score;
use crate::status::Status;

#[derive(Debug, Default)]
struct Node {
    parents: Vec<Id<Formula>>,
    children: Option<Vec<IdSet<Formula>>>,
    score: Score,
    status: Status,
    visits: usize,
}

impl Node {
    fn add_parent(&mut self, parent: Id<Formula>) {
        self.parents.push(parent);
    }

    fn increment_visits(&mut self) {
        self.visits += 1;
    }

    fn uct(parent: &Self, child: &Self) -> Score {
        let parent_visits = parent.visits as f32;
        let visits = child.visits as f32;
        let skepticism = OPTIONS.skepticism;
        let score =
            child.score.0 + (skepticism * parent_visits.ln() / visits).sqrt();
        Score::new(score)
    }
}

pub struct Search {
    root: Id<Formula>,
    nodes: HashMap<Id<Formula>, Node>,
}

impl Search {
    pub fn new(root: Id<Formula>) -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(root.clone(), Node::default());

        Self { nodes, root }
    }

    fn parents_of(&self, leaf: Id<Formula>) -> Vec<Id<Formula>> {
        let mut done = HashSet::new();
        let mut todo = VecDeque::new();
        let mut parents = vec![];
        done.insert(leaf.clone());
        todo.push_back(leaf);

        while let Some(next) = todo.pop_front() {
            if !done.contains(&next) {
                done.insert(next.clone());
                todo.extend(self.nodes[&next].parents.iter().cloned());
                parents.push(next);
            }
        }

        parents
    }

    pub fn status(&self) -> Status {
        self.nodes[&self.root].status
    }

    pub fn set_status(&mut self, f: Id<Formula>, status: Status) {
        self.nodes.get_mut(&f).unwrap().status = status;
        self.propagate_status(f);
    }

    pub fn set_score(&mut self, f: Id<Formula>, score: Score) {
        self.nodes.get_mut(&f).unwrap().score = score;
        self.propagate_score(f)
    }

    fn propagate_status(&mut self, start: Id<Formula>) {
        for f in self.parents_of(start) {
            let status = self.nodes[&f]
                .children
                .as_ref()
                .unwrap()
                .iter()
                .map(|fs| {
                    fs.into_iter().map(|f| self.nodes[f].status).product()
                })
                .sum();
            self.nodes.get_mut(&f).unwrap().status = status;
        }
    }

    fn propagate_score(&mut self, start: Id<Formula>) {
        use Status::*;
        for f in self.parents_of(start) {
            let score = self.nodes[&f]
                .children
                .as_ref()
                .unwrap()
                .iter()
                .map(|fs| {
                    Score::new(
                        fs.into_iter()
                            .map(|f| {
                                let score = self.nodes[f].score.0;
                                let status = self.nodes[f].status;
                                match status {
                                    Sat => 0.0,
                                    Unsat => 1.0,
                                    Unknown => score,
                                }
                            })
                            .sum::<f32>()
                            / fs.len() as f32,
                    )
                })
                .max()
                .unwrap();
            self.nodes.get_mut(&f).unwrap().score = score;
        }
    }
}
