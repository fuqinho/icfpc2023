use euclid::Vector2D;

type P = Vector2D<f64, euclid::UnknownUnit>;

// tangents from p to circle.
// returns right hand side tanget point first.
pub fn tangent_to_circle(p: P, c: P, r: f64) -> (P, P) {
    let d = (p - c).length();
    let a = (d * d - r * r).sqrt();

    cross_points_cc(p, a, c, r)
}

fn cross_points_cc(c1: P, r1: f64, c2: P, r2: f64) -> (P, P) {
    let d = (c1 - c2).length();
    let a = (r1 * r1 - r2 * r2 + d * d) / (2. * d);
    let h = (r1 * r1 - a * a).sqrt();
    let p = c1 + (c2 - c1) * (a / d);
    let w = (c2 - c1).normalize();
    let n = rotate90(w);
    (p - n * h, p + n * h)
}

pub fn rotate90(p: P) -> P {
    P::new(-p.y, p.x)
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
