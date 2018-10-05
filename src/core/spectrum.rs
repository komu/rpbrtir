use core::types::{Float, INFINITY};
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Div;
use image::Rgb;
use std::ops::Add;
use std::iter::Sum;
use core::math::clamp;
use std::ops::Sub;

#[derive(Clone, Copy)]
pub struct Spectrum {
    pub r: Float,
    pub g: Float,
    pub b: Float,
}

impl Spectrum {
    pub fn new(r: Float, g: Float, b: Float) -> Spectrum {
        Spectrum { r, g, b }
    }

    pub fn black() -> Spectrum {
        Spectrum::new(0.0, 0.0, 0.0)
    }
    pub fn white() -> Spectrum {
        Spectrum::new(1.0, 1.0, 1.0)
    }
    pub fn green() -> Spectrum {
        Spectrum::new(0.0, 1.0, 0.0)
    }
    pub fn blue() -> Spectrum {
        Spectrum::new(0.0, 0.0, 1.0)
    }

    pub fn is_black(&self) -> bool {
        self.r == 0.0 && self.g == 0.0 && self.b == 0.0
    }

    pub fn clamp_positive(&self) -> Spectrum {
        self.clamp(0.0, INFINITY)
    }

    pub fn clamp(&self, low: Float, high: Float) -> Spectrum {
        Spectrum {
            r: clamp(self.r, low, high),
            g: clamp(self.g, low, high),
            b: clamp(self.b, low, high)
        }
    }
}

impl From<Float> for Spectrum {
    fn from(v: Float) -> Spectrum {
        Spectrum::new(v, v, v)
    }
}

impl Add<Spectrum> for Spectrum {
    type Output = Spectrum;

    fn add(self, rhs: Spectrum) -> Spectrum {
        Spectrum::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl Sub<Spectrum> for Spectrum {
    type Output = Spectrum;

    fn sub(self, rhs: Spectrum) -> Spectrum {
        Spectrum::new(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b)
    }
}

impl AddAssign<Spectrum> for Spectrum {
    fn add_assign(&mut self, rhs: Spectrum) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Mul<Spectrum> for Spectrum {
    type Output = Spectrum;

    fn mul(self, rhs: Spectrum) -> Spectrum {
        Spectrum {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<Float> for Spectrum {
    type Output = Spectrum;

    fn mul(self, rhs: Float) -> Spectrum {
        Spectrum {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl Mul<Spectrum> for Float {
    type Output = Spectrum;

    fn mul(self, rhs: Spectrum) -> Spectrum {
        Spectrum {
            r: self * rhs.r,
            g: self * rhs.g,
            b: self * rhs.b
        }
    }
}

impl Div<Float> for Spectrum {
    type Output = Spectrum;

    fn div(self, rhs: Float) -> Spectrum {
        Spectrum {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}

impl Div<Spectrum> for Spectrum {
    type Output = Spectrum;

    fn div(self, rhs: Spectrum) -> Spectrum {
        Spectrum {
            r: self.r / rhs.r,
            g: self.g / rhs.g,
            b: self.b / rhs.b,
        }
    }
}

impl Sum<Spectrum> for Spectrum {
    fn sum<I: Iterator<Item=Spectrum>>(iter: I) -> Spectrum {
        let mut sum = Spectrum::black();
        for s in iter {
            sum += s;
        }
        sum
    }
}
