use core::{
    material::Material,
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
    fn get_bsdf<'a>(&self, dg_geom: &'a DifferentialGeometry<'a>, dg_shading: &'a DifferentialGeometry<'a>) -> BSDF<'a> {
        // Allocate _BSDF_, possibly doing bump mapping with _bumpMap_

        let dgs = match self.bump_map {
            Some(_) => unimplemented!("bump-maps are not supported"), // TODO: bump(self.bump_map, dg_geom, dg_shading);
            None => dg_shading
        };

        let mut bsdf = BSDF::new(dgs, dg_geom.nn);

        // Evaluate textures for _MatteMaterial_ material and allocate BRDF
        let r = self.kd.evaluate(dgs).clamp_positive();
        let sig = clamp(self.sigma.evaluate(dgs), 0.0, 90.0);
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
