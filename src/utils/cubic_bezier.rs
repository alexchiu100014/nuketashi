#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CubicBezier {
    a1: f64,
    b1: f64,
    c1: f64,
    a2: f64,
    b2: f64,
    c2: f64,
}

const EPSILON: f64 = 1.0e-3;
const EPSILON_SQ: f64 = EPSILON * EPSILON;

impl CubicBezier {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            a1: 3.0 * x1 - 3.0 * x2 + 1.0,
            b1: -6.0 * x1 + 3.0 * x2,
            c1: 3.0 * x1,
            a2: 3.0 * y1 - 3.0 * y2 + 1.0,
            b2: -6.0 * y1 + 3.0 * y2,
            c2: 3.0 * y1,
        }
    }

    fn x_t(&self, t: f64) -> f64 {
        ((self.a1 * t + self.b1) * t + self.c1) * t
    }

    fn y_t(&self, t: f64) -> f64 {
        ((self.a2 * t + self.b2) * t + self.c2) * t
    }

    fn dx_t(&self, t: f64) -> f64 {
        (3.0 * self.a1 * t + 2.0 * self.b1) * t + self.c1
    }

    pub fn apply(&self, x: f64) -> f64 {
        if x < EPSILON {
            return 0.0;
        } else if (1.0 - EPSILON) < x {
            return 1.0;
        }

        let mut t = x;

        for _ in 0..8 {
            let d = self.x_t(t) - x;
            if d.abs() < EPSILON {
                return self.y_t(t);
            }

            let dx = self.dx_t(t);
            if dx.abs() < EPSILON_SQ {
                break;
            }
            t = t - x / dx;
        }

        let mut t0 = 0.0;
        let mut t1 = 1.0;

        while t0 < t1 {
            let x2 = self.x_t(t);
            if (x2 - x).abs() < EPSILON {
                return self.y_t(t);
            }

            if x > x2 {
                t0 = t;
            } else {
                t1 = t;
            }

            t = (t0 + t1) * 0.5;
        }

        return self.y_t(t);
    }
}
