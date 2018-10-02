use std::sync::Arc;
use std::vec::Vec;

use atomic::Atomic;
use atomic::Ordering::Relaxed;
use parking_lot::RwLock;
use rand::distributions::{Distribution, Uniform};
use rand::{thread_rng, Rng};

use core::{Formula, Goal, Set};
use inferences::infer;
use uct::uct;
use util::Score;

enum GoalNodeContent {
    Leaf,
    Branch(Vec<Box<InferenceNode>>),
}

struct GoalNode {
    goal: Goal,
    locked: RwLock<GoalNodeContent>,
    visits: Atomic<usize>,
    distance: Atomic<u16>,
    complete: Atomic<bool>,
}

impl GoalNode {
    pub fn leaf(goal: Goal) -> Arc<Self> {
        let complete = goal.complete();
        let distance = if complete { 0 } else { 1 };
        trace!("new leaf: {:?}", goal.formulae);

        Arc::new(GoalNode {
            goal,
            locked: RwLock::new(GoalNodeContent::Leaf),
            visits: Atomic::new(1),
            distance: Atomic::new(distance),
            complete: Atomic::new(complete),
        })
    }

    fn check_complete(&self) -> bool {
        self.complete.load(Relaxed)
    }

    fn select(&self, top_distance: u16) -> Option<Arc<Self>> {
        if self.check_complete() {
            return None;
        }

        let lock = self.locked.read();
        let inferences = match *lock {
            GoalNodeContent::Branch(ref inferences) => inferences,
            GoalNodeContent::Leaf => return None,
        };

        let parent_visits = self.visits.load(Relaxed);
        let uct_scores: Vec<f64> = inferences
            .iter()
            .map(|inference| {
                let child_visits = inference.visits.load(Relaxed);
                let distance = inference.distance.load(Relaxed);
                let difference = i32::from(top_distance) - i32::from(distance);
                let score = f64::from(difference) / f64::from(top_distance);
                uct(score, parent_visits + 1, child_visits)
            }).collect();
        let uct_total: f64 = uct_scores.iter().sum();
        let normalised = uct_scores.iter().map(|x| x / uct_total);
        let cumulative: Vec<Score> = normalised
            .scan(0.0, |running, x| {
                *running += x;
                Some(*running)
            }).map(Score::new)
            .collect();

        let mut rng = thread_rng();
        let range = Uniform::new(0.0, 1.0);
        let sample = Score::new(range.sample(&mut rng));
        let selected_index = cumulative
            .binary_search(&sample)
            .unwrap_or_else(|index| index);
        let selected_inference = &inferences[selected_index];

        let available_goals: Vec<&Arc<GoalNode>> = selected_inference
            .subgoals
            .iter()
            .filter(|goal| !goal.check_complete())
            .collect();

        rng.choose(&available_goals).map(|x| (*x).clone())
    }

    fn expand(&self) -> bool {
        if self.check_complete() {
            return false;
        }
        let mut locked = self.locked.write();
        let children = match *locked {
            GoalNodeContent::Leaf => infer(self.goal.clone())
                .into_iter()
                .map(InferenceNode::new)
                .collect(),
            GoalNodeContent::Branch(_) => return false,
        };
        *locked = GoalNodeContent::Branch(children);
        true
    }

    fn update(&self) {
        let locked = self.locked.read();
        let children: &Vec<Box<InferenceNode>> = match *locked {
            GoalNodeContent::Branch(ref x) => &x,
            GoalNodeContent::Leaf => panic!("updating a leaf node"),
        };
        for child in children {
            child.update();
        }

        let complete = children.iter().any(|goal| goal.check_complete());
        let distance = children
            .iter()
            .map(|goal| goal.distance.load(Relaxed))
            .max()
            .expect("exhausted inferences");

        self.visits.fetch_add(1, Relaxed);
        self.complete.store(complete, Relaxed);
        self.distance.store(distance, Relaxed);
    }

    fn proof(&self, mut done: Set<Arc<Formula>>, proof: &mut Vec<Arc<Formula>>) {
        let lock = self.locked.read();
        match *lock {
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

                let inference = inferences.iter().find(|x| x.check_complete()).unwrap();

                inference.proof(&done, proof);
            }
        }
    }
}

struct InferenceNode {
    subgoals: Vec<Arc<GoalNode>>,
    visits: Atomic<usize>,
    distance: Atomic<u16>,
    complete: Atomic<bool>,
}

impl InferenceNode {
    fn new(subgoals: Set<Goal>) -> Box<Self> {
        let node = InferenceNode {
            subgoals: subgoals.into_iter().map(GoalNode::leaf).collect(),
            visits: Atomic::new(1),
            distance: Atomic::new(1),
            complete: Atomic::new(false),
        };
        node.update();
        Box::new(node)
    }

    fn check_complete(&self) -> bool {
        self.complete.load(Relaxed)
    }

    fn update(&self) {
        let complete = self.subgoals.iter().all(|goal| goal.check_complete());
        let distance = self
            .subgoals
            .iter()
            .filter(|goal| !goal.check_complete())
            .map(|goal| goal.distance.load(Relaxed))
            .sum();

        self.visits.fetch_add(1, Relaxed);
        self.complete.store(complete, Relaxed);
        self.distance.store(distance, Relaxed);
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
        self.root.check_complete()
    }

    pub fn step(&self) {
        let mut stack: Vec<Arc<GoalNode>> = vec![];
        let mut current = self.root.clone();
        let top_distance = self.root.distance.load(Relaxed);

        // select
        while let Some(next) = current.clone().select(top_distance) {
            stack.push(current);
            current = next;
        }

        // expand
        if !current.expand() {
            trace!("contention while expanding tree");
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
        self.root.visits.load(Relaxed)
    }
}
