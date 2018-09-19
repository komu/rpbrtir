use core::scene::Scene;
use core::spectrum::Spectrum;
use core::geometry::RayDifferential;
use core::sampler::Sample;
use core::rng::RNG;
use core::types::Float;

pub trait Renderer {
    fn li(&self, scene: &Scene, rd: &mut RayDifferential, sample: Option<&Sample>, rng: &mut RNG) -> Spectrum;
    fn transmittance(&self, scene: &Scene, rd: &RayDifferential, sample: Option<&Sample>, rng: &RNG) -> Float;
}
