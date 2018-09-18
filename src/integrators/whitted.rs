use core::{
    integrator::{Integrator, SurfaceIntegrator, specular_reflect, specular_transmit},
    scene::Scene,
    renderer::Renderer,
    geometry::Ray,
    intersection::Intersection,
    light::{LightSample, VisibilityTester},
    sampler::Sample,
    spectrum::Spectrum,
    rng::RNG,
};
use cgmath::prelude::*;

pub struct WhittedIntegrator {
    max_depth: u32
}

impl WhittedIntegrator {
    pub fn new(max_depth: u32) -> WhittedIntegrator {
        WhittedIntegrator { max_depth }
    }
}

impl Integrator for WhittedIntegrator {}

impl SurfaceIntegrator for WhittedIntegrator {
    fn li(
        &self,
        scene: &Scene,
        renderer: Option<&Renderer>,
        ray: &Ray,
        isect: &Intersection,
        sample: Option<&Sample>,
        rng: &RNG) -> Spectrum {

        let mut l = Spectrum::black();

        // Compute emitted and reflected light at ray intersection point

        // Evaluate BSDF at hit point
        let bsdf = isect.get_bsdf(ray);

        // Initialize common variables for Whitted integrator
        let p = bsdf.dg_shading.p;
        let n = bsdf.dg_shading.nn;
        let wo = -ray.d;

        // Compute emitted light if ray hit an area light source
        l += isect.le(wo);

        // Add contribution of each light source
        for light in &scene.lights {
            let mut visibility = VisibilityTester::new();
            let (li, wi, pdf) = light.sample_l(&p, isect.ray_epsilon, LightSample::new(rng), ray.time, &mut visibility);
            if li.is_black() || pdf == 0.0 {
                continue;
            }

            let f = bsdf.f_all(&wo, &wi);
            if !f.is_black() && visibility.unoccluded(&scene) {
                l += f * li * wi.dot(n.v).abs() * visibility.transmittance(scene, renderer, sample, rng) / pdf;
            }
        }
        if ray.depth + 1 < self.max_depth {
            // Trace rays for specular reflection and refraction
            l += specular_reflect(ray, &bsdf, rng, isect, renderer, scene, sample);
            l += specular_transmit(ray, &bsdf, rng, isect, renderer, scene, sample);
        }
        l
    }
}