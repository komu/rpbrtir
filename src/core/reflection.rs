use core::differential_geometry::DifferentialGeometry;
use core::spectrum::Spectrum;
use core::geometry::Vector3f;
use core::geometry::Normal;
use core::types::Float;
use cgmath::{vec3, prelude::*};
use core::types::INV_PI;

pub struct BSDF<'a> {
    pub dg_shading: &'a DifferentialGeometry<'a>,
    nn: Normal,
    ng: Normal,
    sn: Vector3f,
    tn: Vector3f,
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
        BSDF { dg_shading, ng, nn, sn, tn, bxdfs: Vec::new() }
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

        let mut f = Spectrum::black();
        for bxdf in &self.bxdfs {
            if bxdf.matches_flags(flags) {
                f += bxdf.f(&wo, &wi);
            }
        }
        f
    }

    pub fn add(&mut self, bxdf: Box<BxDF>) {
        self.bxdfs.push(bxdf)
    }

    fn world_to_local(&self, v: &Vector3f) -> Vector3f {
        vec3(v.dot(self.sn), v.dot(self.tn), v.dot(self.nn.v))
    }
}

pub trait BxDF {

    fn matches_flags(&self, flags: BxDFType) -> bool;

    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum;
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
    fn matches_flags(&self, flags: BxDFType) -> bool {
        true // TODO
    }

    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        self.r * INV_PI
    }
}
