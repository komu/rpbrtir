use cgmath::{Point3, Vector3};
use core::types::{Float, INFINITY};
use cgmath::{vec3, prelude::*};

pub type Point3f = Point3<Float>;
pub type Vector3f = Vector3<Float>;

#[derive(Clone, Debug)]
pub struct Ray {
    pub o: Point3f,
    pub d: Vector3f,
    pub mint: Float,
    pub maxt: Float,
    pub time: Float,
    pub depth: u32,
}

impl Ray {
    pub fn new(o: Point3f, d: Vector3f, mint: Float, maxt: Float, time: Float) -> Ray {
        Ray { o, d, mint, maxt, time, depth: 0 }
    }

    pub fn new_with_parent(o: Point3f, d: Vector3f, parent: &Ray, mint: Float) -> Ray {
        Ray { o, d, mint, maxt: INFINITY, time: parent.time, depth: parent.depth + 1 }
    }

    pub fn point_at(&self, t: Float) -> Point3f {
        self.o + t * self.d
    }
}

pub struct RayDifferential {
    pub ray: Ray,
    pub has_differentials: bool,
}

impl RayDifferential {
    pub fn new(o: Point3f, d: Vector3f, mint: Float, maxt: Float, time: Float) -> RayDifferential {
        RayDifferential {
            ray: Ray::new(o, d, mint, maxt, time),
            has_differentials: false,
        }
    }

    pub fn from_ray(ray: Ray) -> RayDifferential {
        RayDifferential { ray, has_differentials: false }
    }

    pub fn new_simple(o: Point3f, d: Vector3f) -> RayDifferential {
        RayDifferential::new(o, d, 0.0, INFINITY, 0.0)
    }

    pub fn new_with_parent(o: Point3f, d: Vector3f, parent: &Ray, mint: Float) -> RayDifferential {
        RayDifferential {
            ray: Ray::new_with_parent(o, d, parent, mint),
            has_differentials: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Normal {
    pub v: Vector3f
}

impl Normal {
    pub fn new(x: Float, y: Float, z: Float) -> Normal {
        Normal::from_vector(vec3(x, y, z))
    }

    pub fn from_vector(v: Vector3f) -> Normal {
        Normal { v }
    }
}

#[inline]
pub fn distance_squared(p1: &Point3f, p2: &Point3f) -> Float {
    (p1 - p2).magnitude2()
}
