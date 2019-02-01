use std::fmt;
use std::iter::FromIterator;
use unique::Id;

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Set<T> {
    elements: Box<[Id<T>]>,
}

macro_rules! set {
    () => {
        unsafe { Set::new_from_sorted_boxed_slice(Box::new([])) }
    };
    ($x:expr) => {
        unsafe { Set::new_from_sorted_boxed_slice(Box::new([$x; 1])) }
    };
    ($($x:expr),*) => {
        Set::new_from_vec(vec![$($x),*])
    };
}

impl<T> Set<T> {
    fn search(&self, element: &Id<T>) -> Result<usize, usize> {
        self.elements.binary_search_by_key(&Id::id(element), Id::id)
    }

    fn index(&self, element: &Id<T>) -> Option<usize> {
        self.search(element).ok()
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

    pub unsafe fn new_from_sorted_boxed_slice(elements: Box<[Id<T>]>) -> Self {
        Self { elements }
    }

    pub fn new_from_vec(mut raw: Vec<Id<T>>) -> Self {
        raw.sort_unstable_by_key(Id::id);
        raw.dedup();
        let elements = raw.into_boxed_slice();
        unsafe { Self::new_from_sorted_boxed_slice(elements) }
    }

    pub fn with(&self, element: Id<T>) -> Self {
        let index = self.search(&element).unwrap_or_else(|index| index);
        let (before, after) = self.elements.split_at(index);
        let mut new = Vec::with_capacity(self.len() + 1);
        new.extend_from_slice(before);
        new.push(element);
        new.extend_from_slice(after);
        let new = new.into_boxed_slice();
        unsafe { Self::new_from_sorted_boxed_slice(new) }
    }

    pub fn union(&self, other: &Self) -> Self {
        let mut new = Vec::with_capacity(self.len() + other.len());
        new.extend_from_slice(&self.elements);
        new.extend_from_slice(&other.elements);
        Self::new_from_vec(new)
    }

    pub fn difference(&self, other: &Self) -> Self {
        let new = self
            .elements
            .iter()
            .filter(|x| !other.contains(x))
            .cloned()
            .collect::<Vec<_>>();
        let new = new.into_boxed_slice();
        unsafe { Self::new_from_sorted_boxed_slice(new) }
    }
}

impl<T> FromIterator<Id<T>> for Set<T> {
    fn from_iter<I>(iterator: I) -> Self
    where
        I: IntoIterator<Item = Id<T>>,
    {
        let raw: Vec<_> = iterator.into_iter().collect();
        Set::new_from_vec(raw)
    }
}

impl<'a, T> IntoIterator for &'a Set<T> {
    type Item = <&'a [Id<T>] as IntoIterator>::Item;
    type IntoIter = <&'a [Id<T>] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<T: fmt::Debug> fmt::Debug for Set<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.elements.fmt(f)
    }
}
