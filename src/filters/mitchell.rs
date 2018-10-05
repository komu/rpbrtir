use core::{
    filter::{Filter, FilterDimensions},
    types::Float,
};

pub struct MitchellFilter {
    dimensions: FilterDimensions,
    b: Float,
    c: Float,
}

impl MitchellFilter {
    fn new(xw: Float, yw: Float, b: Float, c: Float) -> MitchellFilter {
        MitchellFilter { dimensions: FilterDimensions::new(xw, yw), b, c }
    }

    fn mitchell_1d(&self, x: Float) -> Float {
        let b = self.b;
        let c = self.c;
        let x = (2.0 * x).abs();
        if x > 1.0 {
            ((-b - 6.0 * c) * x * x * x + (6.0 * b + 30.0 * c) * x * x +
                (-12.0 * b - 48.0 * c) * x + (8.0 * b + 24.0 * c)) * (1.0 / 6.0)
        } else {
            ((12.0 - 9.0 * b - 6.0 * c) * x * x * x +
                (-18.0 + 12.0 * b + 6.0 * c) * x * x +
                (6.0 - 2.0 * b)) * (1.0 / 6.0)
        }
    }
}

impl Filter for MitchellFilter {
    fn evaluate(&self, x: Float, y: Float) -> Float {
        self.mitchell_1d(x * self.dimensions.inv_x_width) * self.mitchell_1d(y * self.dimensions.inv_y_width)
    }

    fn dimensions(&self) -> &FilterDimensions {
        &self.dimensions
    }
}

impl Default for MitchellFilter {
    fn default() -> Self {
        MitchellFilter::new(2.0, 2.0, 1.0/3.0, 1.0/3.0)
    }
}
