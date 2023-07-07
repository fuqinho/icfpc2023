use std::{borrow::Borrow, fmt::Display};

use num::traits::{NumRef, RefNum};

use crate::Point;

// Transformation of subtracting, rotating, and adding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Transform<T> {
    pub sub: Point<T>,
    // The norm of rot must be 1.
    // Use mul_complex for transformation.
    pub rot: Point<T>,
    pub add: Point<T>,
}

impl<T: Display> Display for Transform<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sub: {}, rot: {}, add: {}", self.sub, self.rot, self.add)
    }
}

impl<T> Transform<T> {
    pub fn new(sub: Point<T>, rot: Point<T>, add: Point<T>) -> Self {
        Self { sub, rot, add }
    }
}

impl<T: NumRef + Ord + PartialOrd> Transform<T>
where
    for<'r> &'r T: RefNum<T>,
{
    pub fn transform<U: Borrow<Point<T>>>(&self, p: U) -> Point<T> {
        (p.borrow() - &self.sub).mul_complex(&self.rot) + &self.add
    }
}
