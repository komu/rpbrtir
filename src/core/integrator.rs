use core::{
    scene::Scene,
    renderer::Renderer,
    geometry::{Ray, RayDifferential, RayDifferentials},
    intersection::Intersection,
    spectrum::Spectrum,
    sampler::Sample,
    reflection::{BSDF, BxDFType, BSDFSample},
    rng::RNG,
    types::Float
};
use cgmath::prelude::*;

pub trait Integrator {}

pub trait SurfaceIntegrator: Integrator {
    fn li(
        &self,
        scene: &Scene,
        renderer: &Renderer,
        rd: &RayDifferential,
        isect: &mut Intersection,
        sample: Option<&Sample>,
        rng: &mut RNG) -> Spectrum;
}

pub trait VolumeIntegrator {
    fn transmittance(&self, scene: &Scene, renderer: &Renderer, ray: &RayDifferential, sample: Option<&Sample>, rng: &RNG) -> Float;
    fn li(&self, scene: &Scene, renderer: &Renderer, ray: &RayDifferential, sample: Option<&Sample>, rng: &RNG) -> (Spectrum, Spectrum);
}

pub struct NoOpVolumeIntegrator {}

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

    if let Some((f, wi, pdf, _)) = bsdf.sample_f(&wo, BSDFSample::new(rng), BxDFType::BSDF_REFLECTION | BxDFType::BSDF_SPECULAR) {
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

    if let Some((f, wi, pdf, _)) = bsdf.sample_f(&wo, BSDFSample::new(rng), BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_SPECULAR) {
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