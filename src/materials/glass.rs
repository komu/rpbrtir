use std::sync::Arc;
use core::{
    differential_geometry::DifferentialGeometry,
    material::Material,
    reflection::BSDF,
    spectrum::Spectrum,
    texture::Texture,
    types::Float,
};
use textures::ConstantTexture;
use core::reflection::FresnelDielectric;
use core::reflection::SpecularReflection;
use core::reflection::SpecularTransmission;

pub struct GlassMaterial {
    kr: Arc<Texture<Spectrum>>,
    kt: Arc<Texture<Spectrum>>,
    index: Arc<Texture<Float>>,
    bump_map: Option<Arc<Texture<Float>>>,
}

impl GlassMaterial {
    pub fn new(kr: Arc<Texture<Spectrum>>,
               kt: Arc<Texture<Spectrum>>,
               index: Arc<Texture<Float>>,
               bump_map: Option<Arc<Texture<Float>>>) -> GlassMaterial {
        GlassMaterial { kr, kt, index, bump_map }
    }
}

impl Material for GlassMaterial {
    fn get_bsdf<'a>(&self, dg_geom: &'a DifferentialGeometry<'a>, dg_shading: &'a DifferentialGeometry<'a>) -> BSDF<'a> {
        let dgs = if self.bump_map.is_some() {
            panic!("bump maps not supported") // TODO Bump(bumpMap, dgGeom, dgShading, &dgs);
        } else {
            dg_shading
        };

        let ior = self.index.evaluate(dgs);
        let mut bsdf = BSDF::new_with_eta(dgs, dg_geom.nn, ior);
        let r = self.kr.evaluate(dgs).clamp_positive();
        let t = self.kt.evaluate(dgs).clamp_positive();
        if !r.is_black() {
            bsdf.add(Box::new(SpecularReflection::new(r, Box::new(FresnelDielectric::new(1.0, ior)))));
        }
        if !t.is_black() {
            bsdf.add(Box::new(SpecularTransmission::new(t, 1.0, ior)));
        }

        return bsdf

    }
}

impl Default for GlassMaterial {
    fn default() -> Self {
        GlassMaterial {
            kr: Arc::new(ConstantTexture::new(Spectrum::white())),
            kt: Arc::new(ConstantTexture::new(Spectrum::white())),
            index: Arc::new(ConstantTexture::new(1.5)),
            bump_map: None
        }
    }
}
