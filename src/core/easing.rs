/// Easing curves from <https://easings.net/>.
///
/// [`Self::evaluate`] expects a normalized animation progress from `0.0` to `1.0`.
#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InQuart,
    OutQuart,
    InOutQuart,
    InQuint,
    OutQuint,
    InOutQuint,
    InSine,
    OutSine,
    InOutSine,
    InExpo,
    OutExpo,
    InOutExpo,
    InCirc,
    OutCirc,
    InOutCirc,
    InBack,
    OutBack,
    InOutBack,
    InElastic,
    OutElastic,
    InOutElastic,
    InBounce,
    OutBounce,
    InOutBounce,
}

impl Easing {
    /// Every easing curve supported by Kinematic.
    pub const ALL: [Self; 31] = [
        Self::Linear,
        Self::InQuad,
        Self::OutQuad,
        Self::InOutQuad,
        Self::InCubic,
        Self::OutCubic,
        Self::InOutCubic,
        Self::InQuart,
        Self::OutQuart,
        Self::InOutQuart,
        Self::InQuint,
        Self::OutQuint,
        Self::InOutQuint,
        Self::InSine,
        Self::OutSine,
        Self::InOutSine,
        Self::InExpo,
        Self::OutExpo,
        Self::InOutExpo,
        Self::InCirc,
        Self::OutCirc,
        Self::InOutCirc,
        Self::InBack,
        Self::OutBack,
        Self::InOutBack,
        Self::InElastic,
        Self::OutElastic,
        Self::InOutElastic,
        Self::InBounce,
        Self::OutBounce,
        Self::InOutBounce,
    ];

    pub fn evaluate(self, x: f32) -> f32 {
        match self {
            Self::Linear => x,
            Self::InQuad => x * x,
            Self::OutQuad => 1.0 - (1.0 - x) * (1.0 - x),
            Self::InOutQuad => {
                if x < 0.5 {
                    2.0 * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0) * (-2.0 * x + 2.0) * 0.5
                }
            }
            Self::InCubic => x * x * x,
            Self::OutCubic => 1.0 - (1.0 - x).powi(3),
            Self::InOutCubic => {
                if x < 0.5 {
                    4.0 * x * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(3) * 0.5
                }
            }
            Self::InQuart => x.powi(4),
            Self::OutQuart => 1.0 - (1.0 - x).powi(4),
            Self::InOutQuart => {
                if x < 0.5 {
                    8.0 * x.powi(4)
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(4) * 0.5
                }
            }
            Self::InQuint => x.powi(5),
            Self::OutQuint => 1.0 - (1.0 - x).powi(5),
            Self::InOutQuint => {
                if x < 0.5 {
                    16.0 * x.powi(5)
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(5) * 0.5
                }
            }
            Self::InSine => 1.0 - (x * std::f32::consts::FRAC_PI_2).cos(),
            Self::OutSine => (x * std::f32::consts::FRAC_PI_2).sin(),
            Self::InOutSine => -((std::f32::consts::PI * x).cos() - 1.0) * 0.5,
            Self::InExpo => {
                if x == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * x - 10.0)
                }
            }
            Self::OutExpo => {
                if x == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * x)
                }
            }
            Self::InOutExpo => {
                if x == 0.0 {
                    0.0
                } else if x == 1.0 {
                    1.0
                } else if x < 0.5 {
                    2.0_f32.powf(20.0 * x - 10.0) * 0.5
                } else {
                    (2.0 - 2.0_f32.powf(-20.0 * x + 10.0)) * 0.5
                }
            }
            Self::InCirc => 1.0 - (1.0 - x.powi(2)).sqrt(),
            Self::OutCirc => (1.0 - (x - 1.0).powi(2)).sqrt(),
            Self::InOutCirc => {
                if x < 0.5 {
                    (1.0 - (1.0 - (2.0 * x).powi(2)).sqrt()) * 0.5
                } else {
                    ((1.0 - (-2.0 * x + 2.0).powi(2)).sqrt() + 1.0) * 0.5
                }
            }
            Self::InBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * x.powi(3) - c1 * x.powi(2)
            }
            Self::OutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
            }
            Self::InOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;

                if x < 0.5 {
                    (2.0 * x).powi(2) * ((c2 + 1.0) * 2.0 * x - c2) * 0.5
                } else {
                    ((2.0 * x - 2.0).powi(2) * ((c2 + 1.0) * (x * 2.0 - 2.0) + c2) + 2.0) * 0.5
                }
            }
            Self::InElastic => {
                let c4 = 2.0 * std::f32::consts::PI / 3.0;

                if x == 0.0 {
                    0.0
                } else if x == 1.0 {
                    1.0
                } else {
                    -2.0_f32.powf(10.0 * x - 10.0) * ((10.0 * x - 10.75) * c4).sin()
                }
            }
            Self::OutElastic => {
                let c4 = 2.0 * std::f32::consts::PI / 3.0;

                if x == 0.0 {
                    0.0
                } else if x == 1.0 {
                    1.0
                } else {
                    2.0_f32.powf(-10.0 * x) * ((10.0 * x - 0.75) * c4).sin() + 1.0
                }
            }
            Self::InOutElastic => {
                let c5 = 2.0 * std::f32::consts::PI / 4.5;

                if x == 0.0 {
                    0.0
                } else if x == 1.0 {
                    1.0
                } else if x < 0.5 {
                    -2.0_f32.powf(20.0 * x - 10.0) * ((20.0 * x - 11.125) * c5).sin() * 0.5
                } else {
                    2.0_f32.powf(-20.0 * x + 10.0) * ((20.0 * x - 11.125) * c5).sin() * 0.5 + 1.0
                }
            }
            Self::InBounce => 1.0 - Self::out_bounce(1.0 - x),
            Self::OutBounce => Self::out_bounce(x),
            Self::InOutBounce => {
                if x < 0.5 {
                    (1.0 - Self::out_bounce(1.0 - 2.0 * x)) * 0.5
                } else {
                    (1.0 + Self::out_bounce(2.0 * x - 1.0)) * 0.5
                }
            }
        }
    }

    fn out_bounce(x: f32) -> f32 {
        let n1 = 7.5625;
        let d1 = 2.75;

        if x < 1.0 / d1 {
            n1 * x * x
        } else if x < 2.0 / d1 {
            let x = x - 1.5 / d1;
            n1 * x * x + 0.75
        } else if x < 2.5 / d1 {
            let x = x - 2.25 / d1;
            n1 * x * x + 0.9375
        } else {
            let x = x - 2.625 / d1;
            n1 * x * x + 0.984375
        }
    }
}

impl Default for Easing {
    fn default() -> Self {
        Self::InOutCubic
    }
}

#[cfg(test)]
mod tests {
    use super::Easing;

    #[test]
    fn all_easings_preserve_animation_endpoints() {
        assert_eq!(Easing::ALL.len(), 31);

        for easing in Easing::ALL {
            assert!(
                easing.evaluate(0.0).abs() < 0.000_001,
                "{easing:?} must start at zero."
            );
            assert!(
                (easing.evaluate(1.0) - 1.0).abs() < 0.000_001,
                "{easing:?} must end at one."
            );
        }
    }
}
