use core::sampler::CameraSample;
use core::geometry::{Ray, RayDifferential};
use core::types::Float;
use core::film::Film;

pub trait Camera {
    fn generate_ray(&self, sample: &CameraSample) -> (Ray, Float);

    fn generate_ray_differential(&self, sample: &CameraSample) -> (RayDifferential, Float) {
        let (ray, wt) = self.generate_ray(sample);
        let (rx, wtx) = self.generate_ray(&CameraSample { image_x: sample.image_x + 1.0, ..*sample });
        let (ry, wty) = self.generate_ray(&CameraSample { image_y: sample.image_y + 1.0, ..*sample });

        if wtx == 0.0 || wty == 0.0 {
            return (RayDifferential::from_ray(ray), 0.0);
        }

        let rd = RayDifferential {
            ray,
            has_differentials: true,
            rx_origin: rx.o,
            rx_direction: rx.d,
            ry_origin: ry.o,
            ry_direction:ry.d
        };

        return (rd, wt);
    }

    fn get_film(&mut self) -> &mut Film;
}

pub trait ProjectiveCamera : Camera { }
