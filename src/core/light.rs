use core::{
    spectrum::Spectrum,
    geometry::{Point3f, Vector3f},
    types::Float,
    scene::Scene,
};
use core::sampler::Sample;
use core::renderer::Renderer;
use core::rng::RNG;
use core::geometry::{Ray, RayDifferential};
use cgmath::prelude::*;
use core::geometry::distance;

pub trait Light {
    fn sample_l(
        &self,
        p: &Point3f,
        p_epsilon: Float,
        ls: LightSample,
        time: Float,
        visibility: &mut VisibilityTester)
        -> (Spectrum, Vector3f, Float);

    fn le(&self, ray: &RayDifferential) -> Spectrum {
        Spectrum::black()
    }
}

pub struct LightSample {}

impl LightSample {
    pub fn new(rng: &RNG) -> LightSample {
        LightSample {}
    }
}

pub struct VisibilityTester {
    ray: Option<Ray>
}

impl VisibilityTester {
    pub fn new() -> VisibilityTester {
        VisibilityTester { ray: None }
    }

    pub fn unoccluded(&self, scene: &Scene) -> bool {
        !scene.intersect_p(self.ray())
    }

    pub fn transmittance(&self, scene: &Scene, renderer: &Renderer, sample: Option<&Sample>, rng: &RNG) -> Float {
        renderer.transmittance(scene, &RayDifferential::from_ray(self.ray().clone()), sample, rng)
    }

    fn ray(&self) -> &Ray {
        self.ray.as_ref().expect("no ray for VisibilityTester")
    }

    pub fn set_segment(&mut self, p1: Point3f, eps1: Float, p2: Point3f, eps2: Float, time: Float) {
        let dist = distance(&p1, &p2);
        self.ray = Some(Ray::new(p1, (p2 - p1) / dist, eps1, dist * (1.0 - eps2), time));
    }
}