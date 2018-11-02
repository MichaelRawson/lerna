use std::vec::Vec;
use crate::types::Dag;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use parking_lot::RwLock;

use crate::goal::Goal;
use crate::inferences::infer;
use crate::proof::Proof;
use crate::random::{equal_choice, weighted_choice};
use crate::simplifications::simplify;
use crate::types::Set;

pub fn uct(score: f64, parent_visits: usize, child_visits: usize) -> f64 {
    #[allow(non_snake_case)]
    let N = parent_visits as f64;
    let n = child_visits as f64;
    score + (2.0 * N.ln() / n).sqrt()
}

#[derive(Debug)]
struct GoalNode {
    goal: Goal,
    children: RwLock<Vec<Box<InferenceNode>>>,
    atomic_expanded: Atomic<bool>,
    atomic_visits: Atomic<usize>,
    atomic_distance: Atomic<u16>,
    atomic_complete: Atomic<bool>,
}

impl GoalNode {
    fn leaf(goal: Goal) -> Dag<Self> {
        let complete = goal.complete();
        let distance = if complete { 0 } else { 1 };

        dag!(GoalNode {
            goal,
            children: RwLock::new(vec![]),
            atomic_expanded: Atomic::new(false),
            atomic_visits: Atomic::new(1),
            atomic_distance: Atomic::new(distance),
            atomic_complete: Atomic::new(complete),
        })
    }

    fn expanded(&self) -> bool {
        self.atomic_expanded.load(Relaxed)
    }

    fn visits(&self) -> usize {
        self.atomic_visits.load(Relaxed)
    }

    fn distance(&self) -> u16 {
        self.atomic_distance.load(Relaxed)
    }

    fn complete(&self) -> bool {
        self.atomic_complete.load(Relaxed)
    }

    fn step(&self) -> bool {
        let selected = self.select();
        let should_update = match selected {
            Some(next) => next.step(),
            None => self.expand(),
        };
        if should_update {
            self.update();
        }
        should_update
    }

    fn select(&self) -> Option<Dag<Self>> {
        if !self.expanded() {
            return None;
        }

        let inferences = self.children.read();
        let visits = self.visits();
        let distance = self.distance();
        let uct_scores: Vec<f64> = inferences
            .iter()
            .map(|inference| inference.uct_score(visits, distance))
            .collect();

        let selected_index = weighted_choice(&uct_scores);
        let selected_inference = &inferences[selected_index];
        selected_inference.select()
    }

    fn expand(&self) -> bool {
        if self.complete() {
            return false;
        }

        let mut children = self.children.write();
        if self.expanded() {
            return false;
        }

        *children = infer(self.goal.clone())
            .into_iter()
            .map(|x| Box::new(InferenceNode::new(x)))
            .collect();
        self.atomic_expanded.store(true, Relaxed);
        true
    }

    fn update(&self) {
        if !self.expanded() {
            return;
        }
        let children = self.children.read();
        for child in &*children {
            child.update();
        }

        let complete = children.iter().any(|goal| goal.complete());
        let distance = children
            .iter()
            .map(|goal| goal.distance())
            .max()
            .expect("exhausted inferences")
            + 1;

        self.atomic_visits.fetch_add(1, Relaxed);
        self.atomic_complete.store(complete, Relaxed);
        self.atomic_distance.store(distance, Relaxed);
    }

    fn proof(&self) -> Proof {
        if !self.expanded() {
            Proof::leaf(self.goal.clone())
        } else {
            let inferences = self.children.read();
            Proof::branch(
                self.goal.clone(),
                inferences.iter().find(|x| x.complete()).unwrap().proofs(),
            )
        }
    }
}

#[derive(Debug)]
struct InferenceNode {
    subgoals: Vec<Dag<GoalNode>>,
    atomic_visits: Atomic<usize>,
    atomic_distance: Atomic<u16>,
    atomic_complete: Atomic<bool>,
}

impl InferenceNode {
    fn new(subgoals: Set<Goal>) -> Self {
        let node = InferenceNode {
            subgoals: subgoals.into_iter().map(GoalNode::leaf).collect(),
            atomic_visits: Atomic::new(1),
            atomic_distance: Atomic::new(1),
            atomic_complete: Atomic::new(false),
        };
        node.update();
        node
    }

    fn visits(&self) -> usize {
        self.atomic_visits.load(Relaxed)
    }

    fn distance(&self) -> u16 {
        self.atomic_distance.load(Relaxed)
    }

    fn complete(&self) -> bool {
        self.atomic_complete.load(Relaxed)
    }

    fn uct_score(&self, parent_visits: usize, parent_distance: u16) -> f64 {
        let child_visits = self.visits();
        let child_distance = self.distance();
        let difference = i32::from(parent_distance) - i32::from(child_distance);
        let score = f64::from(difference) / f64::from(parent_distance);
        uct(score, parent_visits + 1, child_visits)
    }

    fn select(&self) -> Option<Dag<GoalNode>> {
        let available: Vec<&Dag<GoalNode>> = self
            .subgoals
            .iter()
            .filter(|goal| !goal.complete())
            .collect();
        equal_choice(&available).map(|x| (*x).clone())
    }

    fn update(&self) {
        let complete = self.subgoals.iter().all(|goal| goal.complete());
        let distance = self
            .subgoals
            .iter()
            .filter(|goal| !goal.complete())
            .map(|goal| goal.distance())
            .sum();

        self.atomic_visits.fetch_add(1, Relaxed);
        self.atomic_complete.store(complete, Relaxed);
        self.atomic_distance.store(distance, Relaxed);
    }

    fn proofs(&self) -> Vec<Proof> {
        self.subgoals.iter().map(|x| x.proof()).collect()
    }
}

pub struct Tree {
    root: Dag<GoalNode>,
}

impl Tree {
    pub fn new(start: &Goal) -> Self {
        debug!("simplifying start goal...");
        let simplified = simplify(&start);
        debug!("...simplified.");

        Tree {
            root: GoalNode::leaf(simplified),
        }
    }

    pub fn complete(&self) -> bool {
        self.root.complete()
    }

    pub fn step(&self) {
        self.root.step();
    }

    pub fn proof(self) -> Proof {
        assert!(self.complete());
        self.root.proof()
    }

    pub fn total_visits(&self) -> usize {
        self.root.visits()
    }
}
