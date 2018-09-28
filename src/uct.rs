pub fn uct(score: f64, parent_visits: usize, child_visits: usize) -> f64 {
    #[allow(non_snake_case)]
    let N = parent_visits as f64;
    let n = child_visits as f64;
    score + (2.0 * N.ln() / n).sqrt()
}
