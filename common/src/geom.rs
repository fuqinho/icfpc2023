use euclid::Vector2D;

type P = Vector2D<f64, euclid::UnknownUnit>;

// tangents from p to circle.
// returns right hand side tanget point first.
pub fn tangent_to_circle(p: P, c: P, r: f64) -> (P, P) {
    let d2 = (p - c).square_length();
    let a2 = d2 - r * r;

    cross_points_cc2(p, a2, c, r * r)
}

fn cross_points_cc(c1: P, r1: f64, c2: P, r2: f64) -> (P, P) {
    cross_points_cc2(c1, r1 * r1, c2, r2 * r2)
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
//                            - sqrt(1 - {(r1^2 + d^2 - r2^2) / (2 * r1 * d)}) * dv.y
//       = c1.x + {(r1^2 + d^2 - r2^2) / (2 * d^2)} * dv.x
//              - sqrt(r1^2 / d^2 - {(r1^2 + d^2 - r2^2) / (2 * d^2)}) * dv.y
//       = c1.x + {(r1^2 + d^2 - r2^2) / (2 * d^2)} * dv.x
//              - sqrt(4 * r1^2 * d^2 - (r1^2 + d^2 - r2^2)) / (2 * d^2) * dv.y
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
// This function assumes that c1 and c2 have the same radius (which can be different from r)
// The circle returned is on the left hand side of the line c1c2.
pub fn tangent_circle(c1: P, c2: P, r: f64) -> Option<P> {
    let rc = (c1 - c2).length() / 2.;
    let d2 = (r + rc) * (r + rc) - (rc * rc);
    if d2 < 0. {
        return None;
    }
    let d = d2.sqrt();

    let mid = (c1 + c2) / 2.0;
    let n = rotate90(c2 - c1).normalize();

    Some(mid + n * d)
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
}
