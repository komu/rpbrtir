use core::types::Float;

pub trait Filter {
    fn evaluate(&self, x: Float, y: Float) -> Float;
    fn dimensions(&self) -> &FilterDimensions;
}

pub struct FilterDimensions {
    pub x_width: Float,
    pub y_width: Float,
    pub inv_x_width: Float,
    pub inv_y_width: Float,
}

impl FilterDimensions {
    pub fn new(x_width: Float, y_width: Float) -> FilterDimensions {
        FilterDimensions {
            x_width,
            y_width,
            inv_x_width: 1.0 / x_width,
            inv_y_width: 1.0 / y_width,
        }
    }
}
