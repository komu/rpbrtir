use core::{
    geometry::{Point3f, Normal, Vector3f, Ray},
    types::Float,
};
use cgmath::{Matrix4, SquareMatrix, Transform as TransformCG, prelude::*};
use core::math::radians;
use std::ops::Mul;

#[derive(Clone, Debug)]
pub struct Transform {
    m: Matrix4<Float>,
    m_inv: Matrix4<Float>,
}

impl Transform {
    pub fn new(m: Matrix4<Float>) -> Transform {
        Transform { m, m_inv: m.invert().expect("Transformation matrix is not invertible") }
    }

    pub fn identity() -> Transform {
        Transform { m: Matrix4::identity(), m_inv: Matrix4::identity() }
    }

    #[inline]
    pub fn transform_point(&self, p: Point3f) -> Point3f {
        self.m.transform_point(p)
    }

    #[inline]
    pub fn transform_vector(&self, v: Vector3f) -> Vector3f {
        self.m.transform_vector(v)
    }

    #[inline]
    pub fn transform_normal(&self, n: Normal) -> Normal {
        let x = n.v.x;
        let y = n.v.y;
        let z = n.v.z;
        let m_inv = &self.m_inv;

        let nx = m_inv[0][0] * x + m_inv[1][0] * y + m_inv[2][0] * z;
        let ny = m_inv[0][1] * x + m_inv[1][1] * y + m_inv[2][1] * z;
        let nz = m_inv[0][2] * x + m_inv[1][2] * y + m_inv[2][2] * z;

        Normal::new(nx, ny, nz)
    }

    #[inline]
    pub fn transform_ray(&self, ray: &Ray) -> Ray {
        Ray::new(self.transform_point(ray.o), self.transform_vector(ray.d), ray.mint, ray.maxt, ray.time)
    }

    pub fn invert(&self) -> Transform {
        Transform { m: self.m_inv, m_inv: self.m }
    }
}

impl<'a, 'b> Mul<&'b Transform> for &'a Transform {
    type Output = Transform;

    fn mul(self, rhs: &'b Transform) -> Transform {
        Transform {
            m: self.m * rhs.m,
            m_inv: rhs.m_inv * self.m_inv,
        }
    }
}

impl<'a> Mul<&'a Transform> for Transform {
    type Output = Transform;

    fn mul(self, rhs: &'a Transform) -> Transform {
        Transform {
            m: self.m * rhs.m,
            m_inv: rhs.m_inv * self.m_inv,
        }
    }
}

pub fn scale(x: Float, y: Float, z: Float) -> Transform {
    Transform {
        m: Matrix4::from_nonuniform_scale(x, y, z),
        m_inv: Matrix4::from_nonuniform_scale(1.0 / x, 1.0 / y, 1.0 / z),
    }
}

pub fn translate(delta: &Vector3f) -> Transform {
    Transform {
        m: Matrix4::from_translation(*delta),
        m_inv: Matrix4::from_translation(-*delta),
    }
}

pub fn perspective(fov: Float, n: Float, f: Float) -> Transform {
    // Perform projective divide
    let persp = Matrix4::new(1.0, 0.0, 0.0, 0.0,
                             0.0, 1.0, 0.0, 0.0,
                             0.0, 0.0, f / (f - n), 1.0,
                             0.0, 0.0, -f * n / (f - n), 0.0);

    // Scale to canonical viewing volume
    let inv_tan_ang = 1.0 / (radians(fov) / 2.0).tan();
    return scale(inv_tan_ang, inv_tan_ang, 1.0) * &Transform::new(persp);
}

pub fn look_at(pos: &Point3f, look: &Point3f, up: &Vector3f) -> Transform {
    let dir = (look - pos).normalize();
    println!("dir: {:?}", dir); // TODO
    if up.normalize().cross(dir).magnitude() == 0.0 {
        println!("'up' and 'look' vectors pointing at same direction");
        return Transform::identity();
    }

//    let cam_to_world = Matrix4::look_at(*pos, *look, *up);
    let left = up.normalize().cross(dir).normalize();
    let new_up = dir.cross(left);

    let cam_to_world = Matrix4::new(
        left.x, left.y, left.z, 0.0,
        new_up.x, new_up.y, new_up.z, 0.0,
        dir.x, dir.y, dir.z, 0.0,
        pos.x, pos.y, pos.z, 1.0
    );

    Transform {
        m: cam_to_world,
        m_inv: cam_to_world.invert().unwrap(),
    }
}

// Matrix4x4 Method Definitions
pub fn solve_linear_system_2x2(a: &[[Float; 2]; 2], b: &[Float; 2]) -> Option<(Float, Float)> {
    let det = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    if det.abs() < 1e-10 {
        return None;
    }
    let x0 = (a[1][1] * b[0] - a[0][1] * b[1]) / det;
    let x1 = (a[0][0] * b[1] - a[1][0] * b[0]) / det;
    if x0.is_nan() || x1.is_nan() {
        None
    } else {
        Some((x0, x1))
    }
}
