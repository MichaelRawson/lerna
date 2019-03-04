use smallvec::{smallvec, SmallVec};
use std::fmt;
use std::iter::FromIterator;
use unique::Id;

pub type List<T> = SmallVec<[T; 4]>;
pub use smallvec::smallvec as list;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Set<T> {
    elements: List<Id<T>>,
}

macro_rules! set {
    () => {
        Set::empty()
    };
    ($x:expr) => {
        Set::singleton($x)
    };
    ($($x:expr),*) => {
        list![$($x),*].into()
    };
}

impl<T> Set<T> {
    fn search(&self, element: &Id<T>) -> Result<usize, usize> {
        self.elements.binary_search_by_key(&Id::id(element), Id::id)
    }

    fn index(&self, element: &Id<T>) -> Option<usize> {
        self.search(element).ok()
    }

    pub unsafe fn from_sorted_list(sorted: List<Id<T>>) -> Self {
        Self { elements: sorted }
    }

    pub fn empty() -> Self {
        Self { elements: list![] }
    }

    pub fn singleton(element: Id<T>) -> Self {
        Self {
            elements: list![element],
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, element: &Id<T>) -> bool {
        self.index(element).is_some()
    }

    pub fn with(&self, element: Id<T>) -> Self {
        let index = self.search(&element).unwrap_or_else(|index| index);
        let (before, after) = self.elements.split_at(index);
        let mut new = List::with_capacity(self.len() + 1);
        new.extend(before.iter().cloned());
        new.push(element);
        new.extend(after.iter().cloned());
        unsafe { Self::from_sorted_list(new) }
    }

    pub fn without(&self, element: &Id<T>) -> Self {
        let new = self
            .elements
            .iter()
            .filter(|x| *x != element)
            .cloned()
            .collect::<List<_>>();
        unsafe { Self::from_sorted_list(new) }
    }

    pub fn union(&self, other: &Self) -> Self {
        let mut new = List::with_capacity(self.len() + other.len());
        new.extend(self.elements.clone());
        new.extend(other.elements.clone());
        new.into()
    }

    pub fn difference(&self, other: &Self) -> Self {
        let new = self
            .elements
            .iter()
            .filter(|x| !other.contains(x))
            .cloned()
            .collect::<List<_>>();
        unsafe { Self::from_sorted_list(new) }
    }
}

impl<T> From<List<Id<T>>> for Set<T> {
    fn from(mut list: List<Id<T>>) -> Self {
        list.sort_unstable_by_key(Id::id);
        list.dedup();
        Self { elements: list }
    }
}

impl<T> FromIterator<Id<T>> for Set<T> {
    fn from_iter<I>(iterator: I) -> Self
    where
        I: IntoIterator<Item = Id<T>>,
    {
        iterator.into_iter().collect::<List<_>>().into()
    }
}

impl<'a, T> IntoIterator for &'a Set<T> {
    type Item = <&'a List<Id<T>> as IntoIterator>::Item;
    type IntoIter = <&'a List<Id<T>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.elements).into_iter()
    }
}

impl<T: fmt::Debug> fmt::Debug for Set<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.elements.fmt(f)
    }
}
