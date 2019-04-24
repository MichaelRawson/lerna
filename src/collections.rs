use smallvec::SmallVec;
use std::fmt;
use std::iter::FromIterator;
use unique::Id;

pub type IdList<T> = SmallVec<[Id<T>; 8]>;

macro_rules! idlist {
    () => {
        IdList::new()
    };
    ($($x:expr),*) => {{
        let mut new = crate::collections::IdList::new();
        $(new.push($x);)*
        new
    }}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct IdSet<T> {
    elements: IdList<T>,
}

macro_rules! idset {
    () => {
        crate::collections::IdSet::empty()
    };
    ($x:expr) => {
        crate::collections::IdSet::singleton($x)
    };
    ($($x:expr),*) => {{
        let mut new = crate::collections::IdList::new();
        $(new.push($x);)*
        let set: crate::collections::IdSet<_> = new.into();
        set
    }};
}

impl<T> IdSet<T> {
    fn search(&self, element: &Id<T>) -> Result<usize, usize> {
        self.elements.binary_search_by_key(&Id::id(element), Id::id)
    }

    pub unsafe fn from_sorted_list(mut sorted: IdList<T>) -> Self {
        sorted.dedup();
        Self { elements: sorted }
    }

    pub fn empty() -> Self {
        Self {
            elements: idlist![],
        }
    }

    pub fn singleton(element: Id<T>) -> Self {
        Self {
            elements: idlist![element],
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, element: &Id<T>) -> bool {
        self.elements.contains(element)
    }

    pub fn with(&self, element: Id<T>) -> Self {
        let index = self.search(&element).unwrap_or_else(|index| index);
        let (before, after) = self.elements.split_at(index);
        let mut new = IdList::with_capacity(self.len() + 1);
        new.extend(before.iter().cloned());
        new.push(element);
        new.extend(after.iter().cloned());
        unsafe { Self::from_sorted_list(new) }
    }

    pub fn without(&self, element: &Id<T>) -> Self {
        if let Ok(index) = self.search(&element) {
            let (before, after) = self.elements.split_at(index);
            let after = &after[1..];
            let mut new = IdList::with_capacity(self.len() - 1);
            new.extend(before.iter().cloned());
            new.extend(after.iter().cloned());
            unsafe { Self::from_sorted_list(new) }
        } else {
            Self {
                elements: self.elements.clone(),
            }
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        let mut new = IdList::with_capacity(self.len() + other.len());
        new.extend(self.elements.clone());
        new.extend(other.elements.clone());
        new.into()
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.into_iter().any(|x| other.contains(x))
    }

    pub fn difference(&self, other: &Self) -> Self {
        let new = self
            .elements
            .iter()
            .filter(|x| !other.contains(x))
            .cloned()
            .collect::<IdList<_>>();
        unsafe { Self::from_sorted_list(new) }
    }

    pub fn pairs(&self) -> impl Iterator<Item = (&Id<T>, &Id<T>)> {
        let lefts = self.into_iter();
        let rights = self.into_iter().skip(1);
        lefts.zip(rights)
    }
}

impl<'a, T: 'a + Clone> IdSet<T> {
    pub fn combine_overlapping<I: IntoIterator<Item = &'a IdSet<T>>>(
        iter: I,
    ) -> Vec<IdSet<T>> {
        let mut classes: Vec<IdSet<_>> = vec![];
        for class in iter {
            let mut found = false;
            for existing in &mut classes {
                if class.overlaps(existing) {
                    existing.extend(class.into_iter().cloned());
                    found = true;
                    break;
                }
            }
            if !found {
                classes.push(class.clone());
            }
        }
        classes
    }
}

impl<T> Default for IdSet<T> {
    fn default() -> Self {
        IdSet::empty()
    }
}

impl<T> From<IdList<T>> for IdSet<T> {
    fn from(mut list: IdList<T>) -> Self {
        list.sort_unstable_by_key(Id::id);
        list.dedup();
        Self { elements: list }
    }
}

impl<T> AsRef<IdList<T>> for IdSet<T> {
    fn as_ref(&self) -> &IdList<T> {
        &self.elements
    }
}

impl<T> Extend<Id<T>> for IdSet<T> {
    fn extend<I: IntoIterator<Item = Id<T>>>(&mut self, iter: I) {
        self.elements.extend(iter);
        self.elements.sort_unstable_by_key(Id::id);
        self.elements.dedup();
    }
}

impl<T> FromIterator<Id<T>> for IdSet<T> {
    fn from_iter<I>(iterator: I) -> Self
    where
        I: IntoIterator<Item = Id<T>>,
    {
        iterator.into_iter().collect::<IdList<_>>().into()
    }
}

impl<T> IntoIterator for IdSet<T> {
    type Item = Id<T>;
    type IntoIter = <IdList<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a IdSet<T> {
    type Item = &'a Id<T>;
    type IntoIter = <&'a IdList<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.as_ref().iter()
    }
}

impl<T: fmt::Debug> fmt::Debug for IdSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.elements.fmt(f)
    }
}
