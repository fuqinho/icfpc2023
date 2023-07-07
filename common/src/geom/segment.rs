use std::{borrow::Borrow, fmt::Display, str::FromStr};

use num::traits::{NumRef, RefNum};

use crate::{Point, SegmentIntersection, Transform, CCW};

pub type Segment<T> = GenericSegment<T, Point<T>>;
pub type RefSegment<'a, T> = GenericSegment<T, &'a Point<T>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericSegment<T, P: Borrow<Point<T>>> {
    pub from: P,
    pub to: P,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, P: Borrow<Point<T>>> GenericSegment<T, P> {
    pub fn new(from: P, to: P) -> Self {
        Self {
            from,
            to,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn from(&self) -> &Point<T> {
        self.from.borrow()
    }

    pub fn to(&self) -> &Point<T> {
        self.to.borrow()
    }

    pub fn as_ref(&self) -> GenericSegment<T, &Point<T>> {
        GenericSegment::new(self.from(), self.to())
    }

    pub fn reversed(self) -> Self {
        Self::new(self.to, self.from)
    }
}

impl<T: Clone, P: Borrow<Point<T>>> GenericSegment<T, P> {
    pub fn cloned(&self) -> GenericSegment<T, Point<T>> {
        GenericSegment::new(self.from().clone(), self.to().clone())
    }
}

impl<T: NumRef + Ord + Clone + PartialOrd, P: Borrow<Point<T>>> GenericSegment<T, P>
where
    for<'r> &'r T: RefNum<T>,
{
    // https://onlinejudge.u-aizu.ac.jp/courses/library/4/CGL/all/CGL_2_B
    // cf. https://bakamono1357.hatenablog.com/entry/2020/04/29/025320
    pub fn intersect<U: Borrow<Self>>(&self, t: U) -> bool {
        let s = self;
        let t = t.borrow();

        s.from().ccw(s.to(), t.from()) as i8 * s.from().ccw(s.to(), t.to()) as i8 <= 0
            && t.from().ccw(t.to(), s.from()) as i8 * t.from().ccw(t.to(), s.to()) as i8 <= 0
    }

    // Directional vector from `from` to `to`.
    pub fn dir(&self) -> Point<T> {
        self.to() - self.from()
    }

    // Project: https://onlinejudge.u-aizu.ac.jp/problems/CGL_1_A
    pub fn project<U: Borrow<Point<T>>>(&self, p: U) -> Point<T> {
        let line = self;
        let v = line.dir();
        let t = p.borrow().dot(&v) / v.dot(&v);
        line.from() + v * t
    }

    // Reflect: https://onlinejudge.u-aizu.ac.jp/problems/CGL_1_B
    pub fn reflect<U: Borrow<Point<T>>>(&self, p: U) -> Point<T> {
        let q = self.project(p.borrow());
        &q + (&q - p)
    }

    pub fn parallel<U: Borrow<Self>>(&self, t: U) -> bool {
        self.dir().cross(t.borrow().dir()).is_zero()
    }

    pub fn orthogonal<U: Borrow<Self>>(&self, t: U) -> bool {
        self.dir().dot(t.borrow().dir()).is_zero()
    }

    pub fn contains<U: Borrow<Point<T>>>(&self, p: U) -> bool {
        self.from().ccw(self.to(), p.borrow()) == CCW::OnSegment
    }

    pub fn intersection<U: Borrow<Self>>(&self, t: U) -> SegmentIntersection<T> {
        let s = self;
        let t = t.borrow();

        if !s.intersect(t) {
            return SegmentIntersection::None;
        }

        if !s.parallel(t) {
            let d1 = s.dir().cross(t.dir());
            let d2 = s.dir().cross(s.to() - t.from());
            return SegmentIntersection::Point(t.from() + t.dir() * (d2 / d1));
        }

        // Parallel and intersect.
        let a = s.from();
        let b = s.to();
        let c = t.from();
        let d = t.to();

        let c_ccw = a.ccw(b, c);
        let d_ccw = a.ccw(b, d);

        let (from, to) = if c_ccw == CCW::OnSegment {
            if d_ccw == CCW::OnSegment {
                (c, d)
            } else if d_ccw == CCW::OnLineBack {
                (a, c)
            } else {
                (c, b)
            }
        } else if c_ccw == CCW::OnLineFront {
            if d_ccw == CCW::OnSegment {
                (d, b)
            } else {
                (a, b)
            }
        } else {
            if d_ccw == CCW::OnSegment {
                (a, d)
            } else {
                (a, b)
            }
        };
        return SegmentIntersection::Segment(GenericSegment::new(from.clone(), to.clone()));
    }

    pub fn distance2<U: Borrow<Point<T>>>(&self, p: U) -> T {
        let p = p.borrow();

        let c = self.project(p);
        if self.contains(&c) {
            return p.distance2(&c);
        }
        p.distance2(self.from()).min(p.distance2(self.to()))
    }

    pub fn distance2_s<U: Borrow<GenericSegment<T, P>>>(&self, s: U) -> T {
        let s = s.borrow();

        if self.intersect(s) {
            return T::zero();
        }

        self.distance2(s.from())
            .min(self.distance2(s.to()))
            .min(s.distance2(self.from()))
            .min(s.distance2(self.to()))
    }

    pub fn must_transform_onto<U: Borrow<GenericSegment<T, P>>>(&self, t: U) -> Transform<T> {
        let s1 = self;
        let s2 = t.borrow();

        let s_dir = s1.dir();
        let t_dir = s2.dir();

        if s_dir.norm2() != t_dir.norm2() {
            panic!("trans_rot_onto: not same length");
        }

        let rot = t_dir.div_complex(s_dir);

        Transform::new(s1.from().clone(), rot, s2.from().clone())
    }
}

impl<T: FromStr + Clone> FromStr for Segment<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.trim().split_whitespace();
        let p1 = iter.next().unwrap().parse()?;
        let p2 = iter.next().unwrap().parse()?;
        Ok(Segment::new(p1, p2))
    }
}

impl<T: Display, P: Borrow<Point<T>>> Display for GenericSegment<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.from(), self.to())
    }
}

impl<T: num::ToPrimitive> Segment<T> {
    pub fn to_f64(&self) -> Option<Segment<f64>> {
        Some(Segment::new(self.from().to_f64()?, self.to().to_f64()?))
    }
}

#[cfg(test)]
mod tests {
    use crate::SegmentIntersection;

    #[test]
    fn test_segment() {
        use crate::GenericSegment;
        use crate::Point;

        let p00 = Point::new(0, 0);
        let p01 = Point::new(0, 1);
        let p02 = Point::new(0, 2);
        let p10 = Point::new(1, 0);
        let p11 = Point::new(1, 1);

        let _ = GenericSegment::new(p00.clone(), p01.clone());

        let s1 = GenericSegment::new(&p00, &p01);
        let s2 = GenericSegment::new(&p00, &p02);
        let s3 = GenericSegment::new(&p10, &p11);
        let s4 = GenericSegment::new(&p01, &p11);

        assert_eq!(s1.intersect(s2), true);
        assert_eq!(s1.intersect(s3), false);
        assert_eq!(s3.intersect(s4), true);

        assert_eq!(s1.parallel(s2), true);
        assert_eq!(s1.parallel(s3), true);
        assert_eq!(s1.parallel(s4), false);

        assert_eq!(s1.orthogonal(s2), false);
        assert_eq!(s1.orthogonal(s3), false);
        assert_eq!(s1.orthogonal(s4), true);

        assert_eq!(s1.contains(p00), true);
        assert_eq!(s1.contains(p01), true);
        assert_eq!(s1.contains(p02), false);

        assert_eq!(s1.intersection(&s2).must_segment(), s1.cloned());
        assert_eq!(s1.intersection(&s3), SegmentIntersection::None);
        assert_eq!(s1.intersection(&s4).must_point(), p01);

        assert_eq!(s1.distance2_s(s2), 0);
        assert_eq!(s1.distance2_s(s3), 1);
        assert_eq!(s1.distance2_s(s4), 0);
    }
}
