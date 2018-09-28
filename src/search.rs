use std::vec::Vec;
use std::sync::Arc;

use core::{Formula, Goal};
use options::SearchOptions;
use simplifications::simplify;
use tree::Tree;

pub enum SearchResult {
    Failed,
    Proof(Vec<Arc<Formula>>)
}

pub struct Search {
    original: Goal,
    tree: Tree
}

impl Search {
    pub fn new(_options: &SearchOptions, start: Goal) -> Self {
        debug!("simplifying start goal...");
        let simplified = simplify(start.clone());
        debug!("...simplified.");

        let result = Search {
            original: start,
            tree: Tree::new(simplified)
        };
        debug!("search space initialized");
        result
    }

    pub fn go(&self, max: usize) -> SearchResult {
        for _ in 0..max {
            self.tree.step();
        }

        if self.tree.complete() {
            debug!("proof found");
            let formulae = self.original.formulae.clone();
            let mut proof = formulae.clone().into_iter().collect();
            self.tree.proof(formulae, &mut proof);
            SearchResult::Proof(proof)
        }
        else {
            debug!("proof failed");
            SearchResult::Failed
        }
    }
}
