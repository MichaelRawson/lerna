use std::cmp::Ordering;
use std::fmt;
use std::iter::Sum;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Score(pub f32);

impl Score {
    pub fn new(score: f32) -> Self {
        assert!(score.is_finite());
        Score(score)
    }
}

impl Eq for Score {}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.partial_cmp(&other.0).unwrap())
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl fmt::Debug for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Sum for Score {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total = 0.0;
        for s in iter {
            total += s.0;
        }
        Score::new(total)
    }
}
