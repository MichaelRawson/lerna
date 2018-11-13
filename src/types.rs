use im;

use std::collections::HashMap;
use std::hash::Hash;

pub type Set<T> = im::OrdSet<T>;
macro_rules! set {
    [$($x:tt)*] => {ordset![$($x)*]}
}

pub type Map<K, V> = im::OrdMap<K, V>;
macro_rules! map {
    [$($x:tt)*] => {ordmap![$($x)*]}
}

pub struct BiMap<A, B>
where
    A: Clone + Eq + Hash,
    B: Clone + Eq + Hash,
{
    forward: HashMap<A, B>,
    back: HashMap<B, A>,
}

impl<A, B> BiMap<A, B>
where
    A: Clone + Eq + Hash,
    B: Clone + Eq + Hash,
{
    pub fn new() -> Self {
        BiMap {
            forward: HashMap::new(),
            back: HashMap::new(),
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
