extern crate cgmath;
extern crate image;
extern crate rand;
#[macro_use]
extern crate bitflags;

pub mod core;
pub mod films;
pub mod integrators;
pub mod lights;
pub mod renderers;
pub mod shapes;

use core::{
    scene::Scene,
    geometry::{Point3f},
    material::DummyMaterial,
    transform::Transform,
    types::Float,
    primitive::GeometricPrimitive,
    film::Film,
    spectrum::Spectrum,
    light::Light,
};
use shapes::sphere::Sphere;
use lights::point::PointLight;
use cgmath::{prelude::*};
use cgmath::Matrix4;
use renderers::samplerrenderer::SamplerRenderer;
use films::image::ImageFilm;
use core::primitive::CompoundPrimitive;
use core::primitive::Primitive;

fn main() {
    let nx = 600;
    let ny = 300;

    let scene = build_scene();

    let mut film = ImageFilm::new(String::from("images/output.png"), nx, ny);

    {
        let mut renderer = SamplerRenderer::new(&mut film);
        renderer.render(&scene)
    }

    film.write_image();
}

fn build_scene() -> Scene {
    let primitives: Vec<Box<Primitive>> = vec!(
        Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(0.0, 0.0, -1.0), 0.5)), Box::new(DummyMaterial::new()))),
        Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(1.0, -0.3, -1.0), 0.2)), Box::new(DummyMaterial::new())))
    );

    let lights: Vec<Box<Light>> = vec![
        Box::new(PointLight::new(Point3f::new(0.5, 0.5, 0.0), Spectrum::new(1.0, 0.0, 0.0))),
        Box::new(PointLight::new(Point3f::new(-0.5, -0.5, 0.0), Spectrum::new(0.0, 0.9, 0.3))),
    ];

    Scene::new(Box::new(CompoundPrimitive::new(primitives)), lights)
}

fn build_sphere(center: Point3f, radius: Float) -> Sphere {
    let object_to_world = Transform::new(Matrix4::from_translation(center.to_vec()));
    let world_to_object = object_to_world.invert();

    Sphere::new(object_to_world, world_to_object, radius)
}
