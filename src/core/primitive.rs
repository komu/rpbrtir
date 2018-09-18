use core::{
    geometry::Ray,
    intersection::Intersection,
    material::Material,
    shape::Shape,
    differential_geometry::DifferentialGeometry,
    reflection::BSDF,
};
use core::transform::Transform;

pub trait Primitive {
    fn intersect(&self, ray: &mut Ray) -> Option<Intersection>;
    fn intersect_p(&self, ray: &Ray) -> bool;
    fn get_bsdf<'a>(&self, dg: &'a DifferentialGeometry<'a>, object_to_world: &Transform) -> BSDF<'a>;
}

pub struct GeometricPrimitive {
    shape: Box<Shape>,
    material: Box<Material>,
}

impl GeometricPrimitive {
    pub fn new(shape: Box<Shape>, material: Box<Material>) -> GeometricPrimitive {
        GeometricPrimitive {
            shape,
            material,
        }
    }
}

impl Primitive for GeometricPrimitive {
    fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        if let Some((dg, thit, ray_epsilon)) = self.shape.intersect(ray) {
            ray.maxt = thit;
//            isect->primitive = this;
//            isect->WorldToObject = *shape->WorldToObject;
//            isect->shapeId = shape->shapeId;
//            isect->primitiveId = primitiveId;
            let o2w = self.shape.get_object_to_world(); // TODO avoid copy
            Some(Intersection::new(self, dg, ray_epsilon, o2w))
        } else {
            None
        }
    }

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.shape.intersect_p(ray)
    }

    fn get_bsdf<'a>(&self, dg: &'a DifferentialGeometry<'a>, object_to_world: &Transform) -> BSDF<'a> {
        let dgs = dg; // TODO self.shape.get_shading_geometry(object_to_world, dg);
        self.material.get_bsdf(dg, dgs)
    }
}
