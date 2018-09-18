use core::{
    differential_geometry::DifferentialGeometry,
    geometry::Ray,
    types::Float,
    transform::Transform,
};

pub trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<(DifferentialGeometry, Float, Float)>;

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.intersect(&ray).is_some()
    }

    fn get_object_to_world(&self) -> Transform;
}
