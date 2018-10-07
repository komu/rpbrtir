use core::{
    geometry::RayDifferential,
    math::lerp,
    spectrum::Spectrum,
    types::Float,
    rng::RNG,
    integrator::SurfaceIntegrator,
    intersection::Intersection,
};
use core::scene::Scene;
use core::integrator::VolumeIntegrator;
use std::ops::Index;
use std::ops::IndexMut;

pub struct Sample {
    pub cam: CameraSample,
    pub n1d: Vec<usize>,
    pub n2d: Vec<usize>,
    pub one_d: Vec<SamplePtr>,
    pub two_d: Vec<SamplePtr>,
    mem: Vec<Float>
}

#[derive(Clone, Copy)]
pub struct SampleOffset1d(pub usize);

#[derive(Clone, Copy)]
pub struct SampleOffset2d(pub usize);

#[derive(Clone, Copy)]
pub struct SamplePtr(pub usize);

impl Sample {

    pub fn new(sampler: &Sampler,
               surf: Option<&mut SurfaceIntegrator>,
               vol: Option<&mut VolumeIntegrator>,
               scene: &Scene) -> Sample {

        // TODO: make the sample request protocol cleaner
        let mut sample = Sample {
            cam: CameraSample::default(),
            n1d: vec![],
            n2d: vec![],
            one_d: vec![],
            two_d: vec![],
            mem: vec![]
        };

        if let Some(surf) = surf {
            surf.request_samples(Some(sampler), &mut sample, scene);
        }
        if let Some(vol) = vol {
            vol.request_samples(Some(sampler), &mut sample, scene);
        }

        sample.allocate_sample_memory();

        sample
    }

    pub fn offsets_1d(&self) -> Vec<(SampleOffset1d, usize)> {
        self.n1d.iter().enumerate().map(|(i, len)| (SampleOffset1d(i), *len)).collect()
    }

    pub fn offsets_2d(&self) -> Vec<(SampleOffset2d, usize)> {
        self.n2d.iter().enumerate().map(|(i, len)| (SampleOffset2d(i), *len)).collect()
    }

    pub fn add_1d(&mut self, count: u32) -> SampleOffset1d {
        self.n1d.push(count as usize);
        SampleOffset1d(self.n1d.len() - 1)
    }

    pub fn add_2d(&mut self, count: u32) -> SampleOffset2d {
        self.n2d.push(count as usize);
        SampleOffset2d(self.n2d.len() - 1)
    }

    fn allocate_sample_memory(&mut self) {
        let size1: usize = self.n1d.iter().sum();
        let size2: usize = self.n2d.iter().sum();
        let total_samples = size1 + 2 * size2;

        self.mem.reserve(total_samples);

        // TODO: optimize to use aligned memory
        // Allocate storage for sample values
        self.one_d.reserve(self.n1d.len());
        for (i, size) in self.n1d.iter().enumerate() {
            self.one_d.push(SamplePtr(self.mem.len()));
            self.mem.push(0.0);
        }

        self.two_d.reserve(self.n2d.len());
        for (i, size) in self.n2d.iter().enumerate() {
            self.two_d.push(SamplePtr(self.mem.len()));
            self.mem.push(0.0);
            self.mem.push(0.0);
        }
    }
}

impl Index<SampleOffset1d> for Sample {
    type Output = [Float];

    fn index(&self, SampleOffset1d(index): SampleOffset1d) -> &[Float] {
        let SamplePtr(ptr) = self.one_d[index];
        let len = self.n1d[index];

        &self.mem[ptr..ptr+len]
    }
}

impl Index<SampleOffset2d> for Sample {
    type Output = [Float];

    fn index(&self, SampleOffset2d(index): SampleOffset2d) -> &[Float] {
        let SamplePtr(ptr) = self.two_d[index];
        let len = self.n2d[index];

        &self.mem[ptr..ptr+2*len]
    }
}

impl IndexMut<SampleOffset1d> for Sample {
    fn index_mut(&mut self, SampleOffset1d(index): SampleOffset1d) -> &mut [Float] {
        let SamplePtr(ptr) = self.one_d[index];
        let len = self.n1d[index];

        let r = &mut self.mem[ptr..ptr+len];
        r
    }
}

impl IndexMut<SampleOffset2d> for Sample {
    fn index_mut(&mut self, SampleOffset2d(index): SampleOffset2d) -> &mut [Float] {
        let SamplePtr(ptr) = self.two_d[index];
        let len = self.n2d[index];

        &mut self.mem[ptr..ptr+2*len]
    }
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
    fn get_sub_sampler(&self, num: u32, count: u32) -> Option<Box<Sampler>>;
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
