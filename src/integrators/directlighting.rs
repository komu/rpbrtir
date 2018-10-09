use core::{
    geometry::RayDifferential,
    integrator::{Integrator, SurfaceIntegrator,specular_reflect,specular_transmit, uniform_sample_one_light, uniform_sample_all_lights},
    intersection::Intersection,
    light::LightSampleOffsets,
    reflection::BSDFSampleOffsets,
    renderer::Renderer,
    rng::RNG,
    sampler::{Sample, Sampler, SampleOffset1d},
    scene::Scene,
    spectrum::Spectrum,
};

pub enum LightStrategy { SampleAllUniform, SampleOneUniform }

pub struct DirectLightingIntegrator {
    strategy: LightStrategy,
    max_depth: u32,

    light_sample_offsets: Vec<LightSampleOffsets>,
    bsdf_sample_offsets: Vec<BSDFSampleOffsets>,
    light_num_offset: Option<SampleOffset1d>,
}

impl DirectLightingIntegrator {
    pub fn new(strategy: LightStrategy, max_depth: u32) -> DirectLightingIntegrator {
        DirectLightingIntegrator {
            strategy,
            max_depth,
            light_sample_offsets: vec![],
            bsdf_sample_offsets: vec![],
            light_num_offset: None,
        }
    }
}

impl Default for DirectLightingIntegrator {
    fn default() -> Self {
        DirectLightingIntegrator::new(LightStrategy::SampleAllUniform, 5)
    }
}

impl SurfaceIntegrator for DirectLightingIntegrator {
    fn li(&self, scene: &Scene, renderer: &Renderer, rd: &RayDifferential, isect: &mut Intersection, sample: &Sample, rng: &mut RNG) -> Spectrum {
        let mut l = Spectrum::black();

        // Evaluate BSDF at hit point
        let bsdf = isect.get_bsdf(rd);
        let wo = -rd.ray.d;
        let p = bsdf.dg_shading.p;
        let n = bsdf.dg_shading.nn;

        // Compute emitted light if ray hit an area light source
        l += isect.le(wo);

        // Compute direct lighting for _DirectLightingIntegrator_ integrator
        if !scene.lights.is_empty() {
            l += match self.strategy {
                LightStrategy::SampleAllUniform =>
                    uniform_sample_all_lights(scene, renderer, &p, &n, &wo,
                                           isect.ray_epsilon, rd.ray.time, &bsdf, sample, rng,
                                           Some(&self.light_sample_offsets), Some(&self.bsdf_sample_offsets)),
                LightStrategy::SampleOneUniform =>
                    uniform_sample_one_light(scene, renderer, &p, &n, &wo,
                                          isect.ray_epsilon, rd.ray.time, &bsdf, sample, rng,
                                          self.light_num_offset, Some(&self.light_sample_offsets[0]), Some(&self.bsdf_sample_offsets[0]))
            }
        }

        if rd.ray.depth + 1 < self.max_depth {
            // Trace rays for specular reflection and refraction
            l += specular_reflect(rd, &bsdf, rng, isect, renderer, scene, Some(sample));
            l += specular_transmit(rd, &bsdf, rng, isect, renderer, scene, Some(sample));
        }

        l
    }
}

impl Integrator for DirectLightingIntegrator {
    fn request_samples(&mut self, sampler: Option<&Sampler>, sample: &mut Sample, scene: &Scene) {
        match self.strategy {
            LightStrategy::SampleAllUniform => {
                // Allocate and request samples for sampling all lights
                let n_lights = scene.lights.len();
                self.light_sample_offsets.reserve(n_lights);
                self.bsdf_sample_offsets.reserve(n_lights);

                for light in &scene.lights {
                    let mut n_samples = light.num_samples();
                    if let Some(sampler) = sampler {
                        n_samples = sampler.round_size(n_samples);
                    }

                    self.light_sample_offsets.push(LightSampleOffsets::new(n_samples, sample));
                    self.bsdf_sample_offsets.push(BSDFSampleOffsets::new(n_samples, sample));
                }
                self.light_num_offset = None;
            }
            LightStrategy::SampleOneUniform => {
                // Allocate and request samples for sampling one light
                self.light_sample_offsets.push(LightSampleOffsets::new(1, sample));
                self.light_num_offset = Some(sample.add_1d(1));
                self.bsdf_sample_offsets.push(BSDFSampleOffsets::new(1, sample));
            }
        }
    }
}