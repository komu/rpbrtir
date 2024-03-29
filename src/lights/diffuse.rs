use core::{
    geometry::{Normal, Vector3f, Point3f},
    light::{Light, AreaLight, ShapeSet, LightSample, VisibilityTester},
    scene::Scene,
    shape::Shape,
    spectrum::Spectrum,
    transform::Transform,
    types::{Float,PI},
};
use std::sync::Arc;
use cgmath::prelude::*;

pub struct DiffuseAreaLight {
    light_to_world: Transform,
    l_emit: Spectrum,
    shape_set: ShapeSet,
    area: Float,
    num_samples: u32
}

impl DiffuseAreaLight {
    pub fn new(light_to_world: Transform, l_emit: Spectrum, num_samples: u32, shape: Arc<Shape>) -> DiffuseAreaLight {
        let shape_set = ShapeSet::new(shape);
        let area = shape_set.area();
        DiffuseAreaLight { light_to_world, l_emit, shape_set, area, num_samples }
    }
}

impl Light for DiffuseAreaLight {
    fn sample_l(&self, p: &Point3f, p_epsilon: Float, ls: &LightSample, time: Float, visibility: &mut VisibilityTester) -> (Spectrum, Vector3f, Float) {
        let (ps, ns) = self.shape_set.sample_point(&p, &ls);

        let wi = (ps - p).normalize();
        let pdf = self.shape_set.pdf(p, &wi);

        visibility.set_segment(*p, p_epsilon, ps, 1e-3, time);

        let ls = self.l(&ps, &ns, &-wi);

        (ls, wi, pdf)
    }

    fn pdf(&self, p: &Point3f, wi: &Vector3f) -> Float {
        self.shape_set.pdf(p, wi)
    }

    fn power(&self, scene: &Scene) -> Spectrum {
        self.l_emit * self.area * PI
    }

    fn num_samples(&self) -> u32 {
        self.num_samples
    }

    fn is_delta_light(&self) -> bool {
        false
    }
}

impl AreaLight for DiffuseAreaLight {
    fn l(&self, p: &Point3f, n: &Normal, w: &Vector3f) -> Spectrum {
        if n.v.dot(*w) > 0.0 {
            self.l_emit
        } else {
            Spectrum::black()
        }
    }
}
