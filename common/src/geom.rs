use euclid::Vector2D;
use lyon_geom::{Line, LineSegment};

type P = Vector2D<f64, euclid::UnknownUnit>;

// tangents from p to circle.
// returns right hand side tanget point first.
pub fn tangent_to_circle(p: P, c: P, r: f64) -> (P, P) {
    let d2 = (p - c).square_length();
    let a2 = d2 - r * r;

    cross_points_cc2(p, a2, c, r * r)
}

// Taking two circles, one whose center is |c1| and whose radius is
// r1 (where r1 = sqrt(r1_2)), and the other whose center is |c2|
// and whose radius is r2 (where r2 = sqrt(r2_2)),
// returns the crossing point, assuming there exist.
//
// Let
//   - p1, p2 are the crossing points
//   - dv is (c2 - c1)
//   - d is the distance between c1 and c2 (i.e. d == |dv|), and
//   - T is the angle of (p1 - c1) and dv.
//
// From cosine law:
//  r2^2 = r1^2 + d^2 - 2 * r1 * d * cos(T).
//  cos(T) = (r1^2 + d^2 - r2^2) / (2 * r1 * d)
//  sin(T) = sqrt(1 - cos^2(T)) = sqrt(1 - {(r1^2 + d^2 - r2^2) / (2 * r1 * d)}^2)
//  p1 = c1 + (r1 / d) * R(T) * dv
//  p2 = c1 + (r1 / d) * R(-T) * dv
//    where R(T) is the rotation matrix.
// Specifically
//  p1.x = c1.x + (r1 / d) * (cos(T) * dv.x - sin(T) * dv.y)
//       = c1.x + (r1 / d) * ({(r1^2 + d^2 - r2^2) / (2 * r1 * d)} * dv.x
//                            - sqrt(1 - {(r1^2 + d^2 - r2^2) / (2 * r1 * d)}^2) * dv.y
//       = c1.x + {(r1^2 + d^2 - r2^2) / (2 * d^2)} * dv.x
//              - sqrt(r1^2 / d^2 - {(r1^2 + d^2 - r2^2) / (2 * d^2)}^2) * dv.y
//       = c1.x + {(r1^2 + d^2 - r2^2) / (2 * d^2)} * dv.x
//              - sqrt(4 * r1^2 * d^2 - (r1^2 + d^2 - r2^2)^2) / (2 * d^2) * dv.y
#[inline]
fn cross_points_cc2(c1: P, r1_2: f64, c2: P, r2_2: f64) -> (P, P) {
    let dv = c2 - c1;
    let d2 = dv.square_length();
    let cv = d2 + r1_2 - r2_2;
    let sv = (4. * d2 * r1_2 - cv * cv).sqrt();
    let cv = cv / (2. * d2);
    let sv = sv / (2. * d2);
    let cvdx = cv * dv.x;
    let svdx = sv * dv.x;
    let cvdy = cv * dv.y;
    let svdy = sv * dv.y;
    // To align original behavior, swap p1 and p2.
    (
        c1 + P::new(cvdx + svdy, cvdy - svdx),
        c1 + P::new(cvdx - svdy, cvdy + svdx),
    )
}

pub fn rotate90(p: P) -> P {
    P::new(-p.y, p.x)
}

// Returns the center of a circle with radius r tangenting two circles c1 and c2.
// This function assumes that c1 and c2 are tangent and have the same radius (which can be different from r)
// The circle returned is on the left hand side of the line c1c2.
pub fn tangent_circle(c1: P, c2: P, r: f64) -> Option<P> {
    let rc = (c1 - c2).length() / 2.;

    tangent_circle2(c1, c2, rc, r)
}

// Returns the center of a circle with radius r tangenting two circles c1 and c2.
// This function assumes that c1 and c2 have the same radius cr.
// The circle returned is on the left hand side of the line c1c2.
pub fn tangent_circle2(c1: P, c2: P, cr: f64, r: f64) -> Option<P> {
    let s = (c1 - c2).length() / 2.;

    let d2 = (r + cr) * (r + cr) - (s * s);
    if d2 < 0. {
        return None;
    }
    let d = d2.sqrt();

    let mid = (c1 + c2) / 2.0;
    let n = rotate90(c2 - c1).normalize();

    Some(mid + n * d)
}

// Returns all the circles tangenting the given two lines.
// Returns an empty vector if the two lines are parallel.
pub fn circles_tangenting_lines(p1: P, p2: P, q1: P, q2: P, r: f64) -> Vec<P> {
    let np = rotate90(p2 - p1).normalize() * r;
    let nq = rotate90(q2 - q1).normalize() * r;

    let mut res = vec![];
    for dp in [-1., 1.] {
        for dq in [-1., 1.] {
            let line_p = new_line(p1 + np * dp, p2 + np * dp);
            let line_q = new_line(q1 + nq * dq, q2 + nq * dq);

            line_p
                .intersection(&line_q)
                .map(|p| res.push(p.to_vector()));
        }
    }
    res
}

pub fn circles_tangenting_line_and_circle(p1: P, p2: P, c: P, cr: f64, r: f64) -> Vec<P> {
    let dir = rotate90((p2 - p1).normalize() * r);

    assert_not_nan_point(dir);

    let mut res = vec![];
    res.append(&mut line_circle_intersections(
        p1 + dir,
        p2 + dir,
        c,
        r + cr,
    ));
    res.append(&mut line_circle_intersections(
        p1 - dir,
        p2 - dir,
        c,
        r + cr,
    ));
    res
}

pub fn line_circle_intersections(p1: P, p2: P, c: P, cr: f64) -> Vec<P> {
    assert_not_nan(cr);
    assert_not_nan_point(c);
    assert_not_nan_point(p1);
    assert_not_nan_point(p2);

    let line = new_line(p1, p2);

    let p = line.equation().project_point(&c.to_point()).to_vector();

    assert_not_nan_point(p);

    let d2 = (p - c).square_length();

    let cr2 = cr * cr;

    let l2 = cr2 - d2;

    if l2 < 0. {
        return vec![];
    }

    let l = l2.sqrt();

    let n = (p2 - p1).normalize() * l;

    assert_not_nan_point(n);

    vec![p + n, p - n]
}

fn assert_not_nan_point(p: P) {
    assert!(!p.x.is_nan() && !p.y.is_nan());
}

fn assert_not_nan(f: f64) {
    assert!(!f.is_nan());
}

pub fn new_line(p1: P, p2: P) -> Line<f64> {
    LineSegment {
        from: p1.to_point(),
        to: p2.to_point(),
    }
    .to_line()
}

#[cfg(test)]
mod tests {
    use super::{tangent_to_circle, P};

    #[test]
    fn test_tanget_to_circle() {
        let c = P::new(2., 2.);
        let r = 2.;

        for p in vec![P::new(0., 0.), P::new(4., 4.), P::new(4., 0.)] {
            let (q1, q2) = tangent_to_circle(p, c, r);

            assert!(((q1 - c).length() - 2.).abs() < 1e-9, "{:?}", q1);
            assert!(((q2 - c).length() - 2.).abs() < 1e-9, "{:?}", q2);
        }

        let (q1, q2) = tangent_to_circle(P::new(0., 0.), c, r);

        assert!((q1 - P::new(2., 0.)).length() < 1e-9, "{:?}", q1);
        assert!((q2 - P::new(0., 2.)).length() < 1e-9, "{:?}", q1);

        let (q1, q2) = tangent_to_circle(P::new(4., 4.), c, r);

        assert!((q1 - P::new(2., 4.)).length() < 1e-9, "{:?}", q1);
        assert!((q2 - P::new(4., 2.)).length() < 1e-9, "{:?}", q1);
    }

    #[test]
    fn test_circles_tangenting_lines() {
        let p1 = P::new(0., 0.);
        let p2 = P::new(1., 0.);
        let q1 = P::new(0., 0.);
        let q2 = P::new(0., 1.);

        for r in [1., 2.] {
            let res = super::circles_tangenting_lines(p1, p2, q1, q2, r);

            assert_eq!(res.len(), 4);

            for i in 0..4 {
                for j in 0..i {
                    assert!((res[i] - res[j]).length() > 1e-9);
                }
                let dx = (res[i].x.abs() - r).abs();
                let dy = (res[i].y.abs() - r).abs();

                assert!(dx < 1e-9 || dy < 1e-9);
            }
        }
    }

    #[test]
    fn test_circles_tangenting_line_and_circle() {
        let p1 = P::new(0., 0.);
        let p2 = P::new(1., 0.);
        let c = P::new(0., 5.);
        let cr = 3.;
        let r = 2.;

        let res = super::circles_tangenting_line_and_circle(p1, p2, c, cr, r);

        assert_eq!(res.len(), 2);

        for i in 0..2 {
            for j in 0..i {
                assert!((res[i] - res[j]).length() > 1e-9);
            }

            let exp = (4., 2.);

            let dx = (res[i].x.abs() - exp.0).abs();
            let dy = (res[i].y.abs() - exp.1).abs();

            assert!(dx < 1e-9 || dy < 1e-9);
        }
    }
}
