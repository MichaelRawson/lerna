use im;

use std::collections::BTreeMap;

pub type Set<T> = im::OrdSet<T>;
macro_rules! set {
    ($($x:tt)*) => {ordset![$($x)*]}
}
pub type Map<K, V> = im::OrdMap<K, V>;
macro_rules! map {
    ($($x:tt)*) => {ordmap![$($x)*]}
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
