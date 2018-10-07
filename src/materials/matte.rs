use core::{
    material::{Material, bump},
    differential_geometry::DifferentialGeometry,
    reflection::{BSDF, Lambertian},
    texture::Texture,
    spectrum::Spectrum,
    types::Float,
    math::clamp,
};
use std::sync::Arc;

pub struct MatteMaterial {
    kd: Arc<Texture<Spectrum>>,
    sigma: Arc<Texture<Float>>,
    bump_map: Option<Arc<Texture<Float>>>,
}

impl MatteMaterial {
    pub fn new(kd: Arc<Texture<Spectrum>>,
               sigma: Arc<Texture<Float>>,
               bump_map: Option<Arc<Texture<Float>>>) -> MatteMaterial {
        MatteMaterial { kd, sigma, bump_map }
    }
}

impl Material for MatteMaterial {
    fn get_bsdf<'a>(&self, dg_geom: &DifferentialGeometry<'a>, dg_shading: &DifferentialGeometry<'a>) -> BSDF<'a> {
        let dgs = self.bump_map.as_ref().map_or_else(|| dg_shading.clone(), |b| bump(b.as_ref(), dg_geom, dg_shading));

        // Evaluate textures for _MatteMaterial_ material and allocate BRDF
        let r = self.kd.evaluate(&dgs).clamp_positive();
        let sig = clamp(self.sigma.evaluate(&dgs), 0.0, 90.0);
        let mut bsdf = BSDF::new(dgs, dg_geom.nn);
        if !r.is_black() {
            if sig == 0.0 {
                bsdf.add(Box::new(Lambertian::new(r)));
            } else {
                unimplemented!("OrenNayar is not supported") // TODO
                //bsdf.add(OrenNayar::new(r, sig));
            }
        }

        bsdf
    }
}
