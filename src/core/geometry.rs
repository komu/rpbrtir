use cgmath::{Point3, Vector3};
use core::types::{Float, INFINITY};
use cgmath::{vec3, prelude::*};
use core::math::lerp;
use std::ops::MulAssign;

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

    pub fn has_nans(&self) -> bool {
        self.o.x.is_nan() || self.o.y.is_nan() || self.o.z.is_nan()
    }
}

pub struct RayDifferential {
    pub ray: Ray,
    pub differentials: Option<RayDifferentials>,
}

pub struct RayDifferentials {
    pub rx_origin: Point3f,
    pub ry_origin: Point3f,
    pub rx_direction: Vector3f,
    pub ry_direction: Vector3f,
}

impl RayDifferential {
    pub fn new(o: Point3f, d: Vector3f, mint: Float, maxt: Float, time: Float) -> RayDifferential {
        RayDifferential::from_ray(Ray::new(o, d, mint, maxt, time))
    }

    pub fn from_ray(ray: Ray) -> RayDifferential {
        RayDifferential { ray, differentials: None }
    }

    pub fn new_simple(o: Point3f, d: Vector3f) -> RayDifferential {
        RayDifferential::new(o, d, 0.0, INFINITY, 0.0)
    }

    pub fn new_with_parent(o: Point3f, d: Vector3f, parent: &Ray, mint: Float) -> RayDifferential {
        RayDifferential { ray: Ray::new_with_parent(o, d, parent, mint), differentials: None }
    }

    pub fn has_nans(&self) -> bool {
        self.ray.has_nans()
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

    pub fn normalize(&self) -> Normal {
        Normal { v: self.v.normalize() }
    }
}

impl MulAssign<Float> for Normal {
    fn mul_assign(&mut self, rhs: Float) {
        self.v.x *= rhs;
        self.v.y *= rhs;
        self.v.z *= rhs;
    }
}

#[inline]
pub fn distance(p1: &Point3f, p2: &Point3f) -> Float {
    (p1 - p2).magnitude()
}

#[inline]
pub fn distance_squared(p1: &Point3f, p2: &Point3f) -> Float {
    (p1 - p2).magnitude2()
}

#[derive(PartialEq, Debug, Clone)]
pub struct BBox {
    p_min: Point3f,
    p_max: Point3f,
}

impl BBox {
    pub fn new(p1: &Point3f, p2: Point3f) -> BBox {
        BBox {
            p_min: Point3f::new(p1.x.min(p2.x), p1.y.min(p2.y), p1.z.min(p2.z)),
            p_max: Point3f::new(p1.x.max(p2.x), p1.y.max(p2.y), p1.z.max(p2.z)),
        }
    }

    pub fn from_point(p: Point3f) -> BBox {
        BBox { p_min: p, p_max: p }
    }

    pub fn union_point(&self, p: &Point3f) -> BBox {
        BBox {
            p_min: Point3f::new(self.p_min.x.min(p.x), self.p_min.y.min(p.y), self.p_min.z.min(p.z)),
            p_max: Point3f::new(self.p_max.x.max(p.x), self.p_max.y.max(p.y), self.p_max.z.max(p.z)),
        }
    }

    pub fn union(&self, b: &BBox) -> BBox {
        BBox {
            p_min: Point3f::new(self.p_min.x.min(b.p_min.x), self.p_min.y.min(b.p_min.y), self.p_min.z.min(b.p_min.z)),
            p_max: Point3f::new(self.p_max.x.max(b.p_max.x), self.p_max.y.max(b.p_max.y), self.p_max.z.max(b.p_max.z)),
        }
    }

    pub fn overlaps(&self, b: &BBox) -> bool {
        let x = (self.p_max.x >= b.p_min.x) && (self.p_min.x <= b.p_max.x);
        let y = (self.p_max.y >= b.p_min.y) && (self.p_min.y <= b.p_max.y);
        let z = (self.p_max.z >= b.p_min.z) && (self.p_min.z <= b.p_max.z);
        x && y && z
    }

    pub fn inside(&self, pt: &Point3f) -> bool {
        pt.x >= self.p_min.x && pt.x <= self.p_max.x
            && pt.y >= self.p_min.y && pt.y <= self.p_max.y
            && pt.z >= self.p_min.z && pt.z <= self.p_max.z
    }

    pub fn expand(&mut self, delta: Float) {
        let v = vec3(delta, delta, delta);
        self.p_min -= v;
        self.p_max += v;
    }

    pub fn surface_area(&self) -> Float {
        let d = self.p_max - self.p_min;
        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    pub fn volume(&self) -> Float {
        let d = self.p_max - self.p_min;
        d.x * d.y * d.z
    }

    pub fn maximum_extent(&self) -> u8 {
        let diag = self.p_max - self.p_min;
        if diag.x > diag.y && diag.x > diag.z {
            0
        } else if diag.y > diag.z {
            1
        } else {
            2
        }
    }

    pub fn lerp(&self, tx: Float, ty: Float, tz: Float) -> Point3f {
        Point3f::new(lerp(tx, self.p_min.x, self.p_max.x),
                     lerp(ty, self.p_min.y, self.p_max.y),
                     lerp(tz, self.p_min.z, self.p_max.z))
    }

    pub fn offset(&self, p: &Point3f) -> Vector3f {
        vec3((p.x - self.p_min.x) / (self.p_max.x - self.p_min.x),
             (p.y - self.p_min.y) / (self.p_max.y - self.p_min.y),
             (p.z - self.p_min.z) / (self.p_max.z - self.p_min.z))
    }

    pub fn bounding_sphere(&self) -> (Point3f, Float) {
        let c = Point3f::from_vec(0.5 * self.p_min.to_vec() + 0.5 * self.p_max.to_vec());
        let rad = if self.inside(&c) { distance(&c, &self.p_max) } else { 0.0 };
        (c, rad)
    }

    pub fn intersect_p(&self, ray: &Ray) -> Option<(Float, Float)> {
        let inv_dir = Vector3::new(1.0, 1.0, 1.0).div_element_wise(ray.d);
        let mut min_t = ray.mint;
        let mut max_t = ray.maxt;

        for i in 0..3 {
            let a = (self.p_min[i] - ray.o[i]) * inv_dir[i];
            let b = (self.p_max[i] - ray.o[i]) * inv_dir[i];

            min_t = min_t.max(a.min(b));
            max_t = max_t.min(a.max(b));

            if min_t > max_t { return None; }
        }

        return Some((min_t, max_t));
    }
}

pub fn spherical_direction(sintheta: Float, costheta: Float, phi: Float) -> Vector3f {
    vec3(sintheta * phi.cos(), sintheta * phi.sin(), costheta)
}
