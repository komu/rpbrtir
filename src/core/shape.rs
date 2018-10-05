use core::{
    differential_geometry::DifferentialGeometry,
    geometry::Ray,
    types::Float,
    transform::Transform,
};
use std::fmt::Debug;
use std::sync::Arc;
use core::geometry::Vector3f;
use core::geometry::Point3f;
use core::geometry::Normal;

pub trait Shape : Debug {
    fn intersect(&self, ray: &Ray) -> Option<(DifferentialGeometry, Float, Float)>;

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.intersect(&ray).is_some()
    }

    fn pdf(&self, p: &Point3f, wi: &Vector3f) -> Float;

    fn get_object_to_world(&self) -> &Transform;

    fn get_shading_geometry<'a, 'b>(&'a self, _obj_to_world: &'b Transform, dg: &'a DifferentialGeometry) -> &'a DifferentialGeometry {
        dg
    }

    fn area(&self) -> Float;

    fn can_intersect(&self) -> bool {
        true
    }

    fn sample_point(&self, _p: &Point3f, u1: Float, u2: Float) -> (Point3f, Normal) {
        self.sample(u1, u2)
    }

    fn sample(&self, u1: Float, u2: Float) -> (Point3f, Normal) {
        unimplemented!()
    }

    fn refine(&self, shapes: &mut Vec<Arc<Shape>>) {
        unimplemented!("refine called for non-refinable shape")
    }
}
