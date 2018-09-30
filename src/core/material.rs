use core::{
    differential_geometry::DifferentialGeometry,
    reflection::BSDF,
};

pub trait Material {
    fn get_bsdf<'a>(&self, dg_geom: &'a DifferentialGeometry<'a>, dg_shading: &'a DifferentialGeometry<'a>) -> BSDF<'a>;
}
