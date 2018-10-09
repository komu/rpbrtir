use core::{
    scene::Scene,
    renderer::Renderer,
    geometry::{Ray, RayDifferential, RayDifferentials, Point3f, Normal, Vector3f},
    intersection::Intersection,
    light::{Light, LightSample, LightSampleOffsets, VisibilityTester, AreaLight},
    montecarlo::{power_heuristic},
    spectrum::Spectrum,
    sampler::{Sample, Sampler},
    reflection::{BSDF, BxDFType, BSDFSample, BSDFSampleOffsets},
    rng::RNG,
    types::{Float, INFINITY},
};
use cgmath::prelude::*;
use core::sampler::SampleOffset1d;

pub trait Integrator {
    fn request_samples(&mut self, sampler: Option<&Sampler>, sample: &mut Sample, scene: &Scene) {}
}

pub trait SurfaceIntegrator: Integrator {
    fn li(
        &self,
        scene: &Scene,
        renderer: &Renderer,
        rd: &RayDifferential,
        isect: &mut Intersection,
        sample: &Sample,
        rng: &mut RNG) -> Spectrum;
}

pub trait VolumeIntegrator: Integrator {
    fn transmittance(&self, scene: &Scene, renderer: &Renderer, ray: &RayDifferential, sample: Option<&Sample>, rng: &RNG) -> Float;
    fn li(&self, scene: &Scene, renderer: &Renderer, ray: &RayDifferential, sample: Option<&Sample>, rng: &RNG) -> (Spectrum, Spectrum);
}

pub struct NoOpVolumeIntegrator {}

impl Integrator for NoOpVolumeIntegrator { }

impl VolumeIntegrator for NoOpVolumeIntegrator {
    fn transmittance(&self, _scene: &Scene, _renderer: &Renderer, _ray: &RayDifferential, _sample: Option<&Sample>, _rng: &RNG) -> Float {
        1.0
    }

    fn li(&self, _scene: &Scene, _renderer: &Renderer, _ray: &RayDifferential, _sample: Option<&Sample>, _rng: &RNG) -> (Spectrum, Spectrum) {
        (Spectrum::black(), Spectrum::white())
    }
}

#[allow(non_snake_case)]
pub fn specular_reflect(rd1: &RayDifferential, bsdf: &BSDF, rng: &mut RNG, isect: &Intersection, renderer: &Renderer, scene: &Scene, sample: Option<&Sample>) -> Spectrum {
    let ray = &rd1.ray;
    let wo = -ray.d;
    let p = bsdf.dg_shading.p;
    let n = &bsdf.dg_shading.nn;

    if let Some((f, wi, pdf, _)) = bsdf.sample_f(&wo, &BSDFSample::gen(rng), BxDFType::BSDF_REFLECTION | BxDFType::BSDF_SPECULAR) {
        if pdf > 0.0 && !f.is_black() && wi.dot(n.v).abs() != 0.0 {
            let mut rd = RayDifferential {
                ray: Ray::new_with_parent(p, wi, ray, isect.ray_epsilon),
                differentials: match &rd1.differentials {
                    Some(r) => {
                        let isect_dg = isect.dg.differentials.borrow();
                        let shading_dg = bsdf.dg_shading.differentials.borrow();

                        // Compute differential reflected directions
                        let dndx = bsdf.dg_shading.dndu.v * shading_dg.dudx + bsdf.dg_shading.dndv.v * shading_dg.dvdx;
                        let dndy = bsdf.dg_shading.dndu.v * shading_dg.dudy + bsdf.dg_shading.dndv.v * shading_dg.dvdy;
                        let dwodx = -r.rx_direction - wo;
                        let dwody = -r.ry_direction - wo;
                        let dDNdx = dwodx.dot(n.v) + wo.dot(dndx);
                        let dDNdy = dwody.dot(n.v) + wo.dot(dndy);

                        Some(RayDifferentials {
                            rx_origin: p + isect_dg.dpdx,
                            ry_origin: p + isect_dg.dpdy,
                            rx_direction: wi - dwodx + 2.0 * (wo.dot(n.v) * dndx + dDNdx * n.v),
                            ry_direction: wi - dwody + 2.0 * (wo.dot(n.v) * dndy + dDNdy * n.v),
                        })
                    }
                    None => None
                },
            };

            let li = renderer.li(scene, &mut rd, sample, rng);
            return f * li * wi.dot(n.v).abs() / pdf;
        }
    }

    Spectrum::black()
}

#[allow(non_snake_case)]
pub fn specular_transmit(rd1: &RayDifferential, bsdf: &BSDF, rng: &mut RNG, isect: &Intersection, renderer: &Renderer, scene: &Scene, sample: Option<&Sample>) -> Spectrum {
    let ray = &rd1.ray;
    let wo = -ray.d;
    let p = bsdf.dg_shading.p;
    let n = &bsdf.dg_shading.nn;

    if let Some((f, wi, pdf, _)) = bsdf.sample_f(&wo, &BSDFSample::gen(rng), BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_SPECULAR) {
        if pdf > 0.0 && !f.is_black() && wi.dot(n.v).abs() != 0.0 {
            // Compute ray differential _rd_ for specular transmission
            let mut rd = RayDifferential {
                ray: Ray::new_with_parent(p, wi, ray, isect.ray_epsilon),
                differentials: if rd1.differentials.is_some() {
                    let w = -wo;
                    let eta = if wo.dot(n.v) < 0.0 { 1.0 / bsdf.eta } else { bsdf.eta };
                    let mut eta = bsdf.eta;

                    let isect_dg = isect.dg.differentials.borrow();
                    let shading_dg = bsdf.dg_shading.differentials.borrow();

                    let dndx = bsdf.dg_shading.dndu.v * shading_dg.dudx + bsdf.dg_shading.dndv.v * shading_dg.dvdx;
                    let dndy = bsdf.dg_shading.dndu.v * shading_dg.dudy + bsdf.dg_shading.dndv.v * shading_dg.dvdy;

                    let diffs = rd1.differentials.as_ref().unwrap();
                    let dwodx = -diffs.rx_direction - wo;
                    let dwody = -diffs.ry_direction - wo;
                    let dDNdx = dwodx.dot(n.v) + wo.dot(dndx);
                    let dDNdy = dwody.dot(n.v) + wo.dot(dndy);

                    let mu = eta * w.dot(n.v) - wi.dot(n.v);
                    let dmudx = (eta - (eta * eta * w.dot(n.v)) / wi.dot(n.v)) * dDNdx;
                    let dmudy = (eta - (eta * eta * w.dot(n.v)) / wi.dot(n.v)) * dDNdy;

                    Some(RayDifferentials {
                        rx_origin: p + isect_dg.dpdx,
                        ry_origin: p + isect_dg.dpdy,
                        rx_direction: wi + eta * dwodx - (mu * dndx + dmudx * n.v),
                        ry_direction: wi + eta * dwody - (mu * dndy + dmudy * n.v),
                    })
                } else {
                    None
                },
            };

            let li = renderer.li(scene, &mut rd, sample, rng);
            return f * li * wi.dot(n.v).abs() / pdf;
        }
    }

    Spectrum::black()
}

pub fn uniform_sample_all_lights(scene: &Scene,
                                 renderer: &Renderer,
                                 p: &Point3f,
                                 n: &Normal,
                                 wo: &Vector3f,
                                 ray_epsilon: Float,
                                 time: Float,
                                 bsdf: &BSDF,
                                 sample: &Sample,
                                 rng: &mut RNG,
                                 light_sample_offsets: Option<&[LightSampleOffsets]>,
                                 bsdf_sample_offsets: Option<&[BSDFSampleOffsets]>) -> Spectrum {

    let mut l = Spectrum::black();

    for (i, light) in scene.lights.iter().enumerate() {
        let n_samples = light_sample_offsets.map_or(1, |o| o[i].n_samples) as usize;

        // Estimate direct lighting from _light_ samples
        let mut ld = Spectrum::black();

        for j in 0..n_samples {
            // Find light and BSDF sample values for direct lighting estimate
            let light_sample: LightSample;
            let bsdf_sample: BSDFSample;
            if light_sample_offsets.is_some() && bsdf_sample_offsets.is_some() {
                light_sample = LightSample::new(&sample, &light_sample_offsets.unwrap()[i], j);
                bsdf_sample = BSDFSample::new(sample, &bsdf_sample_offsets.unwrap()[i], j);
            } else {
                light_sample = LightSample::gen(rng);
                bsdf_sample = BSDFSample::gen(rng);
            }

            ld += estimate_direct(scene, renderer, light.as_ref(), p, n, wo,
                                  ray_epsilon, time, bsdf, rng, &light_sample, &bsdf_sample,
                                  BxDFType::BSDF_ALL & !BxDFType::BSDF_SPECULAR);
        }

        l += ld / (n_samples as Float);
    }

    l
}

pub fn uniform_sample_one_light(scene: &Scene,
                                renderer: &Renderer,
                                p: &Point3f,
                                n: &Normal,
                                wo: &Vector3f,
                                ray_epsilon: Float,
                                time: Float,
                                bsdf: &BSDF,
                                sample: &Sample,
                                rng: &mut RNG,
                                light_num_offset: Option<SampleOffset1d>,
                                light_sample_offset: Option<&LightSampleOffsets>,
                                bsdf_sample_offset: Option<&BSDFSampleOffsets>) -> Spectrum {
    // Randomly choose a single light to sample, _light_
    let n_lights = scene.lights.len();
    if n_lights == 0 {
        return Spectrum::black();
    }

    let mut light_num = if let Some(offset) = light_num_offset {
        (sample[offset][0] * (n_lights as Float)).floor() as usize
    } else {
        (rng.random_float() * (n_lights as Float)).floor() as usize
    };

    light_num = light_num.min(n_lights - 1);
    let light = &scene.lights[light_num];

    // Initialize light and bsdf samples for single light sample
    let light_sample: LightSample;
    let bsdf_sample: BSDFSample;
    if light_sample_offset.is_some() && bsdf_sample_offset.is_some() {
        light_sample = LightSample::new(sample, light_sample_offset.unwrap(), 0);
        bsdf_sample = BSDFSample::new(sample, bsdf_sample_offset.unwrap(), 0);
    } else {
        light_sample = LightSample::gen(rng);
        bsdf_sample = BSDFSample::gen(rng);
    }

    return (n_lights as Float) *
        estimate_direct(scene, renderer, light.as_ref(), p, n, wo,
                        ray_epsilon, time, bsdf, rng, &light_sample,
                        &bsdf_sample, BxDFType::BSDF_ALL & !BxDFType::BSDF_SPECULAR);

}

fn estimate_direct(scene: &Scene,
                   renderer: &Renderer,
                   light: &Light,
                   p: &Point3f,
                   n: &Normal,
                   wo: &Vector3f,
                   ray_epsilon: Float,
                   time: Float,
                   bsdf: &BSDF,
                   rng: &RNG,
                   light_sample: &LightSample,
                   bsdf_sample: &BSDFSample,
                   flags: BxDFType) -> Spectrum {

    let mut ld = Spectrum::black();

    // Sample light source with multiple importance sampling
    let mut visibility = VisibilityTester::new();
    let (mut li, wi, light_pdf) = light.sample_l(p, ray_epsilon, light_sample, time, &mut visibility);

    if light_pdf > 0.0 && !li.is_black() {
        let f = bsdf.f(wo, &wi, flags);
        if !f.is_black() && visibility.unoccluded(scene) {
            // Add light's contribution to reflected radiance
            li *= visibility.transmittance(scene, renderer, None, rng);
            if light.is_delta_light() {
                ld += f * li * (wi.dot(n.v).abs() / light_pdf);
            } else {
                let bsdf_pdf = bsdf.pdf(wo, &wi, flags);
                let weight = power_heuristic(1, light_pdf, 1, bsdf_pdf);
                ld += f * li * (wi.dot(n.v).abs() * weight / light_pdf);
            }
        }
    }

    // Sample BSDF with multiple importance sampling

    if !light.is_delta_light() {
        if let Some((f, wi, bsdf_pdf, sampled_type)) = bsdf.sample_f(wo, bsdf_sample, flags) {
            if !f.is_black() && bsdf_pdf > 0.0 {
                let mut weight = 1.0;

                if !(sampled_type.contains(BxDFType::BSDF_SPECULAR)) {
                    let light_pdf = light.pdf(p, &wi);
                    if light_pdf == 0.0 {
                        return ld;
                    }
                    weight = power_heuristic(1, bsdf_pdf, 1, light_pdf);
                }

                // Add light contribution from BSDF sampling

                let mut li = Spectrum::black();
                let mut ray = RayDifferential::new(*p, wi, ray_epsilon, INFINITY, time);
                if let Some(light_isect) = scene.intersect(&mut ray.ray) {
                    let area_light = light_isect.primitive.get_area_light();
                    if area_light.is_some() && is_same_light(light, area_light.unwrap()) {
                        li = light_isect.le(-wi);
                    }
                } else {
                    li = light.le(&ray);
                }

                if !li.is_black() {
                    li *= renderer.transmittance(scene, &ray, None, rng);
                    ld += f * li * wi.dot(n.v).abs() * weight / bsdf_pdf;
                }
            }
        }
    }
    ld
}

fn is_same_light(l1: &Light, l2: &AreaLight) -> bool {
    unimplemented!()
}