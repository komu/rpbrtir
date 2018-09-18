use core::light::Light;
use core::spectrum::Spectrum;
use core::geometry::Point3f;
use core::light::LightSample;
use core::light::VisibilityTester;
use core::geometry::Vector3f;
use core::types::Float;
use core::geometry::distance_squared;
use cgmath::prelude::*;

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
    fn sample_l(&self, p: &Point3f, p_epsilon: Float, _: LightSample, time: Float, visibility: &mut VisibilityTester) -> (Spectrum, Vector3f, Float) {
        let wi = (self.pos - p).normalize();
        let pdf = 1.0;
        visibility.set_segment(*p, p_epsilon, self.pos, 0.0, time);
        let intensity = self.intensity;
        let c = intensity / distance_squared(&self.pos, p);
        (c, wi, pdf)
    }
}
