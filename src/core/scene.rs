use core::{
    geometry::Ray,
    intersection::Intersection,
    light::Light,
    primitive::Primitive,
};

pub struct Scene {
    aggregate: Box<Primitive>,
    pub lights: Vec<Box<Light>>,
}

impl Scene {
    pub fn new(aggregate: Box<Primitive>, lights: Vec<Box<Light>>) -> Scene {
        Scene { aggregate, lights }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        self.aggregate.intersect(ray)
    }

    pub fn intersect_p(&self, ray: &Ray) -> bool {
        self.aggregate.intersect_p(ray)
    }
}
