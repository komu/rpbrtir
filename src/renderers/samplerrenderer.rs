use core::integrator::SurfaceIntegrator;
use integrators::whitted::WhittedIntegrator;
use core::renderer::Renderer;
use core::scene::Scene;
use core::geometry::RayDifferential;
use core::sampler::Sample;
use core::rng::RNG;
use core::spectrum::Spectrum;
use core::types::Float;

pub struct SamplerRenderer {
    integrator: Box<SurfaceIntegrator>
}

impl SamplerRenderer {
    pub fn new() -> SamplerRenderer {
        SamplerRenderer {
            integrator: Box::new(WhittedIntegrator::new(50))
        }
    }
}

impl Renderer for SamplerRenderer {
    fn li(&self, scene: &Scene, rd: &mut RayDifferential, sample: Option<&Sample>, rng: &mut RNG) -> Spectrum {
        if let Some(mut isect) = scene.intersect(&mut rd.ray) {
            self.integrator.li(scene, self, rd, &mut isect, sample, rng)
        } else {
            scene.lights.iter().map(|l| { l.le(rd) }).sum()
        }

        // TODO: add contribution from volume integrator
    }

    fn transmittance(&self, scene: &Scene, rd: &RayDifferential, sample: Option<&Sample>, rng: &RNG) -> Float {
        1.0 // TODO
    }
}