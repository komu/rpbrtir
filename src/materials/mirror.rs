use core::{
    differential_geometry::DifferentialGeometry,
    material::{Material, bump},
    reflection::{FresnelNoOp, BSDF, SpecularReflection},
    spectrum::Spectrum,
    texture::Texture,
    types::Float,
};
use textures::ConstantTexture;
use std::sync::Arc;

pub struct MirrorMaterial {
    kr: Arc<Texture<Spectrum>>,
    bump_map: Option<Arc<Texture<Float>>>,
}

impl MirrorMaterial {
    pub fn new(kr: Arc<Texture<Spectrum>>,
               bump_map: Option<Arc<Texture<Float>>>) -> MirrorMaterial {
        MirrorMaterial { kr, bump_map }
    }
}

impl Material for MirrorMaterial {
    fn get_bsdf<'a>(&self, dg_geom: &DifferentialGeometry<'a>, dg_shading: &DifferentialGeometry<'a>) -> BSDF<'a> {
        let dgs = self.bump_map.as_ref().map_or_else(|| dg_shading.clone(), |b| bump(b.as_ref(), dg_geom, dg_shading));

        let r = self.kr.evaluate(&dgs).clamp_positive();

        let mut bsdf = BSDF::new(dgs, dg_geom.nn);
        if !r.is_black() {
            bsdf.add(Box::new(SpecularReflection::new(r, Box::new(FresnelNoOp::default()))));
        }

        bsdf
    }
}

impl Default for MirrorMaterial {
    fn default() -> Self {
        MirrorMaterial::new(Arc::new(ConstantTexture::new(Spectrum::from(0.9))), None)
    }
}
