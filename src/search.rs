use std::cell::{Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet, VecDeque};
use unique::Id;

use crate::collections::IdSet;
use crate::deduction::deductions;
use crate::formula::Formula;
use crate::options::OPTIONS;
use crate::score::Score;
use crate::status::Status;

#[derive(Debug, Default)]
pub struct Node {
    parents: Vec<Id<Formula>>,
    children: Option<Vec<IdSet<Formula>>>,
    score: Score,
    status: Status,
    visits: usize,
}

impl Node {
    fn add_parent(&mut self, parent: &Id<Formula>) {
        if !self.parents.contains(parent) {
            self.parents.push(parent.clone());
        }
    }

    fn children(&self) -> &Vec<IdSet<Formula>> {
        self.children.as_ref().unwrap()
    }
}

fn uct(parent_visits: usize, child_visits: usize, score: Score) -> Score {
    let parent_visits = parent_visits as f32;
    let child_visits = (child_visits + 1) as f32;
    let skepticism = OPTIONS.skepticism;
    Score::new(
        score.0 + (skepticism * parent_visits.ln() / child_visits).sqrt(),
    )
}

pub struct Search {
    root: Id<Formula>,
    pub nodes: HashMap<Id<Formula>, RefCell<Node>>,
}

impl Search {
    pub fn new(f: &Id<Formula>) -> Self {
        let root = f.clone();
        let nodes = HashMap::new();
        let mut new = Self { nodes, root };
        new.insert_node(&f);
        new
    }

    pub fn status(&self) -> Status {
        self.node(&self.root).status
    }

    pub fn set_status(&mut self, f: &Id<Formula>, status: Status) {
        self.node_mut(f).status = status;
        self.propagate_status(f);
    }

    pub fn set_score(&mut self, f: &Id<Formula>, score: Score) {
        self.node_mut(f).score = score;
        self.propagate_score(f)
    }

    pub fn do_step(&mut self) -> Vec<Id<Formula>> {
        let (leaf, path) = self.select();
        let new_formulae = self.expand(&leaf, &path);

        if self.node(&leaf).status != Status::Unknown {
            self.propagate_status(&leaf);
        }

        new_formulae
    }

    fn insert_node(&mut self, f: &Id<Formula>) {
        self.nodes.insert(f.clone(), RefCell::default());
    }

    fn node(&self, f: &Id<Formula>) -> Ref<Node> {
        self.nodes[f].borrow()
    }

    fn node_mut(&self, f: &Id<Formula>) -> RefMut<Node> {
        self.nodes[f].borrow_mut()
    }

    fn ancestors_of(&self, leaf: &Id<Formula>) -> Vec<Id<Formula>> {
        let mut done = HashSet::new();
        let mut todo = VecDeque::new();
        let mut parents = vec![];
        done.insert(leaf.clone());
        todo.extend(self.node(&leaf).parents.iter().cloned());

        while let Some(next) = todo.pop_front() {
            if !done.contains(&next) {
                done.insert(next.clone());
                todo.extend(self.node(&next).parents.iter().cloned());
                parents.push(next);
            }
        }

        parents
    }

    fn select(&self) -> (Id<Formula>, HashSet<Id<Formula>>) {
        let mut path = HashSet::new();
        let mut current = self.root.clone();
        while self.node(&current).children.is_some() {
            self.node_mut(&current).visits += 1;
            let node = self.node(&current);

            log::debug!("{:?}, {:?}", current, node);
            let next = node
                .children()
                .iter()
                .filter(|inference| {
                    !inference
                        .into_iter()
                        .any(|f| self.node(f).status == Status::Sat)
                })
                .max_by_key(|inference| {
                    let score = inference
                        .into_iter()
                        .map(|f| self.node(f).score)
                        .min()
                        .unwrap();
                    let parent_visits = node.visits;
                    let child_visits = inference
                        .into_iter()
                        .map(|f| self.node(f).visits)
                        .sum();
                    uct(parent_visits, child_visits, score)
                })
                .unwrap()
                .into_iter()
                .filter(|f| self.node(f).status != Status::Unsat)
                .min_by_key(|f| self.node(f).score)
                .unwrap()
                .clone();

            path.insert(current);
            current = next;
        }

        self.node_mut(&current).visits += 1;
        path.insert(current.clone());
        (current, path)
    }

    fn expand(
        &mut self,
        f: &Id<Formula>,
        filter: &HashSet<Id<Formula>>,
    ) -> Vec<Id<Formula>> {
        let (deduced, new_formulae) = {
            let mut new_formulae = vec![];
            let deduced = deductions(f)
                .into_iter()
                .filter(|inference| {
                    !inference.into_iter().any(|f| filter.contains(f))
                })
                .collect::<Vec<_>>();

            for inference in &deduced {
                for new in inference.into_iter() {
                    self.nodes.entry(new.clone()).or_default();
                    self.node_mut(new).add_parent(f);
                    new_formulae.push(new.clone());
                }
            }

            (deduced, new_formulae)
        };

        let mut node = self.node_mut(f);
        if deduced.is_empty() {
            node.status = Status::Sat;
        } else if deduced.contains(&idset![Id::new(Formula::F)]) {
            node.status = Status::Unsat;
        }

        node.children = Some(deduced);
        new_formulae
    }

    fn propagate_status(&mut self, start: &Id<Formula>) {
        for f in self.ancestors_of(start) {
            let status = self
                .node(&f)
                .children()
                .iter()
                .map(|fs| fs.into_iter().map(|f| self.node(f).status).product())
                .sum::<Status>();
            self.node_mut(&f).status = status;
        }
    }

    fn propagate_score(&mut self, start: &Id<Formula>) {
        use Status::*;
        for f in self.ancestors_of(start) {
            if self.node(&f).children.is_none() {
                log::debug!("{:?}", f);
                log::debug!("{:#?}", self.nodes);
                assert!(false);
            }
            let score = self
                .node(&f)
                .children()
                .iter()
                .map(|fs| {
                    fs.into_iter()
                        .map(|f| {
                            let node = self.node(f);
                            let score = node.score;
                            let status = node.status;
                            match status {
                                Unsat => Score::new(1.0),
                                _ => score,
                            }
                        })
                        .sum::<Score>()
                        / Score::new(fs.len() as f32)
                })
                .max()
                .unwrap_or_default();
            self.node_mut(&f).score = score;
        }
    }
}
