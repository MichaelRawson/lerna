use chashmap::CHashMap;
use crossbeam::channel::{Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;
use unique::Id;

use crate::collections::{list, List, Set};
use crate::formula::Formula;
use crate::goal::{Goal, GoalInfo};
use crate::simplification::simplify;

type GoalNodePtr = Arc<Mutex<GoalNode>>;
type InferenceNodePtr = Arc<Mutex<InferenceNode>>;

struct GoalNode {
    goal: Id<Goal>,
    refuted: bool,
    exhausted: bool,
    score: f32,
    visits: u32,
    parents: List<GoalNodePtr>,
    children: Option<List<InferenceNodePtr>>,
}

struct InferenceNode {
    children: List<GoalNodePtr>
}

impl GoalNode {
    fn new_for_goal(
        goals: &CHashMap<Id<Goal>, GoalNodePtr>,
        parents: &List<GoalNodePtr>,
        goal: &Id<Goal>,
    ) -> GoalNodePtr {
        goals.upsert(
            goal.clone(),
            || Arc::new(Mutex::new(GoalNode {
                    goal: goal.clone(),
                    refuted: false,
                    exhausted: false,
                    score: 0.0,
                    visits: 1,
                    parents: parents.clone(),
                    children: None,
            })),
            |node| {
                node.lock().parents.extend(parents.clone());
            },
        );
        goals.get(goal).unwrap().clone()
    }

    fn select_child(&self) -> Option<GoalNodePtr> {
        let children = self.children.as_ref()?;
        unimplemented!()
    }
}

struct Search {
    axioms: Set<Formula>,
    goals: CHashMap<Id<Goal>, GoalNodePtr>,
    root: GoalNodePtr,
    goal_queue: Sender<Id<Goal>>,
    score_queue: Receiver<(Id<Goal>, f32)>,
}

impl Search {
    pub fn new(
        axioms: Set<Formula>,
        negated_conjecture: Id<Formula>,
        goal_queue: Sender<Id<Goal>>,
        score_queue: Receiver<(Id<Goal>, f32)>,
    ) -> Self {
        let start = Id::new(Goal::new(set![negated_conjecture]));
        log::info!("simplifying initial goal...");
        let info = GoalInfo::new(&start, &axioms);
        let (start, _) = simplify(&start, &info);
        log::info!("...done, search space initialised.");

        goal_queue.send(start.clone()).unwrap();

        let goals = CHashMap::new();
        let root = GoalNode::new_for_goal(&goals, &list![], &start);

        Self {
            axioms,
            goals,
            root,
            goal_queue,
            score_queue,
        }
    }

    pub fn is_refuted(&self) -> bool {
        self.root.lock().refuted
    }


}

/*
    pub fn work(&mut self) {
        if let Some((goal, score)) = self.try_deque() {
            self.record_mut(&goal).score = score;
            self.propagate(goal);
        } else if let Some(selected) = self.select() {
            self.expand(&selected);
        }
    }

    fn record_mut(&mut self, goal: &Id<Goal>) -> &mut Record {
        self.space.get_mut(goal).unwrap()
    }

    fn goal_uct(&self, goal: &Id<Goal>, parent: &Id<Goal>) -> Score {
        let record = &self.space[goal];

        let parent_visits = self.space[parent].visits as f32;
        let visits = record.visits as f32;
        let skepticism = OPTIONS.skepticism;
        let max_score = self.max_score as f32;
        let score = record.score;
        let exploitation = 1.0 - score / max_score;
        let exploration = (skepticism * parent_visits.ln() / visits).sqrt();

        Score::new(exploitation + exploration)
    }

    fn deduction_uct(&self, goals: &Set<Goal>, parent: &Id<Goal>) -> Score {
        goals
            .into_iter()
            .map(|goal| self.goal_uct(goal, parent))
            .min()
            .unwrap()
    }

    fn select_child(
        &self,
        children: &List<Set<Goal>>,
        parent: &Id<Goal>,
    ) -> Option<Id<Goal>> {
        children
            .into_iter()
            .filter(|child| 
            .max_by_key(|child| self.deduction_uct(child, &parent))?
            .into_iter()
            .min_by_key(|goal| self.goal_uct(goal, &parent))
            .expect("inference produced empty set")
            .clone()
    }

    fn select(&mut self) -> Option<(Id<Goal>, Set<Id<Goal>>)> {
        let mut current_goal = self.start.clone();
        let mut visited = List::new();

        loop {
            self.record_mut(&current_goal).visits += 1;
            visited.push(current_goal.clone());

            if let Some(children) = &self.space[&current_goal].children {
                if let Some(selection) = self.select_child(children, &current_goal) {
                }
                else {
                    //self.record_mut(current_goal).exhausted = true;
                }
            }
            else {
                return (current_goal, visited.iter().into())
            }
            current_goal = ;
        }
    }

    fn simplified_deductions(&self, goal: &Id<Goal>) -> List<Set<Goal>> {
        let info = GoalInfo::new(goal, &self.axioms);
        let mut deduced: List<Set<Goal>> = deductions(goal, &info)
            .into_iter()
            .map(|(_, inferences)| {
                inferences
                    .into_iter()
                    .map(|inference| Goal::apply(goal, &inference))
                    .map(|goal| simplify(&goal, &info).0)
                    .collect()
            })
            .collect();

        deduced
    }

    fn expand(&mut self, goal: &Id<Goal>) {
        let deduced = self.simplified_deductions(goal);
        let new_goals: HashSet<&Id<Goal>> = (&deduced)
            .into_iter()
            .map(|goals| goals.into_iter())
            .flatten()
            .collect();

        let max_score = self.max_score;
        for new in new_goals {
            let record = self.space.entry(new.clone()).or_insert(Record {
                refuted: false,
                visits: 1,
                score: max_score,
                parents: set![],
                children: None,
            });
            record.parents = record.parents.with(goal.clone());
            self.submit(new.clone());
        }

        self.record_mut(goal).children = Some(deduced);
    }

    fn goal_score(&self, goal: &Id<Goal>) -> f32 {
        let record = &self.space[goal];
        match &record.children {
            Some(ref children) => {
                children
                    .iter()
                    .map(|goals| {
                        goals
                            .into_iter()
                            .map(|goal| self.space[goal].score)
                            .sum()
                    })
                    .map(Score::new)
                    .min()
                    .unwrap()
                    .0
            }
            None => record.score,
        }
    }

    fn goal_refuted(&self, goal: &Id<Goal>) -> bool {
        match &self.space[goal].children {
            Some(ref children) => children.iter().any(|goals| {
                goals.into_iter().all(|goal| self.space[goal].refuted)
            }),
            None => Goal::is_trivial(&goal),
        }
    }

    fn propagate_goal(&mut self, goal: &Id<Goal>) -> Set<Goal> {
        let score = self.goal_score(goal);
        self.max_score = max(Score::new(self.max_score), Score::new(score)).0;
        let refuted = self.goal_refuted(goal);

        let record = self.record_mut(goal);
        record.score = score;
        record.refuted = refuted;
        record.parents.clone()
    }

    fn propagate(&mut self, start: Id<Goal>) {
        let mut completed = HashSet::new();
        let mut todo = VecDeque::new();
        todo.push_back(start);

        while let Some(next) = todo.pop_front() {
            let parents = self.propagate_goal(&next);
            completed.insert(next);
            todo.extend(
                (&parents)
                    .into_iter()
                    .filter(|goal| !completed.contains(*goal))
                    .cloned(),
            );
        }
    }

    fn submit(&self, goal: Id<Goal>) {
        self.goal_queue.send(goal).unwrap_or_else(|e| {
            log::error!("failed to queue goal: {:?}", e);
            panic!("queue failed");
        });
    }

    fn try_deque(&self) -> Option<(Id<Goal>, f32)> {
        self.score_queue
            .try_recv()
            .map_err(|e| {
                if e.is_disconnected() {
                    log::error!("failed to dequeue score: {:?}", e);
                    panic!("queue failed");
                }
            })
            .ok()
    }
}
*/
