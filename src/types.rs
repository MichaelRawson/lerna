use im;
use smallvec;

use std::collections::HashMap;
use std::hash::Hash;

pub type List<T> = smallvec::SmallVec<[T; 8]>;
macro_rules! list {
    [$($x:tt)*] => {(smallvec![$($x)*]::<List<_>>)}
}

pub type Set<T> = im::HashSet<T>;
macro_rules! set {
    [$($x:tt)*] => {hashset![$($x)*]}
}

pub type Map<K, V> = im::HashMap<K, V>;
macro_rules! map {
    [$($x:tt)*] => {hashmap![$($x)*]}
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
