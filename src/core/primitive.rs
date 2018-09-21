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
    fn get_bsdf<'a, 'b>(&'a self, dg: &'a DifferentialGeometry<'a>, object_to_world: &'b Transform) -> BSDF<'a>;
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
            let o2w = self.shape.get_object_to_world().clone(); // TODO avoid copy
            Some(Intersection::new(self, dg, ray_epsilon, o2w))
        } else {
            None
        }
    }

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.shape.intersect_p(ray)
    }

    fn get_bsdf<'a, 'b>(&'a self, dg: &'a DifferentialGeometry<'a>, object_to_world: &'b Transform) -> BSDF<'a> {
        let dgs = self.shape.get_shading_geometry(object_to_world, dg);
        self.material.get_bsdf(dg, dgs)
    }
}

// TODO: simple temporary compound before accelerators are implemented
pub struct CompoundPrimitive {
    primitives: Vec<Box<Primitive>>
}

impl CompoundPrimitive {
    pub fn new(primitives: Vec<Box<Primitive>>) -> CompoundPrimitive {
        CompoundPrimitive { primitives }
    }
}

impl Primitive for CompoundPrimitive {

    fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut best: Option<Intersection> = None;

        for p in &self.primitives {
            if let Some(isect) = p.intersect(ray) {
                best = Some(isect);
            }
        }

        best
    }

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.primitives.iter().any(|p| { p.intersect_p(ray) })
    }

    fn get_bsdf<'a, 'b>(&'a self, dg: &'a DifferentialGeometry<'a>, object_to_world: &'b Transform) -> BSDF<'a> {
        panic!("get_bsdf should not be called for Aggregate")
    }
}
