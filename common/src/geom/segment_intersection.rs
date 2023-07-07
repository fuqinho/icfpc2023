use crate::{Point, Segment};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SegmentIntersection<T> {
    None,
    Point(Point<T>),
    Segment(Segment<T>),
}

impl<T> SegmentIntersection<T> {
    pub fn must_point(self) -> Point<T> {
        match self {
            Self::Point(p) => p,
            _ => panic!("must be a point"),
        }
    }

    pub fn must_segment(self) -> Segment<T> {
        match self {
            Self::Segment(s) => s,
            _ => panic!("must be a segment"),
        }
    }
}
