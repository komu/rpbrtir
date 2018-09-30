use core::{
    texture::{Texture, TextureMapping2D, TextureMapping3D},
    differential_geometry::DifferentialGeometry,
    math::floor_to_int,
    types::Float,
};
use std::sync::Arc;

pub struct Checkerboard2DTexture<T> {
    mapping: Box<TextureMapping2D>,
    tex1: Arc<Texture<T>>,
    tex2: Arc<Texture<T>>,
    aa_method: AAMethod,
}

pub struct Checkerboard3DTexture<T> {
    mapping: Box<TextureMapping3D>,
    tex1: Arc<Texture<T>>,
    tex2: Arc<Texture<T>>,
}

#[derive(PartialEq, Eq)]
pub enum AAMethod {
    None,
    ClosedForm,
}

impl<T> Checkerboard2DTexture<T> {
    pub fn new(mapping: Box<TextureMapping2D>,
               tex1: Arc<Texture<T>>,
               tex2: Arc<Texture<T>>,
               aa_method: AAMethod) -> Checkerboard2DTexture<T> {
        Checkerboard2DTexture { mapping, tex1, tex2, aa_method }
    }
}

impl<T> Texture<T> for Checkerboard2DTexture<T> {
    fn evaluate(&self, dg: &DifferentialGeometry) -> T {
        let r = self.mapping.map(dg);

        if self.aa_method == AAMethod::None {
            if (floor_to_int(r.s) + floor_to_int(r.t)) % 2 == 0 {
                self.tex1.evaluate(dg)
            } else {
                self.tex2.evaluate(dg)
            }
        } else {
            // Compute closed-form box-filtered _Checkerboard2DTexture_ value
            // Evaluate single check if filter is entirely inside one of them
            let ds = r.dsdx.abs().max(r.dsdy.abs());
            let dt = r.dtdx.abs().max(r.dtdy.abs());
            let s0 = r.s - ds;
            let s1 = r.s + ds;
            let t0 = r.t - dt;
            let t1 = r.t + dt;
            if floor_to_int(s0) == floor_to_int(s1) && floor_to_int(t0) == floor_to_int(t1) {
                // Point sample _Checkerboard2DTexture_
                return if floor_to_int(r.s) + floor_to_int(r.t) % 2 == 0 {
                    self.tex1.evaluate(dg)
                } else {
                    self.tex2.evaluate(dg)
                };
            }

            // Apply box filter to checkerboard region
            let sint = (bumpint(s1) - bumpint(s0)) / (2.0 * ds);
            let tint = (bumpint(t1) - bumpint(t0)) / (2.0 * dt);
            let mut area2 = sint + tint - 2.0 * sint * tint;
            if ds > 1.0 || dt > 1.0 {
                area2 = 0.5;
            }

            unimplemented!() // TODO
//            return self.tex1.evaluate(dg) * (1.0 - area2) + self.tex2.evaluate(dg) * area2;
        }
    }
}

fn bumpint(x: Float) -> Float {
    (x / 2.0).floor() + 2.0 * ((x / 2.0) - (x / 2.0).floor() - 0.5).max(0.0)
}

impl<T> Checkerboard3DTexture<T> {
    pub fn new(mapping: Box<TextureMapping3D>,
               tex1: Arc<Texture<T>>,
               tex2: Arc<Texture<T>>) -> Checkerboard3DTexture<T> {
        Checkerboard3DTexture { mapping, tex1, tex2 }
    }
}

impl<T> Texture<T> for Checkerboard3DTexture<T> {
    fn evaluate(&self, dg: &DifferentialGeometry) -> T {
        let (p, _, _) = self.mapping.map(dg);
        return if (floor_to_int(p.x) + floor_to_int(p.y) + floor_to_int(p.z)) % 2 == 0 {
            self.tex1.evaluate(dg)
        } else {
            self.tex2.evaluate(dg)
        };
    }
}