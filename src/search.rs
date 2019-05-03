use std::collections::{HashMap, HashSet, VecDeque};
use unique::Id;

use crate::collections::IdSet;
use crate::deduction::deductions;
use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::score::Score;
use crate::status::Status;
use crate::record::record;

#[derive(Debug, Default)]
pub struct Node {
    parents: Vec<Id<Formula>>,
    children: Option<Vec<IdSet<Formula>>>,
    score: Score,
    status: Status,
    visits: usize,
}

fn uct(parent_visits: usize, child_visits: usize, score: Score) -> Score {
    let parent_visits = parent_visits as f32;
    let child_visits = (child_visits + 1) as f32;
    let skepticism = OPTIONS.skepticism;
    (score.0 + (skepticism * parent_visits.ln() / child_visits).sqrt()).into()
}

pub struct Search {
    root: Id<Formula>,
    nodes: HashMap<Id<Formula>, Node>,
}

impl Search {
    pub fn new(f: Id<Formula>) -> Self {
        let root = f.clone();
        let nodes = HashMap::new();
        let mut new = Self { nodes, root };
        new.insert_node(f);
        new
    }

    pub fn status(&self) -> Status {
        self.node(&self.root).status
    }

    pub fn set_status(&mut self, f: &Id<Formula>, new_status: Status) {
        let old_status = self.node_status(f);
        if !old_status.is_known() && old_status != new_status {
            record(f, new_status);
            self.node_mut(f).status = new_status;
            self.propagate_status(f);
        }
    }

    pub fn set_score(&mut self, f: &Id<Formula>, score: Score) {
        if !self.node_status(f).is_known() {
            self.node_mut(f).score = score;
            self.propagate_score(f)
        }
    }

    pub fn do_step(&mut self) -> Option<HashSet<Id<Formula>>> {
        assert!(!self.status().is_known());

        let leaf = self.select()?;

        assert!(!self.node_status(&leaf).is_known());

        let mut ancestors: HashSet<_> =
            self.ancestors_of(&leaf).into_iter().collect();
        ancestors.insert(leaf.clone());
        let new_formulae = self.expand(&leaf, &ancestors);

        if self.node_status(&leaf).is_known() {
            self.propagate_status(&leaf);
        }

        Some(new_formulae)
    }

    pub fn proof(&self) -> Vec<Id<Formula>> {
        assert_eq!(self.status(), Status::Unsat);
        let mut log = vec![];
        self.proof_of(&self.root, &mut log);
        log
    }

    fn node(&self, f: &Id<Formula>) -> &Node {
        self.nodes.get(f).expect("node did not exist")
    }

    fn node_mut(&mut self, f: &Id<Formula>) -> &mut Node {
        self.nodes.get_mut(f).expect("node did not exist")
    }

    fn node_children(&self, f: &Id<Formula>) -> &Vec<IdSet<Formula>> {
        self.node(f)
            .children
            .as_ref()
            .expect("node had no children")
    }

    fn node_parents(&self, f: &Id<Formula>) -> &Vec<Id<Formula>> {
        &self.node(f).parents
    }

    fn node_score(&self, f: &Id<Formula>) -> Score {
        self.node(f).score
    }

    fn node_status(&self, f: &Id<Formula>) -> Status {
        self.node(f).status
    }

    fn node_visits(&self, f: &Id<Formula>) -> usize {
        self.node(f).visits
    }

    fn insert_node(&mut self, f: Id<Formula>) {
        self.nodes.insert(f, Node::default());
    }

    fn ancestors_of(&self, leaf: &Id<Formula>) -> Vec<Id<Formula>> {
        let mut done = HashSet::new();
        let mut todo = VecDeque::new();
        let mut ancestors = vec![];

        done.insert(leaf);
        todo.extend(self.node_parents(&leaf).iter());

        while let Some(next) = todo.pop_front() {
            if !done.contains(&next) {
                done.insert(next);
                todo.extend(self.node_parents(&next).iter());
                ancestors.push(next.clone());
            }
        }

        ancestors
    }

    fn proof_of(&self, f: &Id<Formula>, log: &mut Vec<Id<Formula>>) {
        assert_eq!(self.node_status(f), Status::Unsat);
        log.push(f.clone());
        if let Some(children) = self.node(f).children.as_ref() {
            let proved = children.iter().find(|inference| {
                inference
                    .into_iter()
                    .all(|f| self.node_status(f) == Status::Unsat)
            });
            if let Some(inference) = proved {
                for f in inference {
                    self.proof_of(f, log);
                }
            }
        }
    }

    fn computed_status(&self, f: &Id<Formula>) -> Status {
        if *f == Id::new(Formula::F) {
            Status::Unsat
        } else {
            self.node_children(&f)
                .iter()
                .map(|inference| {
                    inference.into_iter().map(|f| self.node_status(f)).product()
                })
                .sum()
        }
    }

    fn propagate_status(&mut self, start: &Id<Formula>) {
        for f in self.ancestors_of(start) {
            let old_status = self.node(&f).status;
            let new_status = self.computed_status(&f);
            if new_status.is_known() && old_status != new_status {
                record(&f, new_status);
                self.node_mut(&f).status = new_status;
            }
        }
    }

    fn propagate_score(&mut self, start: &Id<Formula>) {
        use Status::*;
        for f in self.ancestors_of(start) {
            self.node_mut(&f).score = self
                .node_children(&f)
                .iter()
                .map(|inference| {
                    let mut total = Score::default();
                    let mut count = 0;
                    for f in inference {
                        let status = self.node_status(f);
                        match status {
                            Sat => return 0.into(),
                            Unsat => {
                                continue;
                            }
                            Unknown => {
                                total += self.node_score(f);
                                count += 1;
                            }
                        }
                    }
                    if count != 0 {
                        total / count.into()
                    } else {
                        1.into()
                    }
                })
                .max()
                .unwrap_or_default();
        }
    }

    fn select(&mut self) -> Option<Id<Formula>> {
        let mut current = self.root.clone();

        while self.node(&current).children.is_some() {
            assert_eq!(self.node_status(&current), Status::Unknown);

            let parent_visits = self.node_visits(&current);
            let children = self.node_children(&current).iter();
            let possible = children.filter(|inference| {
                !inference
                    .into_iter()
                    .any(|f| self.node_status(f) == Status::Sat)
            });
            let selected_inference = possible.max_by_key(|inference| {
                let score = inference
                    .into_iter()
                    .map(|f| self.node_score(f))
                    .min()
                    .expect("inference had no children");
                let child_visits =
                    inference.into_iter().map(|f| self.node_visits(f)).sum();
                uct(parent_visits, child_visits, score)
            })?;
            let possible_formulae = selected_inference
                .into_iter()
                .filter(|f| !self.node_status(f).is_known());
            let selected = possible_formulae
                .min_by_key(|f| self.node_score(f))?
                .clone();

            self.node_mut(&current).visits += 1;
            current = selected;
        }

        self.node_mut(&current).visits += 1;
        Some(current)
    }

    fn expand(
        &mut self,
        leaf: &Id<Formula>,
        filter: &HashSet<Id<Formula>>,
    ) -> HashSet<Id<Formula>> {
        let mut new_formulae = HashSet::new();
        let deduced = deductions(leaf)
            .into_iter()
            .filter(|inference| {
                !inference.into_iter().any(|f| filter.contains(f))
            })
            .collect::<Vec<_>>();

        for inference in &deduced {
            for f in inference.into_iter() {
                if !self.nodes.contains_key(f) {
                    self.insert_node(f.clone());
                    new_formulae.insert(f.clone());
                }

                let parents = self.node_parents(f);
                if !parents.contains(leaf) {
                    self.node_mut(f).parents.push(leaf.clone());
                }
            }
        }

        self.node_mut(leaf).children = Some(deduced);
        self.node_mut(leaf).status = self.computed_status(leaf);
        new_formulae
    }
}
