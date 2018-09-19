use core::{
    differential_geometry::DifferentialGeometry,
    geometry::Ray,
    types::Float,
    transform::Transform,
};
use std::fmt::Debug;

pub trait Shape : Debug {
    fn intersect(&self, ray: &Ray) -> Option<(DifferentialGeometry, Float, Float)>;

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.intersect(&ray).is_some()
    }

    fn get_object_to_world(&self) -> &Transform;

    fn get_shading_geometry<'a, 'b>(&'a self, _obj_to_world: &'b Transform, dg: &'a DifferentialGeometry) -> &'a DifferentialGeometry {
        dg
    }
}
