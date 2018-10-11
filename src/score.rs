use std::cmp::Ordering;

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
