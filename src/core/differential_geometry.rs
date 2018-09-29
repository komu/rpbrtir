use core::{
    geometry::{Point3f, Vector3f, Normal},
    shape::Shape,
    types::Float,
};
use cgmath::prelude::*;
use core::geometry::RayDifferential;
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub struct DifferentialGeometry<'a> {
    pub p: Point3f,
    pub nn: Normal,
    pub dpdu: Vector3f,
    pub dpdv: Vector3f,
    pub dndu: Normal,
    pub dndv: Normal,
    pub uu: Float,
    pub vv: Float,
    pub sh: &'a Shape,
    differentials: RefCell<Option<Differentials>>
}

#[derive(Clone, Debug)]
struct Differentials {
    dpdx: Vector3f,
    dpdy: Vector3f,
    dudx: Float,
    dvdx: Float,
    dudy: Float,
    dvdy: Float,
}

impl<'a> DifferentialGeometry<'a> {
    pub fn new(p: Point3f,
               dpdu: Vector3f,
               dpdv: Vector3f,
               dndu: Normal,
               dndv: Normal,
               uu: Float,
               vv: Float,
               sh: &'a Shape) -> DifferentialGeometry<'a> {
        DifferentialGeometry {
            p,
            nn: Normal::from_vector(dpdu.cross(dpdv).normalize()),
            dpdu,
            dpdv,
            dndu,
            dndv,
            uu,
            vv,
            sh,
            differentials: RefCell::new(None)
        }
    }

    pub fn compute_differentials(&self, ray: &RayDifferential) {
        if ray.differentials.is_some() {
            // TODO
            unimplemented!()
        } else {
            self.differentials.replace(None);
        }
    }
}
