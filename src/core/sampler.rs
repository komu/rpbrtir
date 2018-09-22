use core::types::Float;

pub struct Sample {}

pub struct CameraSample {
    pub image_x: Float,
    pub image_y: Float,
    pub lens_u: Float,
    pub lens_v: Float,
    pub time: Float,
}
