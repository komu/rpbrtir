use core::differential_geometry::DifferentialGeometry;
use core::spectrum::Spectrum;
use core::geometry::{Normal, Vector3f};
use cgmath::{vec3, prelude::*};
use core::types::{Float, INV_PI, INV_TWO_PI};
use core::rng::RNG;
use core::montecarlo::cosine_sample_hemisphere;
use core::math::floor_to_int;
use core::types::PI;
use core::geometry::spherical_direction;
use core::math::clamp;

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
            return None;
        }

        let which = floor_to_int(bsdf_sample.u_component * matching_comps as Float).min(matching_comps as i32 - 1) as usize;

        let bxdf = self.bxdfs.iter().filter(|x| { x.matches_flags(flags) }).nth(which).expect("no bxdf");

        // Sample chosen _BxDF_
        let wo = self.world_to_local(wo_w);
        let mut wi = Vector3f::unit_x();
        let (mut f, mut pdf) = bxdf.sample_f(wo, &mut wi, bsdf_sample.u_dir[0], bsdf_sample.u_dir[1]);

        if pdf == 0.0 {
            return None;
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

        return Some((f, wi_w, pdf, sampled_type));
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

pub struct SpecularReflection {
    r: Spectrum,
    fresnel: Box<Fresnel>,
}

impl SpecularReflection {
    pub fn new(r: Spectrum, fresnel: Box<Fresnel>) -> SpecularReflection {
        SpecularReflection { r, fresnel }
    }
}

impl BxDF for SpecularReflection {
    fn f(&self, _wo: &Vector3f, _wi: &Vector3f) -> Spectrum {
        Spectrum::black()
    }

    fn sample_f(&self, wo: Vector3f, wi: &mut Vector3f, u1: Float, u2: Float) -> (Spectrum, Float) {
        // Compute perfect specular reflection direction
        *wi = vec3(-wo.x, -wo.y, wo.z);
        let l = self.fresnel.evaluate(cos_theta(&wo)) * self.r / abs_cos_theta(wi);
        (l, 1.0)
    }

    fn bxdf_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_SPECULAR
    }

    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> Float {
        0.0
    }
}

pub struct SpecularTransmission {
    t: Spectrum,
    etai: Float,
    etat: Float,
    fresnel: FresnelDielectric,
}

impl SpecularTransmission {
    pub fn new(t: Spectrum, etai: Float, etat: Float) -> SpecularTransmission {
        SpecularTransmission {
            t,
            etai,
            etat,
            fresnel: FresnelDielectric::new(etai, etat),
        }
    }
}

impl BxDF for SpecularTransmission {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        Spectrum::black()
    }

    fn sample_f(&self, wo: Vector3f, wi: &mut Vector3f, u1: Float, u2: Float) -> (Spectrum, Float) {
        // Figure out which $\eta$ is incident and which is transmitted
        let entering = cos_theta(&wo) > 0.0;
        let (ei, et) = if entering { (self.etai, self.etat) } else { (self.etat, self.etai) };

        // Compute transmitted ray direction
        let sini2 = sin_theta2(&wo);
        let eta = ei / et;
        let sint2 = eta * eta * sini2;

        // Handle total internal reflection for transmission
        if sint2 >= 1.0 {
            return (Spectrum::black(), 0.0);
        }

        let mut cost = (1.0 - sint2).max(0.0).sqrt();
        if entering {
            cost = -cost;
        }

        let sint_over_sini = eta;
        *wi = Vector3f::new(sint_over_sini * -wo.x, sint_over_sini * -wo.y, cost);

        let f = self.fresnel.evaluate(cos_theta(&wo));
        let l = /*(ei*ei)/(et*et) * */ (Spectrum::white() - f) * self.t / abs_cos_theta(wi);

        (l, 1.0)
    }

    fn bxdf_type(&self) -> BxDFType {
        BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_SPECULAR
    }

    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> Float {
        0.0
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

pub trait MicrofacetDistribution {
    fn d(&self, wh: &Vector3f) -> Float;
    fn sample_f(&self, wo: &Vector3f, u1: Float, u2: Float) -> (Vector3f, Float);
    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> Float;
}

pub struct Blinn {
    exponent: Float
}

impl Blinn {
    pub fn new(exponent: Float) -> Blinn {
        Blinn { exponent }
    }
}

impl MicrofacetDistribution for Blinn {
    fn d(&self, wh: &Vector3f) -> Float {
        let costhetah = abs_cos_theta(wh);
        return (self.exponent + 2.0) * INV_TWO_PI * costhetah.powf(self.exponent);
    }

    fn sample_f(&self, wo: &Vector3f, u1: Float, u2: Float) -> (Vector3f, Float) {
        // Compute sampled half-angle vector $\wh$ for Blinn distribution
        let costheta = u1.powf(1.0 / (self.exponent + 1.0));
        let sintheta = (1.0 - costheta * costheta).max(0.0).sqrt();
        let phi = u2 * 2.0 * PI;
        let mut wh = spherical_direction(sintheta, costheta, phi);
        if !same_hemisphere(wo, &wh) {
            wh = -wh;
        }

        // Compute incident direction by reflecting about $\wh$
        let wi = -1.0 * wo + 2.0 * wo.dot(wh) * wh;

        // Compute PDF for $\wi$ from Blinn distribution
        let mut blinn_pdf = ((self.exponent + 1.0) * costheta.powf(self.exponent)) / (2.0 * PI * 4.0 * wo.dot(wh));
        if wo.dot(wh) <= 0.0 {
            blinn_pdf = 0.0;
        }

        (wi, blinn_pdf)
    }

    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> Float {
        let wh = (wo + wi).normalize();
        let costheta = abs_cos_theta(&wh);
        // Compute PDF for $\wi$ from Blinn distribution
        let mut blinn_pdf = ((self.exponent + 1.0) * costheta.powf(self.exponent)) / (2.0 * PI * 4.0 * wo.dot(wh));
        if wo.dot(wh) <= 0.0 {
            blinn_pdf = 0.0;
        }
        return blinn_pdf;
    }
}

pub trait Fresnel {
    fn evaluate(&self, cosi: Float) -> Spectrum;
}

pub struct FresnelConductor {
    eta: Spectrum,
    k: Spectrum,
}

impl FresnelConductor {
    pub fn new(eta: Spectrum, k: Spectrum) -> FresnelConductor {
        FresnelConductor { eta, k }
    }
}

impl Fresnel for FresnelConductor {
    fn evaluate(&self, cosi: Float) -> Spectrum {
        fr_cond(cosi.abs(), self.eta, &self.k)
    }
}

pub struct FresnelDielectric {
    eta_i: Float,
    eta_t: Float,
}

impl FresnelDielectric {
    pub fn new(eta_i: Float, eta_t: Float) -> FresnelDielectric {
        FresnelDielectric { eta_i, eta_t }
    }
}

impl Fresnel for FresnelDielectric {
    fn evaluate(&self, cosi: Float) -> Spectrum {
        // Compute Fresnel reflectance for dielectric
        let cosi = clamp(cosi, -1.0, 1.0);

        // Compute indices of refraction for dielectric
        let entering = cosi > 0.0;
        let (ei, et) = if entering { (self.eta_i, self.eta_t) } else { (self.eta_t, self.eta_i) };

        // Compute _sint_ using Snell's law
        let sint = ei / et * (1.0 - cosi * cosi).max(0.0).sqrt();
        if sint >= 1.0 {
            // Handle total internal reflection
            Spectrum::white()
        } else {
            let cost = (1.0 - sint * sint).max(0.0).sqrt();
            fr_diel(cosi.abs(), cost, Spectrum::from(ei), Spectrum::from(et))
        }
    }
}

pub struct FresnelNoOp {}

impl Default for FresnelNoOp {
    fn default() -> Self {
        FresnelNoOp { }
    }
}

impl Fresnel for FresnelNoOp {
    fn evaluate(&self, _cosi: Float) -> Spectrum {
        Spectrum::white()
    }
}

fn fr_diel(cosi: Float, cost: Float, etai: Spectrum, etat: Spectrum) -> Spectrum {
    let rparl = ((etat * cosi) - (etai * cost)) / ((etat * cosi) + (etai * cost));
    let rperp = ((etai * cosi) - (etat * cost)) / ((etai * cosi) + (etat * cost));
    (rparl * rparl + rperp * rperp) / 2.0
}

fn fr_cond(cosi: Float, eta: Spectrum, k: &Spectrum) -> Spectrum {
    let tmp = (eta * eta + *k * *k) * cosi * cosi;
    let rparl2 = (tmp - (2.0 * eta * cosi) + Spectrum::white()) / (tmp + (2.0 * eta * cosi) + Spectrum::white());
    let tmp_f = eta * eta + *k * *k;
    let rperp2 =
        (tmp_f - (2.0 * eta * cosi) + Spectrum::from(cosi * cosi)) /
            (tmp_f + (2.0 * eta * cosi) + Spectrum::from(cosi * cosi));

    (rparl2 + rperp2) / 2.0
}

pub struct Microfacet {
    reflectance: Spectrum,
    fresnel: Box<Fresnel>,
    distribution: Box<MicrofacetDistribution>,
}

impl Microfacet {
    pub fn new(reflectance: Spectrum, fresnel: Box<Fresnel>, distribution: Box<MicrofacetDistribution>) -> Microfacet {
        Microfacet { reflectance, fresnel, distribution }
    }

    fn g(&self, wo: &Vector3f, wi: &Vector3f, wh: &Vector3f) -> Float {
        let n_dot_wh = abs_cos_theta(wh);
        let n_dot_wo = abs_cos_theta(wo);
        let n_dot_wi = abs_cos_theta(wi);
        let wo_dot_wh = wo.dot(*wh).abs();

        (2.0 * n_dot_wh * n_dot_wo / wo_dot_wh).min(2.0 * n_dot_wh * n_dot_wi / wo_dot_wh).min(1.0)
    }
}

impl BxDF for Microfacet {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        let cos_theta_o = abs_cos_theta(wo);
        let cos_theta_i = abs_cos_theta(wi);
        if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
            return Spectrum::black();
        }

        let mut wh: Vector3f = wi + wo;
        if wh.x == 0. && wh.y == 0. && wh.z == 0. {
            return Spectrum::black();
        }

        wh = wh.normalize();
        let cos_theta_h = wi.dot(wh);
        let f = self.fresnel.evaluate(cos_theta_h);
        return self.reflectance * self.distribution.d(&wh) * self.g(wo, wi, &wh) * f / (4.0 * cos_theta_i * cos_theta_o);
    }

    fn sample_f(&self, wo: Vector3f, wi: &mut Vector3f, u1: Float, u2: Float) -> (Spectrum, Float) {
        let (wi, pdf) = self.distribution.sample_f(&wo, u1, u2);
        if same_hemisphere(&wo, &wi) {
            (self.f(&wo, &wi), pdf)
        } else {
            (Spectrum::black(), pdf)
        }
    }

    fn bxdf_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_GLOSSY
    }

    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> Float {
        if same_hemisphere(wo, wi) {
            self.distribution.pdf(wo, wi)
        } else {
            0.0
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

#[inline]
fn cos_theta(w: &Vector3f) -> Float {
    w.z
}

#[inline]
fn sin_theta2(w: &Vector3f) -> Float {
    (1.0 - cos_theta(w) * cos_theta(w)).max(0.0)
}

