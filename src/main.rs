extern crate cgmath;
extern crate image;
extern crate rand;
#[macro_use]
extern crate bitflags;

pub mod core;
pub mod integrators;
pub mod lights;
pub mod shapes;

use core::{
    shape::Shape,
    scene::Scene,
    geometry::{Ray, Point3f},
    material::DummyMaterial,
    transform::Transform,
    types::Float,
};
use shapes::sphere::Sphere;
use cgmath::{vec3, prelude::*};
use cgmath::Matrix4;
use image::ImageBuffer;
use core::primitive::GeometricPrimitive;
use integrators::whitted::WhittedIntegrator;
use core::integrator::SurfaceIntegrator;
use core::rng::RNG;
use core::spectrum::Spectrum;
use lights::point::PointLight;
use core::light::Light;

fn main() {
    let nx = 600;
    let ny = 300;

    let lower_left_corner = Point3f::new(-2.0, -1.0, -1.0);
    let horizontal = vec3(4.0, 0.0, 0.0);
    let vertical = vec3(0.0, 2.0, 0.0);
    let origin = Point3f::new(0.0, 0.0, 0.0);
    let integrator = WhittedIntegrator::new(50);
    let scene = build_scene();
    let rng = RNG {};

    let img = ImageBuffer::from_fn(nx, ny, |i, j| {
        let j = ny - j;

        let u = (i as Float) / (nx as Float);
        let v = (j as Float) / (ny as Float);

        let mut r = Ray::new_simple(origin, lower_left_corner.to_vec() + u * horizontal + v * vertical);
        color(&scene, &integrator, &mut r, &rng).to_rgb()
    });

    img.save("images/output.png").unwrap();
}

fn color(scene: &Scene, integrator: &SurfaceIntegrator, r: &mut Ray, rng: &RNG) -> Spectrum {
    if let Some(isect) = scene.intersect(r) {
        let spectrum = integrator.li(scene, None, r, &isect, None, rng);
        return spectrum;
//        let n = isect.dg.nn.v;
//        return 0.5 * Spectrum::new(n.x + 1.0, n.y + 1.0, n.z + 1.0);
    }

    let unit_direction = r.d.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    return (1.0 - t) * Spectrum::white() + t * Spectrum::new(0.5, 0.7, 1.0);
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
