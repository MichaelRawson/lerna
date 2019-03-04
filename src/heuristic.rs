use crossbeam::channel::{Receiver, Sender};
use std::time::Duration;
use unique::Id;

use crate::goal::Goal;

pub struct Heuristic {
    goal_queue: Receiver<Id<Goal>>,
    score_queue: Sender<(Id<Goal>, f32)>,
}

impl Heuristic {
    pub fn new(
        goal_queue: Receiver<Id<Goal>>,
        score_queue: Sender<(Id<Goal>, f32)>,
    ) -> Self {
        Self {
            goal_queue,
            score_queue,
        }
    }

    fn try_deque(&self) -> Option<Id<Goal>> {
        self.goal_queue
            .recv_timeout(Duration::from_millis(1))
            .map_err(|e| {
                if e.is_disconnected() {
                    log::error!("failed to dequeue score: {:?}", e);
                    panic!("queue failed");
                }
            })
            .ok()
    }

    fn submit(&self, goal: Id<Goal>, score: f32) {
        self.score_queue.send((goal, score)).unwrap_or_else(|e| {
            log::error!("failed to enqueue score: {:?}", e);
            panic!("queue failed")
        })
    }

    pub fn work(&mut self) {
        if let Some(goal) = self.try_deque() {
            self.submit(goal, 0.0);
        }
    }

    pub fn cleanup(&mut self) {
        while !self.goal_queue.is_empty() {
            if self.goal_queue.recv().is_err() {
                return;
            }
        }
    }
}
