use core::differential_geometry::DifferentialGeometry;

pub trait Texture<T> {
    fn evaluate(&self, dg: &DifferentialGeometry) -> T;
}
