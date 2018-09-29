use core::{
    geometry::{Point3f, Vector3f, Normal},
    shape::Shape,
    types::Float,
};
use cgmath::prelude::*;
use core::geometry::RayDifferential;
use std::cell::RefCell;
use core::transform::solve_linear_system_2x2;

#[derive(Clone, Debug)]
pub struct DifferentialGeometry<'a> {
    pub p: Point3f,
    pub nn: Normal,
    pub dpdu: Vector3f,
    pub dpdv: Vector3f,
    pub dndu: Normal,
    pub dndv: Normal,
    pub uu: Float,
    pub vv: Float,
    pub sh: &'a Shape,
    pub differentials: RefCell<Option<Differentials>>,
}

#[derive(Clone, Debug)]
pub struct Differentials {
    pub dpdx: Vector3f,
    pub dpdy: Vector3f,
    pub dudx: Float,
    pub dvdx: Float,
    pub dudy: Float,
    pub dvdy: Float,
}

impl<'a> DifferentialGeometry<'a> {
    pub fn new(p: Point3f,
               dpdu: Vector3f,
               dpdv: Vector3f,
               dndu: Normal,
               dndv: Normal,
               uu: Float,
               vv: Float,
               sh: &'a Shape) -> DifferentialGeometry<'a> {
        DifferentialGeometry {
            p,
            nn: Normal::from_vector(dpdu.cross(dpdv).normalize()),
            dpdu,
            dpdv,
            dndu,
            dndv,
            uu,
            vv,
            sh,
            differentials: RefCell::new(None),
        }
    }

    pub fn compute_differentials(&self, ray: &RayDifferential) {
        if let Some(ref diff) = ray.differentials {
            // Estimate screen space change in $\pt{}$ and $(u,v)$

            // Compute auxiliary intersection points with plane
            let d = -self.nn.v.dot(self.p.to_vec());
            let rxv = diff.rx_origin.to_vec();
            let tx = -(self.nn.v.dot(rxv) + d) / self.nn.v.dot(diff.rx_direction);
            if tx.is_nan() {
                self.differentials.replace(None);
                return;
            }
            let px = diff.rx_origin + tx * diff.rx_direction;
            let ryv = diff.ry_origin.to_vec();
            let ty = -(self.nn.v.dot(ryv) + d) / self.nn.v.dot(diff.ry_direction);
            if ty.is_nan() {
                self.differentials.replace(None);
                return;
            }
            let py = diff.ry_origin + ty * diff.ry_direction;
            let dpdx = px - self.p;
            let dpdy = py - self.p;

            // Compute $(u,v)$ offsets at auxiliary points

            let axes = if self.nn.v.x.abs() > self.nn.v.y.abs() && self.nn.v.x.abs() > self.nn.v.z.abs() {
                [1, 2]
            } else if self.nn.v.y.abs() > self.nn.v.z.abs() {
                [0, 2]
            } else {
                [0, 1]
            };

            // Initialize matrices for chosen projection plane
            // Initialize _A_, _Bx_, and _By_ matrices for offset computation
            let a = [[self.dpdu[axes[0]], self.dpdv[axes[0]]], [self.dpdu[axes[1]], self.dpdv[axes[1]]]];
            let bx = [px[axes[0]] - self.p[axes[0]], px[axes[1]] - self.p[axes[1]]];
            let by = [py[axes[0]] - self.p[axes[0]], py[axes[1]] - self.p[axes[1]]];

            let (dudx, dvdx) = solve_linear_system_2x2(&a, &bx).unwrap_or((0.0, 0.0));
            let (dudy, dvdy) = solve_linear_system_2x2(&a, &by).unwrap_or((0.0, 0.0));

            self.differentials.replace(Some(Differentials {
                dpdx,
                dpdy,
                dudx,
                dvdx,
                dudy,
                dvdy,
            }));
        } else {
            self.differentials.replace(None);
        }
    }
}
