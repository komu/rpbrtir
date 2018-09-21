use core::spectrum::Spectrum;
use core::sampler::CameraSample;
use core::types::Float;

pub trait Film {

    // TODO: temporary
    fn put_pixel(&mut self, x: u32, y: u32, l: &Spectrum);

    fn add_sample(&mut self, sample: &CameraSample, l: &Spectrum);
    fn splat(&mut self, sample: &CameraSample, l: &Spectrum);

    fn get_sample_extent(&self) -> Extent;
    fn get_pixel_extent(&self) -> Extent;

    fn update_display(&self, x0: u32, y0: u32, x1: u32, y1: u32, splat_scale: Float) {}

    fn write_image_with_scale(&self, splat_scale: Float);

    fn write_image(&self) {
        self.write_image_with_scale(1.0)
    }

    fn x_resolution(&self) -> u32;
    fn y_resolution(&self) -> u32;
}

pub struct Extent {
    xstart: u32,
    xend: u32,
    ystart: u32,
    yend: u32,
}