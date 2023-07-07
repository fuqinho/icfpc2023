use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

use num::{
    traits::{NumRef, RefNum},
    Num,
};

use crate::CCW;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Clone> Point<T> {
    /// Returns (v, v).
    pub fn splat(v: T) -> Self {
        Self::new(v.clone(), v)
    }
}

impl<T: Num> Point<T> {
    pub fn zero() -> Self {
        Self::new(num::zero(), num::zero())
    }

    pub fn origin() -> Self {
        Self::zero()
    }

    pub fn one() -> Self {
        Self::new(num::one(), num::one())
    }

    pub fn min_xy(self) -> T
    where
        T: Ord,
    {
        T::min(self.x, self.y)
    }

    pub fn max_xy(self) -> T
    where
        T: Ord,
    {
        T::max(self.x, self.y)
    }
}

macro_rules! add_sub {
    (impl $imp:ident, $method:ident, $op:tt) => {
        // p + q, p + &q
        impl<T: NumRef, U: Borrow<Self>> $imp<U> for Point<T>
        where
            for<'r> &'r T: RefNum<T>,
        {
            type Output = Point<T>;

            fn $method(self, rhs: U) -> Self::Output {
                Self::Output::new(self.x $op &rhs.borrow().x, self.y $op &rhs.borrow().y)
            }
        }

        // &p + q, &p + &q
        impl<T: NumRef, U: Borrow<Point<T>>> $imp<U> for &Point<T>
        where
            for<'r> &'r T: RefNum<T>,
        {
            type Output = Point<T>;

            fn $method(self, rhs: U) -> Self::Output {
                Self::Output::new(&self.x $op &rhs.borrow().x, &self.y $op &rhs.borrow().y)
            }
        }
    };
}

add_sub!(impl Add, add, +);
add_sub!(impl Sub, sub, -);

macro_rules! mul_div {
    (impl $imp:ident, $method:ident, $op:tt) => {
        // p * d, p * &d
        impl<T: NumRef, U: Borrow<T>> $imp<U> for Point<T>
        where
            for<'r> &'r T: RefNum<T>,
        {
            type Output = Point<T>;

            fn $method(self, rhs: U) -> Self::Output {
                Self::Output::new(self.x $op rhs.borrow(), self.y $op rhs.borrow())
            }
        }

        // &p * d, &p * &d
        impl<T: NumRef, U: Borrow<T>> $imp<U> for &Point<T>
        where
            for<'r> &'r T: RefNum<T>,
        {
            type Output = Point<T>;

            fn $method(self, rhs: U) -> Self::Output {
                Self::Output::new(&self.x $op rhs.borrow(), &self.y $op rhs.borrow())
            }
        }
    };
}

mul_div!(impl Mul, mul, *);
mul_div!(impl Div, div, /);

impl<T: NumRef + Ord + PartialOrd> Point<T>
where
    for<'r> &'r T: RefNum<T>,
{
    pub fn dot<U: Borrow<Point<T>>>(&self, rhs: U) -> T {
        &self.x * &rhs.borrow().x + &self.y * &rhs.borrow().y
    }

    pub fn cross<U: Borrow<Point<T>>>(&self, rhs: U) -> T {
        &self.x * &rhs.borrow().y - &self.y * &rhs.borrow().x
    }

    pub fn mul_complex<U: Borrow<Point<T>>>(&self, rhs: U) -> Point<T> {
        let x = &self.x * &rhs.borrow().x - &self.y * &rhs.borrow().y;
        let y = &self.x * &rhs.borrow().y + &self.y * &rhs.borrow().x;
        Point::new(x, y)
    }

    pub fn inverse(&self) -> Point<T> {
        let d = &self.x * &self.x + &self.y * &self.y;
        Point::new(&self.x / &d, T::zero() - &self.y / d)
    }

    pub fn div_complex<U: Borrow<Point<T>>>(&self, rhs: U) -> Point<T> {
        self.mul_complex(rhs.borrow().inverse())
    }

    // CCW: https://onlinejudge.u-aizu.ac.jp/problems/CGL_1_C
    // c.f. https://bakamono1357.hatenablog.com/entry/2020/04/29/025320
    pub fn ccw<U: Borrow<Point<T>>, V: Borrow<Point<T>>>(&self, b: U, c: V) -> CCW {
        let a = self;
        let b = b.borrow() - a;
        let c = c.borrow() - a;
        if b.cross(&c) > T::zero() {
            CCW::CounterClockwise
        } else if b.cross(&c) < T::zero() {
            CCW::Clockwise
        } else if b.dot(&c) < T::zero() {
            CCW::OnLineBack
        } else if b.norm2() < c.norm2() {
            CCW::OnLineFront
        } else {
            CCW::OnSegment
        }
    }

    pub fn distance2<U: Borrow<Self>>(&self, b: U) -> T {
        (self - b.borrow()).norm2()
    }

    pub fn norm2(&self) -> T {
        self.dot(self)
    }

    pub fn argcmp(self: &Self, other: &Self) -> Ordering {
        let Point { x: x0, y: y0 } = self;
        let Point { x: x1, y: y1 } = other;

        let zero = &T::zero();

        ((y0, x0) < (zero, zero))
            .cmp(&((y1, x1) < (zero, zero)))
            .then_with(|| (x1 * y0).cmp(&(x0 * y1)))
    }

    pub fn argcmp_around<U: Borrow<Self>>(origin: Self) -> impl Fn(&U, &U) -> Ordering {
        move |p, q| (p.borrow() - &origin).argcmp(&(q.borrow() - &origin))
    }
}

impl<T: FromStr + Clone> FromStr for Point<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let xy = s
            .split(",")
            .map(T::from_str)
            .collect::<Result<Vec<T>, _>>()?;

        Ok(Self::new(xy[0].clone(), xy[1].clone()))
    }
}

impl<T: Display> Display for Point<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl<T: num::ToPrimitive> Point<T> {
    pub fn to_f64(&self) -> Option<Point<f64>> {
        Point::new(self.x.to_f64()?, self.y.to_f64()?).into()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_add_sub() {
        use num_rational::BigRational;

        use super::*;

        let p = Point::new(
            BigRational::new(1.into(), 1.into()),
            BigRational::new(2.into(), 1.into()),
        );
        let q = Point::new(
            BigRational::new(3.into(), 1.into()),
            BigRational::new(4.into(), 1.into()),
        );

        assert_eq!(
            &p + &q,
            Point::new(
                BigRational::new(4.into(), 1.into()),
                BigRational::new(6.into(), 1.into())
            )
        );

        assert_eq!(
            &q - &p,
            Point::new(
                BigRational::new(2.into(), 1.into()),
                BigRational::new(2.into(), 1.into())
            )
        );

        assert_eq!(
            p.clone() + q.clone(),
            Point::new(
                BigRational::new(4.into(), 1.into()),
                BigRational::new(6.into(), 1.into())
            )
        );

        assert_eq!(
            &p + q.clone(),
            Point::new(
                BigRational::new(4.into(), 1.into()),
                BigRational::new(6.into(), 1.into())
            )
        );

        assert_eq!(
            p + &q,
            Point::new(
                BigRational::new(4.into(), 1.into()),
                BigRational::new(6.into(), 1.into())
            )
        );
    }

    #[test]
    fn test_mul_div() {
        use num_rational::BigRational;

        use super::*;

        let p = Point::new(
            BigRational::new(1.into(), 1.into()),
            BigRational::new(2.into(), 1.into()),
        );
        let d = BigRational::new(3.into(), 1.into());

        assert_eq!(
            &p * &d,
            Point::new(
                BigRational::new(3.into(), 1.into()),
                BigRational::new(6.into(), 1.into())
            )
        );

        assert_eq!(
            &p * d.clone(),
            Point::new(
                BigRational::new(3.into(), 1.into()),
                BigRational::new(6.into(), 1.into())
            )
        );

        assert_eq!(
            p.clone() * d.clone(),
            Point::new(
                BigRational::new(3.into(), 1.into()),
                BigRational::new(6.into(), 1.into())
            )
        );

        assert_eq!(
            p.clone() * &d,
            Point::new(
                BigRational::new(3.into(), 1.into()),
                BigRational::new(6.into(), 1.into())
            )
        );

        assert_eq!(
            &p / &d,
            Point::new(
                BigRational::new(1.into(), 3.into()),
                BigRational::new(2.into(), 3.into())
            )
        );

        assert_eq!(
            &p / d.clone(),
            Point::new(
                BigRational::new(1.into(), 3.into()),
                BigRational::new(2.into(), 3.into())
            )
        );

        assert_eq!(
            p.clone() / d.clone(),
            Point::new(
                BigRational::new(1.into(), 3.into()),
                BigRational::new(2.into(), 3.into())
            )
        );

        assert_eq!(
            p / &d,
            Point::new(
                BigRational::new(1.into(), 3.into()),
                BigRational::new(2.into(), 3.into())
            )
        );
    }

    #[test]
    fn test_dot_cross() {
        use num_rational::BigRational;

        use super::*;

        let p = Point::new(
            BigRational::new(1.into(), 1.into()),
            BigRational::new(2.into(), 1.into()),
        );
        let q = Point::new(
            BigRational::new(3.into(), 1.into()),
            BigRational::new(4.into(), 1.into()),
        );

        assert_eq!(p.dot(&q), BigRational::new(11.into(), 1.into()));
        assert_eq!(p.dot(q.clone()), BigRational::new(11.into(), 1.into()));

        assert_eq!(p.cross(&q), BigRational::new((-2).into(), 1.into()));
        assert_eq!(p.cross(q), BigRational::new((-2).into(), 1.into()));
    }
}
