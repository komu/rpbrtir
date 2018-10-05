use core::{
    differential_geometry::DifferentialGeometry,
    primitive::Primitive,
};
use core::geometry::RayDifferential;
use core::reflection::BSDF;
use core::spectrum::Spectrum;
use core::geometry::Vector3f;
use core::types::Float;
use core::transform::Transform;

pub struct Intersection<'a> {
    pub primitive: &'a Primitive,
    pub dg: DifferentialGeometry<'a>,
    pub ray_epsilon: Float,
    object_to_world: Transform
}

impl<'a> Intersection<'a> {
    pub fn new(
        primitive: &'a Primitive,
        dg: DifferentialGeometry<'a>,
        ray_epsilon: Float,
        object_to_world: Transform)
        -> Intersection<'a> {
        Intersection { primitive, dg, ray_epsilon, object_to_world }
    }

    pub fn get_bsdf(&self, ray: &RayDifferential) -> BSDF {
        self.dg.compute_differentials(ray);
        self.primitive.get_bsdf(&self.dg, &self.object_to_world)
    }

    pub fn le(&self, w: Vector3f) -> Spectrum {
        match self.primitive.get_area_light() {
            Some(area) => area.l(&self.dg.p, &self.dg.nn, &w),
            None => Spectrum::black()
        }
    }
}
