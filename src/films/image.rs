use core::film::Film;
use core::film::Extent;
use core::sampler::CameraSample;
use core::spectrum::Spectrum;
use core::types::Float;
use image::{ImageBuffer, Rgb};
use core::math::clamp;

pub struct ImageFilm {
    path: String,
    x_resolution: u32,
    y_resolution: u32,
    img: Vec<Pixel>,

}

#[derive(Default)]
struct Pixel {
    r: Float,
    g: Float,
    b: Float,
    weight_sum: Float,
}

impl Pixel {
    fn to_rgb(&self) -> Rgb<u8> {
        let ir = image_component(self.r, self.weight_sum);
        let ig = image_component(self.g, self.weight_sum);
        let ib = image_component(self.b, self.weight_sum);

        Rgb([ir, ig, ib])
    }
}

#[inline]
fn image_component(x: Float, weight_sum: Float) -> u8 {
    if weight_sum == 0.0 { return 0 }
    let r = clamp(x / weight_sum, 0.0, 1.0);
    (255.99 * r.sqrt()) as u8
}

impl ImageFilm {
    pub fn new(path: String, x_resolution: u32, y_resolution: u32) -> ImageFilm {
        ImageFilm {
            path,
            x_resolution,
            y_resolution,
            img: (0..x_resolution * y_resolution).map(|_| Pixel::default()).collect(),
        }
    }
}

impl Film for ImageFilm {
    fn add_sample(&mut self, sample: &CameraSample, l: &Spectrum) {

        // TODO: support filters
        let x = sample.image_x.floor() as u32;
        let y = sample.image_y.floor() as u32;

        if x >= self.x_resolution || y >= self.y_resolution {
            return
//            panic!("{},{} is out of bounds ({}, {})", x, y, sample.image_x, sample.image_y);
        }

        let index = (y * self.x_resolution + x) as usize;

        let px = &mut self.img[index];
        px.r += l.r;
        px.g += l.g;
        px.b += l.b;
        px.weight_sum += 1.0;
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
        let mut buf = ImageBuffer::from_fn(self.x_resolution, self.y_resolution, |x, y| {
            let index = (y * self.x_resolution + x) as usize;
            self.img[index].to_rgb()
        });
        buf.save(&self.path).expect("failed to write image");
    }

    fn resolution(&self) -> (u32, u32) {
        (self.x_resolution, self.y_resolution)
    }
}
