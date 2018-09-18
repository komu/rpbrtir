use core::{
    scene::Scene,
    renderer::Renderer,
    geometry::Ray,
    intersection::Intersection,
    spectrum::Spectrum,
    sampler::Sample,
    reflection::BSDF,
    rng::RNG,
};

pub trait Integrator {}

type RayDifferential = Ray; // TODO

pub trait SurfaceIntegrator: Integrator {
    fn li(
        &self,
        scene: &Scene,
        renderer: Option<&Renderer>,
        ray: &RayDifferential,
        isect: &Intersection,
        sample: Option<&Sample>,
        rng: &RNG) -> Spectrum;
}

pub fn specular_reflect(ray: &RayDifferential, bsdf: &BSDF, rng: &RNG, isect: &Intersection, renderer: Option<&Renderer>, scene: &Scene, sample: Option<&Sample>) -> Spectrum {
    Spectrum::black() // TODO
}

pub fn specular_transmit(ray: &RayDifferential, bsdf: &BSDF, rng: &RNG, isect: &Intersection, renderer: Option<&Renderer>, scene: &Scene, sample: Option<&Sample>) -> Spectrum {
    Spectrum::black() // TODO
}