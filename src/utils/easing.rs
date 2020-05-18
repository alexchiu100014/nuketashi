use crate::utils::cubic_bezier::CubicBezier;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    CubicBezier(CubicBezier),
}

impl Easing {
    pub fn apply(self, x: f64) -> f64 {
        match self {
            Self::EaseIn => x * x,
            Self::EaseOut => x * (2.0 - x),
            Self::CubicBezier(b) => b.apply(x),
            _ => x,
        }
    }
}
