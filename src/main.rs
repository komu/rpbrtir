extern crate cgmath;
extern crate image;
extern crate rand;
#[macro_use]
extern crate bitflags;

pub mod core;
pub mod integrators;
pub mod lights;
pub mod renderers;
pub mod shapes;

use core::{
    scene::Scene,
    geometry::{RayDifferential, Point3f},
    material::DummyMaterial,
    transform::Transform,
    types::Float,
    primitive::GeometricPrimitive,
    rng::RNG,
    spectrum::Spectrum,
    light::Light,
    renderer::Renderer,
};
use shapes::sphere::Sphere;
use lights::point::PointLight;
use cgmath::{vec3, prelude::*};
use cgmath::Matrix4;
use image::ImageBuffer;
use renderers::samplerrenderer::SamplerRenderer;

fn main() {
    let nx = 600;
    let ny = 300;

    let lower_left_corner = Point3f::new(-2.0, -1.0, -1.0);
    let horizontal = vec3(4.0, 0.0, 0.0);
    let vertical = vec3(0.0, 2.0, 0.0);
    let origin = Point3f::new(0.0, 0.0, 0.0);
    let scene = build_scene();
    let mut rng = RNG::new();
    let renderer = SamplerRenderer::new();

    let img = ImageBuffer::from_fn(nx, ny, |i, j| {
        let j = ny - j;

        let u = (i as Float) / (nx as Float);
        let v = (j as Float) / (ny as Float);

        let mut r = RayDifferential::new_simple(origin, lower_left_corner.to_vec() + u * horizontal + v * vertical);
        renderer.li(&scene, &mut r, None, &mut rng).to_rgb()
    });

    img.save("images/output.png").unwrap();
}

fn build_scene() -> Scene {
    let sphere = build_sphere(Point3f::new(0.0, 0.0, -1.0), 0.5);
    let material = DummyMaterial::new();

    let primitive = GeometricPrimitive::new(Box::new(sphere), Box::new(material));
    let lights: Vec<Box<Light>> = vec![
        Box::new(PointLight::new(Point3f::new(0.5, 0.5, 0.0), Spectrum::new(1.0, 0.0, 0.0))),
        Box::new(PointLight::new(Point3f::new(-0.5, -0.5, 0.0), Spectrum::new(0.0, 0.9, 0.3))),
//        Box::new(PointLight::new(Point3f::new(1.0, -2.0, 0.0), Spectrum::white())),
//        Box::new(PointLight::new(Point3f::new(4.0, 3.0, 3.0), Spectrum::white())),
//        Box::new(PointLight::new(Point3f::new(-4.0, -3.0, -3.0), Spectrum::white())),
    ];
    Scene::new(Box::new(primitive), lights)
}

fn build_sphere(center: Point3f, radius: Float) -> Sphere {
    let object_to_world = Transform::new(Matrix4::from_translation(center.to_vec()));
    let world_to_object = object_to_world.invert();

    Sphere::new(object_to_world, world_to_object, radius)
}
