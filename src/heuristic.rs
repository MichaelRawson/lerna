use crossbeam::channel::{Receiver, Sender};
use std::thread::sleep;
use std::time::Duration;

use crate::goal::Goal;
use crate::score::Score;

pub struct Heuristic {
    goal_queue: Receiver<Goal>,
    score_queue: Sender<(Goal, Score)>,
}

impl Heuristic {
    pub fn new(goal_queue: Receiver<Goal>, score_queue: Sender<(Goal, Score)>) -> Self {
        let new = Self {
            goal_queue,
            score_queue,
        };

        log::info!("heuristic initialised");
        new
    }

    fn starved(&self) -> bool {
        self.goal_queue.is_empty()
    }

    fn deque_goal(&self) -> Goal {
        self.goal_queue.recv().unwrap_or_else(|e| {
            log::error!("failed to dequeue goal: {:?}", e);
            panic!("queue failed")
        })
    }

    fn enqueue_score(&self, goal: Goal, score: Score) {
        self.score_queue.send((goal, score)).unwrap_or_else(|e| {
            log::error!("failed to enqueue score: {:?}", e);
            panic!("queue failed")
        })
    }

    pub fn step(&self) {
        if self.starved() {
            sleep(Duration::from_millis(1));
            return;
        }

        let goal = self.deque_goal();
        self.enqueue_score(goal, Score::default());
    }
}
