use core::reflection::{BSDF, Lambertian};
use core::differential_geometry::DifferentialGeometry;
use core::spectrum::Spectrum;

pub trait Material {
    fn get_bsdf<'a>(&self, dg_geom: &'a DifferentialGeometry<'a> , dg_shading: &'a DifferentialGeometry<'a>) -> BSDF<'a>;
}

pub struct DummyMaterial {}

impl Material for DummyMaterial {
    fn get_bsdf<'a>(&self, dg_geom: &'a DifferentialGeometry<'a>, dg_shading: &'a DifferentialGeometry<'a>) -> BSDF<'a> {
        let mut bsdf = BSDF::new(dg_shading, dg_geom.nn);

        let r = Spectrum::white();

        bsdf.add(Box::new(Lambertian::new(r)));
        bsdf
    }
}

impl DummyMaterial {
    pub fn new() -> DummyMaterial {
        DummyMaterial {}
    }
}
