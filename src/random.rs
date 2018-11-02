use rand::distributions::{Distribution, Uniform};
use rand::{thread_rng, Rng};

use crate::score::Score;

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
