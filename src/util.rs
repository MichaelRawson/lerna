use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

use rand::distributions::{Distribution, Uniform};
use rand::{thread_rng, Rng};

#[derive(Clone, Copy, PartialEq)]
pub struct Score(pub f64);

impl Score {
    pub fn new(score: f64) -> Self {
        assert!(score.is_finite());
        Score(score)
    }
}

impl Eq for Score {}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.0
                .partial_cmp(&other.0)
                .expect("score is not finite IEEE"),
        )
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub struct BiMap<A, B>
where
    A: Clone + Ord,
    B: Clone + Ord,
{
    forward: BTreeMap<A, B>,
    back: BTreeMap<B, A>,
}

impl<A, B> BiMap<A, B>
where
    A: Clone + Ord,
    B: Clone + Ord,
{
    pub fn new() -> Self {
        BiMap {
            forward: BTreeMap::new(),
            back: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, left: &A, right: &B) {
        self.forward.insert(left.clone(), right.clone());
        self.back.insert(right.clone(), left.clone());
    }

    pub fn forward(&self, left: &A) -> Option<&B> {
        self.forward.get(left)
    }

    pub fn back(&self, right: &B) -> Option<&A> {
        self.back.get(right)
    }

    pub fn len(&self) -> usize {
        self.forward.len()
    }
}

pub fn equal_choice<T>(items: &[T]) -> Option<&T> {
    let mut rng = thread_rng();
    rng.choose(items)
}

pub fn weighted_choice(scores: &[f64]) -> usize {
    let total: f64 = scores.iter().sum();
    let normalised = scores.iter().map(|x| x / total);
    let cumulative: Vec<Score> = normalised
        .scan(0.0, |running, x| {
            *running += x;
            Some(*running)
        }).map(Score::new)
        .collect();

    let mut rng = thread_rng();
    let range = Uniform::new(0.0, 1.0);
    let sample = Score::new(range.sample(&mut rng));
    cumulative
        .binary_search(&sample)
        .unwrap_or_else(|index| index)
}

pub fn assert_ownership<T>(ptr: Arc<T>) -> T
where
    T: Debug,
{
    Arc::try_unwrap(ptr).expect("assumed owned")
}
