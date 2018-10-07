use core::light::Light;
use core::spectrum::Spectrum;
use core::geometry::Point3f;
use core::light::LightSample;
use core::light::VisibilityTester;
use core::geometry::Vector3f;
use core::types::{PI, Float};
use core::geometry::distance_squared;
use cgmath::prelude::*;
use core::scene::Scene;

pub struct PointLight {
    pos: Point3f,
    intensity: Spectrum,
}

impl PointLight {
    pub fn new(pos: Point3f, intensity: Spectrum) -> PointLight {
        PointLight { pos, intensity }
    }
}

impl Light for PointLight {
    fn sample_l(&self, p: &Point3f, p_epsilon: Float, _: &LightSample, time: Float, visibility: &mut VisibilityTester) -> (Spectrum, Vector3f, Float) {
        let wi = (self.pos - p).normalize();
        visibility.set_segment(*p, p_epsilon, self.pos, 0.0, time);
        let c = self.intensity / distance_squared(&self.pos, p);
        (c, wi, 1.0)
    }

    fn pdf(&self, p: &Point3f, wi: &Vector3f) -> Float {
        0.0
    }

    fn power(&self, scene: &Scene) -> Spectrum {
        4.0 * PI * self.intensity
    }

    fn num_samples(&self) -> u32 {
        1
    }

    fn is_delta_light(&self) -> bool {
        true
    }
}
