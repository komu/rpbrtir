use std::sync::Arc;
use core::{
    differential_geometry::DifferentialGeometry,
    reflection::{Blinn, BSDF},
    spectrum::Spectrum,
    texture::Texture,
    types::Float,
    material::Material,
};
use core::reflection::FresnelConductor;
use core::reflection::Microfacet;

pub struct MetalMaterial {
    eta: Arc<Texture<Spectrum>>,
    k: Arc<Texture<Spectrum>>,
    roughness: Arc<Texture<Float>>,
    bump_map: Option<Arc<Texture<Float>>>,
}

impl MetalMaterial {
    pub fn new(eta: Arc<Texture<Spectrum>>,
               k: Arc<Texture<Spectrum>>,
               roughness: Arc<Texture<Float>>,
               bump_map: Option<Arc<Texture<Float>>>) -> MetalMaterial {
        MetalMaterial { eta, k, roughness, bump_map }
    }
}

impl Material for MetalMaterial {
    fn get_bsdf<'a>(&self, dg_geom: &'a DifferentialGeometry<'a>, dg_shading: &'a DifferentialGeometry<'a>) -> BSDF<'a> {
        let dgs = if self.bump_map.is_some() {
            panic!("bump maps not supported") // TODO Bump(bumpMap, dgGeom, dgShading, &dgs);
        } else {
            dg_shading
        };

        let mut bsdf = BSDF::new(dgs, dg_geom.nn);

        let rough = self.roughness.evaluate(dgs);

        let md = Blinn::new(1.0 / rough);
        let fr_mf = FresnelConductor::new(self.eta.evaluate(dgs), self.k.evaluate(dgs));

        bsdf.add(Box::new(Microfacet::new(Spectrum::white(), Box::new(fr_mf), Box::new(md))));

        bsdf
    }
}
