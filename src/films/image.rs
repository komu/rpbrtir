use core::film::Film;
use core::film::Extent;
use core::sampler::CameraSample;
use core::spectrum::Spectrum;
use core::types::Float;
use image::{ImageBuffer, Rgb};

pub struct ImageFilm {
    path: String,
    x_resolution: u32,
    y_resolution: u32,
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,

}

impl ImageFilm {
    pub fn new(path: String, x_resolution: u32, y_resolution: u32) -> ImageFilm {
        ImageFilm { path, x_resolution, y_resolution, img: ImageBuffer::new(x_resolution, y_resolution) }
    }
}

impl Film for ImageFilm {
    fn put_pixel(&mut self, x: u32, y: u32, l: &Spectrum) {
        self.img.put_pixel(x, y, l.to_rgb())
    }

    fn add_sample(&mut self, sample: &CameraSample, l: &Spectrum) {
        unimplemented!()
    }

    fn splat(&mut self, sample: &CameraSample, l: &Spectrum) {
        unimplemented!()
    }

    fn get_sample_extent(&self) -> Extent {
        unimplemented!()
    }

    fn get_pixel_extent(&self) -> Extent {
        unimplemented!()
    }

    fn write_image_with_scale(&self, splat_scale: Float) {
        self.img.save(&self.path).expect("failed to write image");
    }

    fn x_resolution(&self) -> u32 {
        self.x_resolution
    }

    fn y_resolution(&self) -> u32 {
        self.y_resolution
    }
}
