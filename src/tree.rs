use std::sync::Arc;
use std::vec::Vec;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use parking_lot::RwLock;

use core::{Goal, Proof, Set};
use inferences::infer;
use uct::uct;
use util::{assert_ownership, equal_choice, weighted_choice};

#[derive(Debug)]
enum GoalNodeContent {
    Leaf,
    Branch(Vec<Box<InferenceNode>>),
}

#[derive(Debug)]
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

    fn select(&self) -> Option<Arc<Self>> {
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
            return false;
        }

        let mut locked = self.lock.write();
        let children = match *locked {
            GoalNodeContent::Leaf => infer(self.goal.clone())
                .into_iter()
                .map(InferenceNode::new)
                .collect(),
            GoalNodeContent::Branch(_) => {
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

    fn proof(self) -> Box<Proof> {
        let goal = self.goal;
        let unlocked = self.lock.into_inner();
        match unlocked {
            GoalNodeContent::Leaf => Box::new(Proof::leaf(&goal)),
            GoalNodeContent::Branch(inferences) => Box::new(Proof::branch(
                goal,
                inferences
                    .into_iter()
                    .find(|x| x.complete())
                    .unwrap()
                    .proofs(),
            )),
        }
    }
}

#[derive(Debug)]
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

    fn proofs(self) -> Vec<Box<Proof>> {
        self.subgoals
            .into_iter()
            .map(assert_ownership)
            .map(|x| x.proof())
            .collect()
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
        self.root.step();
    }

    pub fn proof(self) -> Box<Proof> {
        assert!(self.complete());
        let root = assert_ownership(self.root);
        root.proof()
    }

    pub fn total_visits(&self) -> usize {
        self.root.visits()
    }
}