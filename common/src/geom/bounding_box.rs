use num::{traits::NumRef, ToPrimitive};

use crate::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundingBox<T>(pub Point<T>, pub Point<T>);

impl<T> BoundingBox<T> {
    pub fn new(min: Point<T>, max: Point<T>) -> Self {
        Self(min, max)
    }
}

impl<T: NumRef + PartialOrd> BoundingBox<T> {
    pub fn strictly_overlap(&self, other: &Self) -> bool {
        let max = |a, b| if a < b { b } else { a };
        let min = |a, b| if a > b { b } else { a };

        let x_start = max(&self.0.x, &other.0.x);
        let x_end = min(&self.1.x, &other.1.x);

        let y_start = max(&self.0.y, &other.0.y);
        let y_end = min(&self.1.y, &other.1.y);

        x_start < x_end && y_start < y_end
    }

    pub fn to_f64(&self) -> Option<BoundingBox<f64>>
    where
        T: ToPrimitive,
    {
        BoundingBox::new(self.0.to_f64()?, self.1.to_f64()?).into()
    }
}
