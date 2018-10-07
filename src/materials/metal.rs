use std::sync::Arc;
use core::{
    differential_geometry::DifferentialGeometry,
    reflection::{Blinn, BSDF},
    spectrum::Spectrum,
    texture::Texture,
    types::Float,
    material::{Material, bump},
};
use core::reflection::{FresnelConductor, Microfacet};

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
    fn get_bsdf<'a>(&self, dg_geom: &DifferentialGeometry<'a>, dg_shading: &DifferentialGeometry<'a>) -> BSDF<'a> {
        let dgs = self.bump_map.as_ref().map_or_else(|| dg_shading.clone(), |b| bump(b.as_ref(), dg_geom, dg_shading));

        let rough = self.roughness.evaluate(&dgs);

        let md = Blinn::new(1.0 / rough);
        let fr_mf = FresnelConductor::new(self.eta.evaluate(&dgs), self.k.evaluate(&dgs));

        let mut bsdf = BSDF::new(dgs, dg_geom.nn);
        bsdf.add(Box::new(Microfacet::new(Spectrum::white(), Box::new(fr_mf), Box::new(md))));
        bsdf
    }
}
