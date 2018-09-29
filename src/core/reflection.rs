use core::differential_geometry::DifferentialGeometry;
use core::spectrum::Spectrum;
use core::geometry::Vector3f;
use core::geometry::Normal;
use core::types::Float;
use cgmath::{vec3, prelude::*};
use core::types::INV_PI;
use core::rng::RNG;
use core::montecarlo::cosine_sample_hemisphere;
use core::math::floor_to_int;

pub struct BSDF<'a> {
    pub dg_shading: &'a DifferentialGeometry<'a>,
    nn: Normal,
    ng: Normal,
    sn: Vector3f,
    tn: Vector3f,
    pub eta: Float,
    bxdfs: Vec<Box<BxDF>>,
}

bitflags! {

    pub struct BxDFType : u8 {
        const BSDF_REFLECTION   = 0x01;
        const BSDF_TRANSMISSION = 0x02;
        const BSDF_DIFFUSE      = 0x04;
        const BSDF_GLOSSY       = 0x08;
        const BSDF_SPECULAR     = 0x10;
        const BSDF_ALL_TYPES        = Self::BSDF_DIFFUSE.bits | Self::BSDF_GLOSSY.bits | Self::BSDF_SPECULAR.bits;
        const BSDF_ALL_REFLECTION   = Self::BSDF_REFLECTION.bits | Self::BSDF_ALL_TYPES.bits;
        const BSDF_ALL_TRANSMISSION = Self::BSDF_TRANSMISSION.bits | Self::BSDF_ALL_TYPES.bits;
        const BSDF_ALL              = Self::BSDF_ALL_REFLECTION.bits | Self::BSDF_ALL_TRANSMISSION.bits;
    }
}

impl<'a> BSDF<'a> {
    pub fn new<'b>(dg_shading: &'b DifferentialGeometry<'b>, ng: Normal) -> BSDF<'b> {
        BSDF::new_with_eta(dg_shading, ng, 1.0)
    }

    pub fn new_with_eta<'b>(dg_shading: &'b DifferentialGeometry<'b>, ng: Normal, eta: Float) -> BSDF<'b> {
        let nn = dg_shading.nn;
        let sn = dg_shading.dpdu.normalize();
        let tn = nn.v.cross(sn);
        BSDF { dg_shading, ng, nn, sn, tn, eta, bxdfs: Vec::new() }
    }

    pub fn f_all(&self, wo_w: &Vector3f, wi_w: &Vector3f) -> Spectrum {
        self.f(wo_w, wi_w, BxDFType::BSDF_ALL)
    }

    pub fn f(&self, wo_w: &Vector3f, wi_w: &Vector3f, mut flags: BxDFType) -> Spectrum {
        let wi = self.world_to_local(wi_w);
        let wo = self.world_to_local(wo_w);

        if wi_w.dot(self.ng.v) * wo_w.dot(self.ng.v) > 0.0 {
            flags.remove(BxDFType::BSDF_TRANSMISSION);
        } else {
            flags.remove(BxDFType::BSDF_REFLECTION);
        }

        self.bxdfs.iter()
            .filter(|x| { x.matches_flags(flags) })
            .map(|x| { x.f(&wo, &wi) })
            .sum()
    }

    pub fn sample_f(&self, wo_w: &Vector3f, bsdf_sample: BSDFSample, mut flags: BxDFType) -> Option<(Spectrum, Vector3f, Float, BxDFType)> {
        // Choose which _BxDF_ to sample
        let matching_comps = self.num_components(flags);

        if matching_comps == 0 {
            return None
        }

        let which = floor_to_int(bsdf_sample.u_component * matching_comps as Float).min(matching_comps as i32 - 1) as usize;

        let bxdf = self.bxdfs.iter().filter(|x| { x.matches_flags(flags) }).nth(which).expect("no bxdf");

        // Sample chosen _BxDF_
        let wo = self.world_to_local(wo_w);
        let mut wi = Vector3f::unit_x();
        let (mut f, mut pdf) = bxdf.sample_f(wo, &mut wi, bsdf_sample.u_dir[0], bsdf_sample.u_dir[1]);

        if pdf == 0.0 {
            return None
        }

        let sampled_type = bxdf.bxdf_type();
        let wi_w = self.local_to_world(&wi);

        // Compute overall PDF with all matching _BxDF_s
        if !bxdf.bxdf_type().contains(BxDFType::BSDF_SPECULAR) && matching_comps > 1 {
            for (i, b) in self.bxdfs.iter().enumerate() {
                if i != which && b.matches_flags(flags) {
                    pdf += b.pdf(&wo, &wi);
                }
            }
        }

        if matching_comps > 1 {
            pdf /= matching_comps as Float;
        }

        // Compute value of BSDF for sampled direction
        if !bxdf.bxdf_type().contains(BxDFType::BSDF_SPECULAR) {
            f = Spectrum::black();

            if wi_w.dot(self.ng.v) * wo_w.dot(self.ng.v) > 0.0 {
                flags.remove(BxDFType::BSDF_TRANSMISSION);
            } else {
                flags.remove(BxDFType::BSDF_REFLECTION);
            }

            for b in &self.bxdfs {
                if b.matches_flags(flags) {
                    f += b.f(&wo, &wi);
                }
            }
        }

        return Some((f, wi_w, pdf, sampled_type))
    }

    fn num_components(&self, flags: BxDFType) -> usize {
        self.bxdfs.iter().filter(|x| { x.matches_flags(flags) }).count()
    }

    pub fn add(&mut self, bxdf: Box<BxDF>) {
        self.bxdfs.push(bxdf)
    }

    fn world_to_local(&self, v: &Vector3f) -> Vector3f {
        vec3(v.dot(self.sn), v.dot(self.tn), v.dot(self.nn.v))
    }

    fn local_to_world(&self, v: &Vector3f) -> Vector3f {
        vec3(self.sn.x * v.x + self.tn.x * v.y + self.nn.v.x * v.z,
             self.sn.y * v.x + self.tn.y * v.y + self.nn.v.y * v.z,
             self.sn.z * v.x + self.tn.z * v.y + self.nn.v.z * v.z)
    }
}

pub trait BxDF {
    fn matches_flags(&self, flags: BxDFType) -> bool {
//        true
        (self.bxdf_type().bits & flags.bits) == self.bxdf_type().bits
    }

    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum;
    fn sample_f(&self, wo: Vector3f, wi: &mut Vector3f, u1: Float, u2: Float) -> (Spectrum, Float);

    fn bxdf_type(&self) -> BxDFType;

    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> Float {
        if same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * INV_PI
        } else {
            0.0
        }
    }
}

pub struct Lambertian {
    r: Spectrum
}

impl Lambertian {
    pub fn new(r: Spectrum) -> Lambertian {
        Lambertian { r }
    }
}

impl BxDF for Lambertian {

    fn f(&self, _wo: &Vector3f, _wi: &Vector3f) -> Spectrum {
        self.r * INV_PI
    }

    fn sample_f(&self, wo: Vector3f, wi: &mut Vector3f, u1: Float, u2: Float) -> (Spectrum, Float) {
        // Cosine-sample the hemisphere, flipping the direction if necessary
        let wi2 = cosine_sample_hemisphere(u1, u2);
        wi.x = wi2.x;
        wi.y = wi2.y;
        wi.z = wi2.z;
        if wo.z < 0.0 {
            wi.z *= -1.0
        }
        let pdf = self.pdf(&wo, &wi);
        let spectrum = self.f(&wo, &wi);
        (spectrum, pdf)
    }

    fn bxdf_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_DIFFUSE
    }
}

pub struct BSDFSample {
    u_component: Float,
    u_dir: [Float; 2],
}

impl BSDFSample {
    pub fn new(rng: &mut RNG) -> BSDFSample {
        BSDFSample {
            u_component: rng.random_float(),
            u_dir: [rng.random_float(), rng.random_float()],
        }
    }
}

#[inline]
fn same_hemisphere(w: &Vector3f, wp: &Vector3f) -> bool {
    w.z * wp.z > 0.0
}

#[inline]
fn abs_cos_theta(w: &Vector3f) -> Float {
    w.z.abs()
}
