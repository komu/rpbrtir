use core::{
    differential_geometry::DifferentialGeometry,
    geometry::Normal,
    reflection::BSDF,
    texture::Texture,
    types::Float
};
use core::geometry::faceforward;
use cgmath::prelude::*;

pub trait Material {
    fn get_bsdf<'a>(&self, dg_geom: &DifferentialGeometry<'a>, dg_shading: &DifferentialGeometry<'a>) -> BSDF<'a>;
}

pub fn bump<'a>(d: &Texture<Float>, dg_geom: &DifferentialGeometry, dgs: &DifferentialGeometry<'a>) -> DifferentialGeometry<'a> {
    // Compute offset positions and evaluate displacement texture
    let mut dg_eval = dgs.clone();

    // Shift _dgEval_ _du_ in the $u$ direction
    let dgs_diffs = dgs.differentials.borrow();
    let mut du = 0.5 * (dgs_diffs.dudx.abs() + dgs_diffs.dudy.abs());
    if du == 0.0 {
        du = 0.01;
    }

    dg_eval.p = dgs.p + du * dgs.dpdu;
    dg_eval.u = dgs.u + du;
    dg_eval.nn = Normal::from(dgs.dpdu.cross(dgs.dpdv) + du * dgs.dndu.v).normalize();
    let u_displace = d.evaluate(&dg_eval);

    // Shift _dgEval_ _dv_ in the $v$ direction
    let mut dv = 0.5 * (dgs_diffs.dvdx.abs() + dgs_diffs.dvdy.abs());
    if dv == 0.0 {
        dv = 0.01;
    }

    dg_eval.p = dgs.p + dv * dgs.dpdv;
    dg_eval.u = dgs.u;
    dg_eval.v = dgs.v + dv;
    dg_eval.nn = Normal::from(dgs.dpdu.cross(dgs.dpdv) + dv * dgs.dndv.v).normalize();
    let v_displace = d.evaluate(&dg_eval);
    let displace = d.evaluate(dgs);

    // Compute bump-mapped differential geometry
    let mut dg_bump = dgs.clone();
    dg_bump.dpdu = dgs.dpdu + (u_displace - displace) / du * dgs.nn.v + displace * dgs.dndu.v;
    dg_bump.dpdv = dgs.dpdv + (v_displace - displace) / dv * dgs.nn.v + displace * dgs.dndv.v;
    dg_bump.nn = Normal::from(dg_bump.dpdu.cross(dg_bump.dpdv).normalize());

    if dgs.shape.reverse_orientation() ^ dgs.shape.transform_swaps_handedness() {
        dg_bump.nn *= -1.0;
    }

    // Orient shading normal to match geometric normal
    dg_bump.nn = faceforward(dg_bump.nn, &dg_geom.nn);

    dg_bump
}
