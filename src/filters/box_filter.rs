use core::{
    filter::{Filter, FilterDimensions},
    types::Float,
};

pub struct BoxFilter {
    dimensions: FilterDimensions
}

impl BoxFilter {
    pub fn new(x_width: Float, y_width: Float) -> BoxFilter {
        BoxFilter { dimensions: FilterDimensions::new(x_width, y_width) }
    }
}

impl Default for BoxFilter {
    fn default() -> Self {
        BoxFilter::new(0.5, 0.5)
    }
}

impl Filter for BoxFilter {
    fn evaluate(&self, _x: Float, _y: Float) -> Float {
        1.0
    }

    fn dimensions(&self) -> &FilterDimensions {
        &self.dimensions
    }
}