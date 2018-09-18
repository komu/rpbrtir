use core::{
    geometry::{Point3f, Normal, Vector3f, Ray},
    types::Float,
};
use cgmath::{Matrix4, SquareMatrix, Transform as TransformCG};

#[derive(Clone)]
pub struct Transform {
    m: Matrix4<Float>,
    m_inv: Matrix4<Float>,
}

impl Transform {
    pub fn new(m: Matrix4<Float>) -> Transform {
        Transform { m, m_inv: m.invert().expect("Transformation matrix is not invertible") }
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
