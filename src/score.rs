use std::cmp::Ordering;
use std::fmt;
use std::ops::{AddAssign, Div};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Score(pub f32);

impl From<f32> for Score {
    fn from(score: f32) -> Self {
        assert!(score.is_finite());
        Self(score)
    }
}

impl From<usize> for Score {
    fn from(score: usize) -> Self {
        (score as f32).into()
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Div for Score {
    type Output = Score;

    fn div(self, other: Self) -> Score {
        (self.0 / other.0).into()
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
