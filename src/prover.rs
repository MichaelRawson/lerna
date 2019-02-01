use crossbeam::channel::bounded;
use crossbeam::scope;
use unique::Id;

use crate::formula::Formula;
use crate::heuristic::Heuristic;
use crate::options::OPTIONS;
use crate::search::Search;
use crate::set::Set;

pub struct Prover {
    search: Search,
    heuristic: Heuristic,
}

impl Prover {
    pub fn new(axioms: Id<Set<Formula>>, negated_conjecture: Id<Formula>) -> Self {
        let (goal_send, goal_recv) = bounded(OPTIONS.goal_queue_size);
        let (score_send, score_recv) = bounded(OPTIONS.score_queue_size);

        let search = Search::new(axioms, negated_conjecture, goal_send, score_recv);
        let heuristic = Heuristic::new(goal_recv, score_send);

        Self { search, heuristic }
    }

    fn should_continue(&self) -> bool {
        self.search.should_continue() && OPTIONS.within_time()
    }

    fn search_explore_task(&self) {
        while self.should_continue() {
            self.search.explore_step();
        }
    }

    fn search_evaluate_task(&self) {
        while self.should_continue() {
            self.search.evaluate_step();
        }
    }

    fn heuristic_task(&self) {
        while self.should_continue() {
            self.heuristic.step();
        }
    }

    pub fn run(&self) -> bool {
        log::info!("searching...");
        scope(|threads| {
            threads.builder()
                .name("search_explore".into())
                .spawn(|_| self.search_explore_task())
                .unwrap();
            threads.builder()
                .name("search_evaluate".into())
                .spawn(|_| self.search_evaluate_task())
                .unwrap();
            threads.builder()
                .name("heuristic".into())
                .spawn(|_| self.heuristic_task())
                .unwrap();
        }).unwrap();
        log::info!("...done");

        self.search.search_complete()
    }
}
