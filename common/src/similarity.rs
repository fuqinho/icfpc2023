use std::collections::HashMap;

use euclid::default::Point2D;

use crate::{Problem, Solution};

pub fn similarity(problem: &Problem, s1: &Solution, s2: &Solution) -> f64 {
    // taste -> [musicians]
    let mut musicians1: HashMap<usize, Vec<Point2D<f64>>> = HashMap::new();
    let mut musicians2: HashMap<usize, Vec<Point2D<f64>>> = HashMap::new();
    for (m, (p1, p2)) in problem
        .musicians
        .iter()
        .zip(s1.placements.iter().zip(s2.placements.iter()))
    {
        if let Some(places) = musicians1.get_mut(m) {
            places.push(p1.position);
        } else {
            musicians1.insert(*m, vec![p1.position]);
        }
        if let Some(places) = musicians2.get_mut(m) {
            places.push(p2.position);
        } else {
            musicians2.insert(*m, vec![p2.position]);
        }
    }
    0.
}
