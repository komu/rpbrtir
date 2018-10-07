use core::{
    rng::RNG,
    sampler::{Sample, Sampler, SamplerWindow},
    types::Float,
};
use core::math::lerp;
use core::sampler::SampleOffset1d;

pub struct RandomSampler {
    window: SamplerWindow,
    samples_per_pixel: usize,
    num_samples: usize,
    shutter_open: Float,
    shutter_close: Float,
    x_pos: u32,
    y_pos: u32,
    sample_pos: usize,
    // TODO: consider using single array for all samples
    image_samples: Vec<Float>,
    lens_samples: Vec<Float>,
    time_samples: Vec<Float>,
}

impl RandomSampler {
    pub fn new(window: SamplerWindow,
               samples_per_pixel: usize,
               shutter_open: Float,
               shutter_close: Float) -> RandomSampler {

        // Get storage for a pixel's worth of stratified samples
        let x_pos = window.x_start;
        let y_pos = window.y_start;
        let mut rng = RNG::from_seed(x_pos + y_pos * (window.x_end - x_pos));

        RandomSampler {
            window,
            samples_per_pixel,
            shutter_open,
            shutter_close,
            num_samples: samples_per_pixel,
            x_pos,
            y_pos,
            sample_pos: 0,
            image_samples: (0..2 * samples_per_pixel).map(|i| {
                let offset = if i % 2 == 0 { x_pos } else { y_pos };
                offset as Float + rng.random_float()
            }).collect(),
            lens_samples: (0..2 * samples_per_pixel).map(|_| rng.random_float()).collect(),
            time_samples: (0..samples_per_pixel).map(|_| rng.random_float()).collect(),
        }
    }
}

impl Sampler for RandomSampler {
    fn get_more_samples(&mut self, sample: &mut Sample, rng: &mut RNG) -> u32 {
        if self.sample_pos == self.num_samples {
            if self.window.x_start == self.window.x_end || self.window.y_start == self.window.y_end {
                return 0
            }

            self.x_pos += 1;
            if self.x_pos == self.window.x_end {
                self.x_pos = self.window.x_start;
                self.y_pos += 1;
            }

            if self.y_pos == self.window.y_end {
                return 0
            }

            // TODO: duplication from constructor
            self.image_samples = (0..2 * self.num_samples).map(|i| {
                let offset = if i % 2 == 0 { i } else { i };
                offset as Float + rng.random_float()
            }).collect();

            for (i, s) in self.image_samples.iter_mut().enumerate() {
                let offset = if i % 2 == 0 { self.x_pos } else { self.y_pos };
                *s = offset as Float + rng.random_float()
            }
            for s in self.lens_samples.iter_mut() {
                *s = rng.random_float();
            }
            for s in self.time_samples.iter_mut() {
                *s = rng.random_float();
            }

            self.sample_pos = 0;
        }

        // Return next RandomSampler sample point
        sample.cam.image_x = self.image_samples[2 * self.sample_pos];
        sample.cam.image_y = self.image_samples[2 * self.sample_pos + 1];
        sample.cam.lens_u = self.lens_samples[2 * self.sample_pos];
        sample.cam.lens_v = self.lens_samples[2 * self.sample_pos + 1];
        sample.cam.time = lerp(self.time_samples[self.sample_pos], self.shutter_open, self.shutter_close);

        for (i, len) in sample.offsets_1d() {
            for j in 0..len {
                let ref mut foo = sample[i];
                foo[j] = rng.random_float();
            }
        }

        for (i, len) in sample.offsets_2d() {
            for j in 0..(2*len) {
                sample[i][j] = rng.random_float();
            }
        }

        self.sample_pos += 1;
        1
    }

    fn maximum_sample_count(&self) -> u32 {
        1
    }

    fn get_sub_sampler(&self, num: u32, count: u32) -> Option<Box<Sampler>> {
        let sub_window = self.window.compute_sub_window(num, count);
        if sub_window.is_empty() {
            None
        } else {
            Some(Box::new(RandomSampler::new(sub_window, self.num_samples, self.shutter_open, self.shutter_close)))
        }
    }

    fn round_size(&self, size: u32) -> u32 {
        size
    }
}