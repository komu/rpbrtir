use core::{
    differential_geometry::DifferentialGeometry,
    geometry::{Point3f, Vector3f},
    transform::Transform,
    types::Float,
};

pub trait Texture<T> {
    fn evaluate(&self, dg: &DifferentialGeometry) -> T;
}

pub trait TextureMapping2D {
    fn map(&self, dg: &DifferentialGeometry) -> TextureMapping2DResult;
}

pub struct TextureMapping2DResult {
    pub s: Float,
    pub t: Float,
    pub dsdx: Float,
    pub dtdx: Float,
    pub dsdy: Float,
    pub dtdy: Float,
}

pub struct UVMapping2D {
    su: Float,
    sv: Float,
    du: Float,
    dv: Float,
}

impl UVMapping2D {
    pub fn new(su: Float, sv: Float, du: Float, dv: Float) -> UVMapping2D {
        UVMapping2D { su, sv, du, dv }
    }
}

impl Default for UVMapping2D {
    fn default() -> Self {
        UVMapping2D::new(1.0, 1.0, 0.0, 0.0)
    }
}

impl TextureMapping2D for UVMapping2D {
    fn map(&self, dg: &DifferentialGeometry) -> TextureMapping2DResult {
        let diffs = dg.differentials.borrow();
        TextureMapping2DResult {
            s: self.su * dg.u + self.du,
            t: self.sv * dg.v + self.dv,
            dsdx: self.su * diffs.dudx,
            dtdx: self.sv * diffs.dvdx,
            dsdy: self.su * diffs.dudy,
            dtdy: self.sv * diffs.dvdy,
        }
    }
}

pub trait TextureMapping3D {
    fn map(&self, dg: &DifferentialGeometry) -> (Point3f, Vector3f, Vector3f);
}

pub struct IdentityMapping3D {
    world_to_texture: Transform
}

impl IdentityMapping3D {
    pub fn new(world_to_texture: Transform) -> IdentityMapping3D {
        IdentityMapping3D { world_to_texture }
    }
}

impl TextureMapping3D for IdentityMapping3D {
    fn map(&self, dg: &DifferentialGeometry) -> (Point3f, Vector3f, Vector3f) {
        let diffs = dg.differentials.borrow();
        (self.world_to_texture.transform_point(dg.p),
         self.world_to_texture.transform_vector(diffs.dpdx),
         self.world_to_texture.transform_vector(diffs.dpdy))
    }
}
