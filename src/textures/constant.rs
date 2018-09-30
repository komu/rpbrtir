use core::texture::Texture;
use core::differential_geometry::DifferentialGeometry;

pub struct ConstantTexture<T> {
    value: T
}

impl <T> ConstantTexture<T> {
    pub fn new(value: T) -> ConstantTexture<T> {
        ConstantTexture { value }
    }
}

impl <T : Copy> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, dg: &DifferentialGeometry) -> T {
        self.value
    }
}
