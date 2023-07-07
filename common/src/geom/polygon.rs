use std::{borrow::Borrow, collections::BTreeSet, fmt::Display, ops::Index};

use num::traits::{NumRef, RefNum};

use crate::{BoundingBox, Point, RefSegment, SegmentIntersection, CCW};

pub type Polygon<T> = GenericPolygon<T, Point<T>>;
pub type RefPolygon<'a, T> = GenericPolygon<T, &'a Point<T>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericPolygon<T, P> {
    pub ps: Vec<P>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, P: Display> Display for GenericPolygon<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.ps
                .iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolygonContains {
    On,
    In,
    Out,
}

impl<T, P: Borrow<Point<T>>> GenericPolygon<T, P> {
    pub fn new(ps: Vec<P>) -> Self {
        Self {
            ps,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.ps.len()
    }

    pub fn segments(&self) -> Vec<RefSegment<T>> {
        let mut ss = vec![];
        for i in 0..self.ps.len() {
            ss.push(RefSegment::new(
                self.ps[i].borrow(),
                self.ps[(i + 1) % self.ps.len()].borrow(),
            ));
        }
        ss
    }
}

impl<T, P> Index<usize> for GenericPolygon<T, P> {
    type Output = P;

    fn index(&self, index: usize) -> &Self::Output {
        &self.ps[index]
    }
}

impl<T: Clone, P: Borrow<Point<T>>> GenericPolygon<T, P> {
    pub fn cloned(&self) -> GenericPolygon<T, Point<T>> {
        GenericPolygon::new(self.ps.iter().map(|p| p.borrow().clone()).collect())
    }
}

impl<T: NumRef + Ord + Clone, P: Borrow<Point<T>>> GenericPolygon<T, P>
where
    for<'r> &'r T: RefNum<T>,
{
    pub fn area(&self) -> T {
        let mut area = T::zero();
        for i in 0..self.ps.len() {
            let p = self.ps[i].borrow();
            let q = self.ps[(i + 1) % self.ps.len()].borrow();
            area = area + p.cross(q);
        }
        area / (T::one() + T::one())
    }

    pub fn is_convex(&self) -> bool {
        let mut has_ccw = false;
        let mut has_cw = false;

        let n = self.ps.len();
        for i in 0..n {
            let p = self.ps[i].borrow();
            let q = self.ps[(i + 1) % n].borrow();
            let r = self.ps[(i + 2) % n].borrow();
            match p.ccw(q, r) {
                CCW::CounterClockwise => has_ccw = true,
                CCW::Clockwise => has_cw = true,
                _ => (),
            }
        }
        !(has_ccw & has_cw)
    }

    pub fn contains(&self, p: &Point<T>) -> PolygonContains {
        self.sub(p).contains_origin()
    }

    pub fn contains_origin(&self) -> PolygonContains {
        let n = self.ps.len();

        let mut inside = false;

        let zero = T::zero();

        for i in 0..n {
            let p = self.ps[i].borrow();
            let q = self.ps[(i + 1) % n].borrow();

            if p.ccw(q, &Point::origin()) == CCW::OnSegment {
                return PolygonContains::On;
            }

            if p.y <= zero && q.y > zero || p.y > zero && q.y <= zero {
                let x_on_axis = &p.x - &p.y * (&q.x - &p.x) / (&q.y - &p.y);

                if x_on_axis.is_zero() {
                    return PolygonContains::On;
                }
                if x_on_axis > zero {
                    inside = !inside;
                }
            }
        }

        if inside {
            PolygonContains::In
        } else {
            PolygonContains::Out
        }
    }

    fn sub(&self, q: &Point<T>) -> GenericPolygon<T, Point<T>> {
        GenericPolygon::new(self.ps.iter().map(|p| p.borrow() - q).collect())
    }

    // In: There's a point on the segment that is in the polygon
    // On: There's a point on the segment that is on the polygon
    // Out: There's no point on the segment that is on or in the polygon
    pub fn contains_segment(&self, s: RefSegment<T>) -> PolygonContains {
        if self.contains(s.from()) == PolygonContains::In {
            return PolygonContains::In;
        }
        if self.contains(s.to()) == PolygonContains::In {
            return PolygonContains::In;
        }

        let mut intersections = BTreeSet::<Point<T>>::new();

        for i in 0..self.ps.len() {
            let p = self.ps[i].borrow();
            let q = self.ps[(i + 1) % self.ps.len()].borrow();

            match s.intersection(RefSegment::new(p, q)) {
                SegmentIntersection::None => (),
                SegmentIntersection::Point(c) => {
                    intersections.insert(c);
                }
                SegmentIntersection::Segment(t) => {
                    intersections.insert(t.from);
                    intersections.insert(t.to);
                }
            }
        }

        if intersections.is_empty() {
            return PolygonContains::Out;
        }

        let is = intersections.into_iter().collect::<Vec<_>>();

        for i in 0..is.len() - 1 {
            let p = (&is[i] + &is[i + 1]) / (T::one() + T::one());

            if self.contains(&p) == PolygonContains::In {
                return PolygonContains::In;
            }
        }

        PolygonContains::On
    }

    // In: There's a point in the rhs that is in the polygon
    // On: There's a point on the rhs that is on the polygon
    // Out: There's no point on the rhs that is on the polygon
    pub fn contains_polygon(&self, other: &Self) -> PolygonContains {
        let mut out = true;

        for i in 0..other.ps.len() {
            let p = other.ps[i].borrow();
            let q = other.ps[(i + 1) % other.ps.len()].borrow();

            match self.contains_segment(RefSegment::new(p, q)) {
                PolygonContains::On => out = false,
                PolygonContains::In => return PolygonContains::In,
                PolygonContains::Out => (),
            }
        }

        for i in 0..self.ps.len() {
            let p = self.ps[i].borrow();
            let q = self.ps[(i + 1) % self.ps.len()].borrow();

            match other.contains_segment(RefSegment::new(p, q)) {
                PolygonContains::On => out = false,
                PolygonContains::In => return PolygonContains::In,
                PolygonContains::Out => (),
            }
        }

        if out {
            return PolygonContains::Out;
        }

        // Returns In if self and other represents the same polygon.
        for p in other.ps.iter() {
            if self.contains(p.borrow()) == PolygonContains::Out {
                return PolygonContains::On;
            }
        }
        for p in self.ps.iter() {
            if other.contains(p.borrow()) == PolygonContains::Out {
                return PolygonContains::On;
            }
        }

        PolygonContains::In
    }

    pub fn bounding_box(&self) -> BoundingBox<T> {
        let mut min_x = &self.ps[0].borrow().x;
        let mut min_y = &self.ps[0].borrow().y;
        let mut max_x = &self.ps[0].borrow().x;
        let mut max_y = &self.ps[0].borrow().y;

        for p in self.ps.iter().skip(1) {
            let p = p.borrow();
            if p.x < *min_x {
                min_x = &p.x;
            }
            if p.y < *min_y {
                min_y = &p.y;
            }
            if p.x > *max_x {
                max_x = &p.x;
            }
            if p.y > *max_y {
                max_y = &p.y;
            }
        }

        BoundingBox::new(
            Point::new(min_x.clone(), min_y.clone()),
            Point::new(max_x.clone(), max_y.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use num_rational::BigRational;

    use crate::{Point, Polygon, PolygonContains, Segment};

    #[test]
    fn test_polygon() {
        use num_rational::BigRational;

        use super::*;

        let p = Point::new(
            BigRational::new(0.into(), 1.into()),
            BigRational::new(0.into(), 1.into()),
        );
        let q = Point::new(
            BigRational::new(1.into(), 1.into()),
            BigRational::new(0.into(), 1.into()),
        );
        let r = Point::new(
            BigRational::new(1.into(), 1.into()),
            BigRational::new(1.into(), 1.into()),
        );
        let s = Point::new(
            BigRational::new(0.into(), 1.into()),
            BigRational::new(1.into(), 1.into()),
        );

        let polygon = GenericPolygon::new(vec![&p, &q, &r, &s]);
        assert_eq!(polygon.area(), BigRational::new(1.into(), 1.into()));

        let polygon = GenericPolygon::new(vec![p, q, r, s]);
        assert_eq!(polygon.area(), BigRational::new(1.into(), 1.into()));
    }

    #[test]
    fn test_contains_segment() {
        let polygon = Polygon::new(
            vec!["0,0", "4,0", "4,4", "0,4"]
                .into_iter()
                .map(|s| s.parse::<Point<BigRational>>().unwrap())
                .collect(),
        );
        for (tc, want) in [
            ("0,0 4,4", PolygonContains::In),
            ("0,0 4,0", PolygonContains::On),
            ("5,0 6,0", PolygonContains::Out),
            ("1,1 3,3", PolygonContains::In),
            ("0,0 0,5", PolygonContains::On),
            ("2,2 6,2", PolygonContains::In),
            ("4,2 6,2", PolygonContains::On),
        ] {
            let s = tc.parse::<Segment<BigRational>>().unwrap();
            assert_eq!(polygon.contains_segment(s.as_ref()), want);
        }
    }
}
