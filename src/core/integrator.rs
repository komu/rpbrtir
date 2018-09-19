use core::{
    scene::Scene,
    renderer::Renderer,
    geometry::RayDifferential,
    intersection::Intersection,
    spectrum::Spectrum,
    sampler::Sample,
    reflection::{BSDF, BxDFType, BSDFSample},
    rng::RNG,
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

pub fn specular_reflect(rd1: &RayDifferential, bsdf: &BSDF, rng: &mut RNG, isect: &Intersection, renderer: &Renderer, scene: &Scene, sample: Option<&Sample>) -> Spectrum {
    let ray = &rd1.ray;
    let wo = -ray.d;
    let p = bsdf.dg_shading.p;
    let n = &bsdf.dg_shading.nn;

    if let (f, Some(wi), pdf, _) = bsdf.sample_f(&wo, BSDFSample::new(rng), BxDFType::BSDF_REFLECTION | BxDFType::BSDF_SPECULAR) {
        if pdf > 0.0 && !f.is_black() && wi.dot(n.v).abs() != 0.0 {
            // TODO: Compute ray differential _rd_ for specular reflection
            let mut rd = RayDifferential::new_with_parent(p, wi, ray, isect.ray_epsilon);
            /*
            RayDifferential rd(p, wi, ray, isect.rayEpsilon);
            if (ray.hasDifferentials) {
                rd.hasDifferentials = true;
                rd.rxOrigin = p + isect.dg.dpdx;
                rd.ryOrigin = p + isect.dg.dpdy;
                // Compute differential reflected directions
                Normal dndx = bsdf->dgShading.dndu * bsdf->dgShading.dudx +
                              bsdf->dgShading.dndv * bsdf->dgShading.dvdx;
                Normal dndy = bsdf->dgShading.dndu * bsdf->dgShading.dudy +
                              bsdf->dgShading.dndv * bsdf->dgShading.dvdy;
                Vector dwodx = -ray.rxDirection - wo, dwody = -ray.ryDirection - wo;
                float dDNdx = Dot(dwodx, n) + Dot(wo, dndx);
                float dDNdy = Dot(dwody, n) + Dot(wo, dndy);
                rd.rxDirection = wi - dwodx + 2 * Vector(Dot(wo, n) * dndx +
                                                         dDNdx * n);
                rd.ryDirection = wi - dwody + 2 * Vector(Dot(wo, n) * dndy +
                                                         dDNdy * n);
            }
            */

            let li = renderer.li(scene, &mut rd, sample, rng);
            return f * li * wi.dot(n.v).abs() / pdf
        }
    }

    Spectrum::black()
}

pub fn specular_transmit(ray: &RayDifferential, bsdf: &BSDF, rng: &mut RNG, isect: &Intersection, renderer: &Renderer, scene: &Scene, sample: Option<&Sample>) -> Spectrum {
    Spectrum::black() // TODO

    /*
        Vector wo = -ray.d, wi;
    float pdf;
    const Point &p = bsdf->dgShading.p;
    const Normal &n = bsdf->dgShading.nn;
    Spectrum f = bsdf->Sample_f(wo, &wi, BSDFSample(rng), &pdf,
                               BxDFType(BSDF_TRANSMISSION | BSDF_SPECULAR));
    Spectrum L = 0.f;
    if (pdf > 0.f && !f.IsBlack() && AbsDot(wi, n) != 0.f) {
        // Compute ray differential _rd_ for specular transmission
        RayDifferential rd(p, wi, ray, isect.rayEpsilon);
        if (ray.hasDifferentials) {
            rd.hasDifferentials = true;
            rd.rxOrigin = p + isect.dg.dpdx;
            rd.ryOrigin = p + isect.dg.dpdy;

            float eta = bsdf->eta;
            Vector w = -wo;
            if (Dot(wo, n) < 0) eta = 1.f / eta;

            Normal dndx = bsdf->dgShading.dndu * bsdf->dgShading.dudx + bsdf->dgShading.dndv * bsdf->dgShading.dvdx;
            Normal dndy = bsdf->dgShading.dndu * bsdf->dgShading.dudy + bsdf->dgShading.dndv * bsdf->dgShading.dvdy;

            Vector dwodx = -ray.rxDirection - wo, dwody = -ray.ryDirection - wo;
            float dDNdx = Dot(dwodx, n) + Dot(wo, dndx);
            float dDNdy = Dot(dwody, n) + Dot(wo, dndy);

            float mu = eta * Dot(w, n) - Dot(wi, n);
            float dmudx = (eta - (eta*eta*Dot(w,n))/Dot(wi, n)) * dDNdx;
            float dmudy = (eta - (eta*eta*Dot(w,n))/Dot(wi, n)) * dDNdy;

            rd.rxDirection = wi + eta * dwodx - Vector(mu * dndx + dmudx * n);
            rd.ryDirection = wi + eta * dwody - Vector(mu * dndy + dmudy * n);
        }
        PBRT_STARTED_SPECULAR_REFRACTION_RAY(const_cast<RayDifferential *>(&rd));
        Spectrum Li = renderer->Li(scene, rd, sample, rng, arena);
        L = f * Li * AbsDot(wi, n) / pdf;
        PBRT_FINISHED_SPECULAR_REFRACTION_RAY(const_cast<RayDifferential *>(&rd));
    }
    return L;
    */
}