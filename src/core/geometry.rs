use cgmath::{Point3, Vector3};
use core::types::{Float, INFINITY};
use cgmath::{vec3, prelude::*};

pub type Point3f = Point3<Float>;
pub type Vector3f = Vector3<Float>;

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

    pub fn new_simple(o: Point3f, d: Vector3f) -> Ray {
        Ray::new(o, d, 0.0, INFINITY, 0.0)
    }

    pub fn point_at(&self, t: Float) -> Point3f {
        self.o + t * self.d
    }
}

#[derive(Clone, Copy)]
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
