use core::{
    geometry::RayDifferential,
    integrator::{Integrator, SurfaceIntegrator, uniform_sample_one_light},
    intersection::Intersection,
    light::LightSampleOffsets,
    reflection::{BxDFType, BSDFSampleOffsets, BSDFSample},
    renderer::Renderer,
    rng::RNG,
    sampler::{Sample, Sampler, SampleOffset1d},
    scene::Scene,
    spectrum::Spectrum,
};

use std::usize;
use cgmath::prelude::*;

const SAMPLE_DEPTH: usize = 3;

pub struct PathIntegrator {
    max_depth: usize,
    light_sample_offsets: Vec<LightSampleOffsets>,
    light_num_offset: Vec<SampleOffset1d>,
    bsdf_sample_offsets: Vec<BSDFSampleOffsets>,
    path_sample_offsets: Vec<BSDFSampleOffsets>,
}

impl PathIntegrator {
    pub fn new(max_depth: usize) -> PathIntegrator {
        PathIntegrator {
            max_depth,
            light_sample_offsets: Vec::with_capacity(SAMPLE_DEPTH),
            light_num_offset: Vec::with_capacity(SAMPLE_DEPTH),
            bsdf_sample_offsets: Vec::with_capacity(SAMPLE_DEPTH),
            path_sample_offsets: Vec::with_capacity(SAMPLE_DEPTH),
        }
    }
}

impl Default for PathIntegrator {
    fn default() -> Self {
        PathIntegrator::new(5)
    }
}

impl SurfaceIntegrator for PathIntegrator {
    fn li(&self, scene: &Scene, renderer: &Renderer, r: &RayDifferential, isect: &mut Intersection, sample: &Sample, rng: &mut RNG) -> Spectrum {
        // Declare common path integration variables
        let mut path_throughput = Spectrum::white();
        let mut L = Spectrum::black();
        let mut ray: RayDifferential = r.clone();
        let mut specular_bounce = false;
        let mut isectp = isect.clone();

        for bounces in 0..usize::MAX {
            // Possibly add emitted light at path vertex
            if bounces == 0 || specular_bounce {
                L += path_throughput * isectp.le(-ray.ray.d);
            }
            {
                // Sample illumination from lights to find path contribution
                let bsdf = isectp.get_bsdf(&ray);
                let p = bsdf.dg_shading.p;
                let n = bsdf.dg_shading.nn;
                let wo = -ray.ray.d;
                if bounces < SAMPLE_DEPTH {
                    L += path_throughput *
                        uniform_sample_one_light(scene, renderer, &p, &n, &wo, isectp.ray_epsilon, ray.ray.time,
                                                 &bsdf, sample, rng,
                                                 Some(self.light_num_offset[bounces]),
                                                 Some(&self.light_sample_offsets[bounces]),
                                                 Some(&self.bsdf_sample_offsets[bounces]));
                } else {
                    L += path_throughput *
                        uniform_sample_one_light(scene, renderer, &p, &n, &wo, isectp.ray_epsilon, ray.ray.time,
                                                 &bsdf, sample, rng, None, None, None);
                }

                // Sample BSDF to get new path direction

                // Get _outgoingBSDFSample_ for sampling new path direction
                let outgoing_bsdfsample = if bounces < SAMPLE_DEPTH {
                    BSDFSample::new(sample, &self.path_sample_offsets[bounces], 0)
                } else {
                    BSDFSample::gen(rng)
                };

                if let Some((f, wi, pdf, flags)) = bsdf.sample_f(&wo, &outgoing_bsdfsample, BxDFType::BSDF_ALL) {
                    if f.is_black() || pdf == 0. {
                        break;
                    }

                    specular_bounce = flags.contains(BxDFType::BSDF_SPECULAR);
                    path_throughput *= f * wi.dot(n.v).abs() / pdf;
                    ray = RayDifferential::new_with_parent(p, wi, &ray.ray, isectp.ray_epsilon);

                    // Possibly terminate the path
                    if bounces > 3 {
                        let continue_probability = path_throughput.y().min(0.5);
                        if rng.random_float() > continue_probability {
                            break;
                        }
                        path_throughput /= continue_probability;
                    }
                    if bounces == self.max_depth {
                        break;
                    }
                }
            }

            // Find next vertex of path
            if let Some(localIsect) = scene.intersect(&mut ray.ray) {
                path_throughput *= renderer.transmittance(scene, &ray, None, rng);
                isectp = localIsect.clone();
            } else {
                if specular_bounce {
                    for light in &scene.lights {
                        L += path_throughput * light.le(&ray);
                    }
                }
                break;
            }
        }
        return L;
    }
}

impl Integrator for PathIntegrator {
    fn request_samples(&mut self, _sampler: Option<&Sampler>, sample: &mut Sample, _scene: &Scene) {
        for i in 0..SAMPLE_DEPTH {
            self.light_sample_offsets.push(LightSampleOffsets::new(1, sample));
            self.light_num_offset.push(sample.add_1d(1));
            self.bsdf_sample_offsets.push(BSDFSampleOffsets::new(1, sample));
            self.path_sample_offsets.push(BSDFSampleOffsets::new(1, sample));
        }
    }
}
