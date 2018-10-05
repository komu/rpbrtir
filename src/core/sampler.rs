use core::{
    geometry::RayDifferential,
    spectrum::Spectrum,
    types::Float,
    rng::RNG,
    intersection::Intersection,
};
use core::math::lerp;

#[derive(Default)]
pub struct Sample {
    pub cam: CameraSample
}

#[derive(Default)]
pub struct CameraSample {
    pub image_x: Float,
    pub image_y: Float,
    pub lens_u: Float,
    pub lens_v: Float,
    pub time: Float,
}

pub trait Sampler {
    fn get_more_samples(&mut self, sample: &mut Sample, rng: &mut RNG) -> u32;
    fn maximum_sample_count(&self) -> u32;
    fn report_results(&mut self, samples: &mut [Sample], rays: &[RayDifferential], ls: &[Spectrum], isects: &[Intersection], count: u32) -> bool {
        true
    }
    fn get_sub_sampler(&self, num: u32, count: u32) -> Option<Box<Self>>;
    fn round_size(&self, size: u32) -> u32;
}

pub struct SamplerWindow {
    pub x_start: u32,
    pub x_end: u32,
    pub y_start: u32,
    pub y_end: u32,
}

impl SamplerWindow {

    pub fn new(x_start: u32, x_end: u32, y_start: u32, y_end: u32) -> SamplerWindow {
        SamplerWindow { x_start, x_end, y_start, y_end }
    }

    pub fn from_dimensions(width: u32, height: u32) -> SamplerWindow {
        SamplerWindow { x_start: 0, x_end: width, y_start: 0, y_end: height }
    }

    pub fn is_empty(&self) -> bool {
        self.x_start == self.x_end || self.y_start == self.y_end
    }

    pub fn compute_sub_window(&self, num: u32, count: u32) -> SamplerWindow {
        // Determine how many tiles to use in each dimension, _nx_ and _ny_
        let dx = self.x_end - self.x_start;
        let dy = self.y_end - self.y_start;
        let mut nx = count;
        let mut ny = 1;
        while (nx & 0x1) == 0 && 2 * dx * ny < dy * nx {
            nx >>= 1;
            ny <<= 1;
        }
        assert_eq!(nx * ny, count);

        // Compute $x$ and $y$ pixel sample range for sub-window
        let xo = num % nx;
        let yo = num / nx;
        let tx0 = float(xo) / float(nx);
        let tx1 = float(xo + 1) / float(nx);
        let ty0 = float(yo) / float(ny);
        let ty1 = float(yo + 1) / float(ny);

        SamplerWindow {
            x_start: lerp(tx0, self.x_start as Float, self.x_end as Float).floor() as u32,
            x_end: lerp(tx1, self.x_start as Float, self.x_end as Float).floor() as u32,
            y_start: lerp(ty0, self.y_start as Float, self.y_end as Float).floor() as u32,
            y_end: lerp(ty1, self.y_start as Float, self.y_end as Float).floor() as u32
        }
    }
}

#[inline]
fn float(x: u32) -> Float {
    x as Float
}
