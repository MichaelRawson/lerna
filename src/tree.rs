use std::sync::Arc;
use std::vec::Vec;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use parking_lot::RwLock;

use core::{Formula, Goal, Set};
use inferences::infer;
use uct::uct;
use util::{equal_choice, weighted_choice};

enum GoalNodeContent {
    Leaf,
    Branch(Vec<Box<InferenceNode>>),
}

struct GoalNode {
    goal: Goal,
    lock: RwLock<GoalNodeContent>,
    atomic_visits: Atomic<usize>,
    atomic_distance: Atomic<u16>,
    atomic_complete: Atomic<bool>,
}

impl GoalNode {
    fn leaf(goal: Goal) -> Arc<Self> {
        let complete = goal.complete();
        let distance = if complete { 0 } else { 1 };

        Arc::new(GoalNode {
            goal,
            lock: RwLock::new(GoalNodeContent::Leaf),
            atomic_visits: Atomic::new(1),
            atomic_distance: Atomic::new(distance),
            atomic_complete: Atomic::new(complete),
        })
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

    fn select(&self) -> Option<Arc<Self>> {
        if self.complete() {
            warn!("selecting complete node");
            return None;
        }

        let locked = self.lock.read();
        let inferences = match *locked {
            GoalNodeContent::Branch(ref inferences) => inferences,
            GoalNodeContent::Leaf => return None,
        };

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
            warn!("contention: expanding complete node");
            return false;
        }
        let mut locked = self.lock.write();
        let children = match *locked {
            GoalNodeContent::Leaf => infer(self.goal.clone())
                .into_iter()
                .map(InferenceNode::new)
                .collect(),
            GoalNodeContent::Branch(_) => {
                debug!("contention: expanding already-expanded node");
                return false;
            }
        };
        *locked = GoalNodeContent::Branch(children);
        true
    }

    fn update(&self) {
        let locked = self.lock.read();
        let children: &Vec<Box<InferenceNode>> = match *locked {
            GoalNodeContent::Branch(ref x) => &x,
            GoalNodeContent::Leaf => return,
        };
        for child in children {
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

    fn proof(&self, mut done: Set<Arc<Formula>>, proof: &mut Vec<Arc<Formula>>) {
        let locked = self.lock.read();
        match *locked {
            GoalNodeContent::Leaf => {
                proof.push(Arc::new(Formula::F));
            }
            GoalNodeContent::Branch(ref inferences) => {
                for f in &self.goal.formulae {
                    if !done.contains(f) {
                        proof.push(f.clone());
                        done.insert(f.clone());
                    }
                }

                let inference = inferences.iter().find(|x| x.complete()).unwrap();

                inference.proof(&done, proof);
            }
        }
    }
}

struct InferenceNode {
    subgoals: Vec<Arc<GoalNode>>,
    atomic_visits: Atomic<usize>,
    atomic_distance: Atomic<u16>,
    atomic_complete: Atomic<bool>,
}

impl InferenceNode {
    fn new(subgoals: Set<Goal>) -> Box<Self> {
        let node = InferenceNode {
            subgoals: subgoals.into_iter().map(GoalNode::leaf).collect(),
            atomic_visits: Atomic::new(1),
            atomic_distance: Atomic::new(1),
            atomic_complete: Atomic::new(false),
        };
        node.update();
        Box::new(node)
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

    fn select(&self) -> Option<Arc<GoalNode>> {
        let available: Vec<&Arc<GoalNode>> = self
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

    fn proof(&self, done: &Set<Arc<Formula>>, proof: &mut Vec<Arc<Formula>>) {
        for subgoal in &self.subgoals {
            subgoal.proof(done.clone(), proof);
        }
    }
}

pub struct Tree {
    root: Arc<GoalNode>,
}

impl Tree {
    pub fn new(goal: Goal) -> Self {
        Tree {
            root: GoalNode::leaf(goal),
        }
    }

    pub fn complete(&self) -> bool {
        self.root.complete()
    }

    pub fn step(&self) {
        let mut stack: Vec<Arc<GoalNode>> = vec![];
        let mut current = self.root.clone();

        // select
        while let Some(next) = current.clone().select() {
            stack.push(current);
            current = next;
        }
        stack.push(current.clone());

        // expand
        if !current.expand() {
            return;
        }

        // update
        while let Some(top) = stack.pop() {
            top.update();
        }
    }

    pub fn proof(&self, done: Set<Arc<Formula>>, proof: &mut Vec<Arc<Formula>>) {
        assert!(self.complete());
        self.root.proof(done, proof);
    }

    pub fn total_visits(&self) -> usize {
        self.root.visits()
    }
}
