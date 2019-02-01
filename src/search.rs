use crossbeam::channel::{Sender, Receiver};
use chashmap::{CHashMap, ReadGuard, WriteGuard};
use std::collections::{HashSet, VecDeque};
use std::mem::drop;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Duration;
use std::vec::Vec;
use unique::Id;

use crate::deduction::deductions;
use crate::formula::Formula;
use crate::goal::Goal;
use crate::options::OPTIONS;
use crate::score::Score;
use crate::set::Set;

#[derive(Debug, Default)]
struct Node {
    pub children: Option<Vec<Vec<Goal>>>,
    pub parents: Vec<Goal>,
    pub score: Score,
    pub visits: AtomicUsize,
    pub refuted: bool,
}

type ReadNode<'a> = ReadGuard<'a, Goal, Node>;
type WriteNode<'a> = WriteGuard<'a, Goal, Node>;

pub struct Search {
    axioms: Id<Set<Formula>>,
    goal_dag: CHashMap<Goal, Node>,
    current_max_score: RwLock<Score>,
    start: Goal,
    goal_queue: Sender<Goal>,
    evaluation_queue: Receiver<(Goal, Score)>,
}

impl Search {
    pub fn new(
        axioms: Id<Set<Formula>>,
        negated_conjecture: Id<Formula>,
        goal_queue: Sender<Goal>,
        evaluation_queue: Receiver<(Goal, Score)>,
    ) -> Self {
        let start = Goal::new(Id::new(set![negated_conjecture]));
        let goal_dag = CHashMap::new();
        goal_dag.insert(start.clone(), Node::default());
        let current_max_score = RwLock::default();

        let new = Self {
            axioms,
            goal_dag,
            current_max_score,
            start,
            goal_queue,
            evaluation_queue,
        };
        new.enqueue_goal(new.start.clone());

        log::info!("search space initialised");
        new
    }

    fn starved(&self) -> bool {
        self.evaluation_queue.is_empty()
    }

    fn max_score(&self) -> Score {
        *self.current_max_score.read().unwrap()
    }

    fn node(&self, goal: &Goal) -> ReadNode {
        self.goal_dag.get(goal).unwrap()
    }

    fn node_mut(&self, goal: &Goal) -> WriteNode {
        self.goal_dag.get_mut(goal).unwrap()
    }

    fn score(&self, goal: &Goal) -> Score {
        self.node(goal).score
    }

    fn refuted(&self, goal: &Goal) -> bool {
        self.node(goal).refuted
    }

    fn visits(&self, goal: &Goal) -> usize {
        self.node(goal).visits.load(Ordering::Relaxed)
    }

    fn enqueue_goal(&self, goal: Goal) {
        self.goal_queue.send(goal).unwrap_or_else(|e| {
            log::error!("failed to enqueue goal: {:?}", e);
            panic!("queue failed")
        })
    }

    fn deque_evaluation(&self) -> (Goal, Score) {
        self.evaluation_queue.recv().unwrap_or_else(|e| {
            log::error!("failed to deque evaluation: {:?}", e);
            panic!("queue failed")
        })
    }

    fn goal_uct(&self, goal: &Goal, parent: &Goal) -> Score {
        let visits = (self.visits(goal) + 1) as f32;
        let parent_visits = self.visits(parent) as f32;
        let skepticism = OPTIONS.skepticism;
        let max_score = (self.max_score().0 + 1.0) as f32;

        let exploitation = 1.0 - self.score(goal).0 / max_score;
        let exploration = (skepticism * parent_visits.ln() / visits).sqrt();

        Score::new(exploitation + exploration)
    }

    fn goals_uct(&self, goals: &[Goal], parent: &Goal) -> Score {
        goals
            .iter()
            .map(|goal| self.goal_uct(goal, parent))
            .min()
            .expect("inference produced the empty set")
    }

    fn unlink(&self, ignore: &Goal) {
        let node = self.node(ignore);
        let child_goals: HashSet<Goal> = node.children.as_ref().unwrap()
            .iter()
            .map(|goals| goals.iter())
            .flatten()
            .filter(|goal| goal != &ignore)
            .cloned()
            .collect();
        let parent_goals: Vec<_> = node.parents.iter()
            .filter(|goal| goal != &ignore)
            .cloned()
            .collect();

        drop(node);

        for child in child_goals {
            let mut write = self.node_mut(&child);
            write.parents.retain(|goal| goal != ignore);
        }

        for parent in parent_goals {
            let mut write = self.node_mut(&parent);
            let children = write.children.as_mut().unwrap();
            children.retain(|goals| !goals.contains(ignore));
        }

        self.goal_dag.remove(ignore);
    }

    fn remove_orphans(&self) {
        self.goal_dag.retain(|_goal, node| !node.parents.is_empty());
    }

    fn select_child(&self, current_goal: &Goal) -> Option<Goal> {
        let read = self.node(&current_goal);
        read.visits.fetch_add(1, Ordering::Relaxed);

        let children = read.children.as_ref().unwrap();
        if children.is_empty() {
            return None;
        }

        let best = children
            .iter()
            .max_by_key(|child| self.goals_uct(child, &current_goal))
            .unwrap();

        let best_hardest = best
            .iter()
            .min_by_key(|child| self.goal_uct(child, &current_goal))
            .unwrap();

        Some(best_hardest.clone())
    }

    fn select(&self) -> Option<Goal> {
        let mut current_goal = self.start.clone();
        loop {
            if self.node(&current_goal).children.is_none() {
                return Some(current_goal);
            }

            current_goal = match self.select_child(&current_goal) {
                Some(goal) => goal,
                None => {
                    if current_goal == self.start || current_goal.is_trivial() {
                        return None;
                    }

                    self.unlink(&current_goal);
                    self.start.clone()
                }
            }
        }
    }

    fn expand(&self, expand: &Goal) {
        let all = deductions(expand, self.axioms.clone());

        let goals: HashSet<Goal> = all.iter()
            .map(|goals| goals.iter())
            .flatten()
            .cloned()
            .collect();

        for goal in goals {
            self.goal_dag.upsert(
                goal.clone(),
                || {
                    let mut node = Node::default();
                    node.parents = vec![expand.clone()];
                    node.score = self.max_score();
                    node.refuted = goal.is_trivial();
                    self.enqueue_goal(goal.clone());
                    node
                },
                |node| {
                    if !node.parents.contains(expand) {
                        node.parents.push(expand.clone());
                    }
                },
            );
        }

        self.node_mut(expand).children = Some(all);
    }

    fn propagate_goal(&self, goal: &Goal) -> Vec<Goal> {
        let (score, refuted) = self
            .node(goal)
            .children
            .as_ref()
            .map(|children| {
                let score = children
                    .iter()
                    .map(|goals| goals.iter().map(|goal| self.score(goal)).sum())
                    .min()
                    .unwrap_or_else(|| self.max_score());

                if score > self.max_score() {
                    let mut current = self.current_max_score.write().unwrap();
                    *current = score.max(*current);
                }

                let refuted = children
                    .iter()
                    .any(|goals| goals.iter().all(|goal| self.refuted(goal)));

                (score, refuted)
            })
            .unwrap_or_else(|| (self.node(goal).score, false));

        let refuted = refuted || goal.is_trivial();
        let mut write = self.node_mut(goal);
        write.score = score;
        write.refuted = refuted;
        write.parents.clone()
    }

    fn propagate(&self, start: Goal) {
        let mut completed = HashSet::new();
        let mut todo = VecDeque::new();
        todo.push_back(start);

        while let Some(next) = todo.pop_front() {
            let parents = self.propagate_goal(&next);
            completed.insert(next);
            todo.extend(parents.into_iter().filter(|goal| !completed.contains(goal)));
        }
    }

    pub fn explore_step(&self) {
        if let Some(goal) = self.select() {
            self.expand(&goal);
        }
        else {
            sleep(Duration::from_millis(1));
        }
    }

    pub fn evaluate_step(&self) {
        if self.starved() {
            sleep(Duration::from_millis(10));
            return;
        }

        let (goal, score) = self.deque_evaluation();
        self.node_mut(&goal).score = score;
        self.propagate(goal);
    }

    pub fn search_exhausted(&self) -> bool {
        let start = self.node(&self.start);
        start.children.as_ref()
            .map(|children| children.is_empty())
            .unwrap_or_default()
    }

    pub fn search_complete(&self) -> bool {
        self.node(&self.start).refuted
    }

    pub fn should_continue(&self) -> bool {
        !self.search_exhausted() && !self.search_complete()
    }
}
