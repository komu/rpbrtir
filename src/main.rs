extern crate cgmath;
extern crate image;
extern crate rand;
#[macro_use]
extern crate bitflags;

pub mod cameras;
pub mod core;
pub mod films;
pub mod integrators;
pub mod lights;
pub mod materials;
pub mod renderers;
pub mod shapes;
pub mod textures;

use core::{
    scene::Scene,
    geometry::Point3f,
    types::Float,
    primitive::GeometricPrimitive,
    film::Film,
    spectrum::Spectrum,
    light::Light,
};
use shapes::sphere::Sphere;
use lights::point::PointLight;
use cgmath::{vec3, prelude::*};
use renderers::samplerrenderer::SamplerRenderer;
use films::image::ImageFilm;
use core::primitive::CompoundPrimitive;
use core::primitive::Primitive;
use cameras::perspective::PerspectiveCamera;
use core::transform::{Transform, look_at, translate};
use rand::random;
use core::geometry::Ray;
use core::shape::Shape;
use core::material::Material;
use materials::matte::MatteMaterial;
use std::sync::Arc;
use textures::constant::ConstantTexture;
use textures::checkerboard::{Checkerboard2DTexture, AAMethod};
use core::texture::UVMapping2D;
use core::texture::IdentityMapping3D;
use textures::checkerboard::Checkerboard3DTexture;
use core::transform::scale;

fn main() {
    let scene = build_scene();

    let mut film = ImageFilm::new(String::from("images/output.png"), 600, 300);
    let origin = Point3f::new(0.0, 0.0, 0.0);

    let cam_to_world = look_at(&origin, &Point3f::new(0.0, 0.0, -1.0), &vec3(0.0, 1.0, 0.0));

    let aspect_ratio = film.aspect_ratio();
    let screen = if aspect_ratio > 1.0 {
        [-aspect_ratio, aspect_ratio, -1.0, 1.0]
    } else {
        [-1.0, 1.0, -1.0 / aspect_ratio, 1.0 / aspect_ratio]
    };

    {
        let mut cam = PerspectiveCamera::new(&cam_to_world, screen, 0.0, 1.0, 0.0, 1e30, 60.0, &mut film);

        let mut renderer = SamplerRenderer::new(&mut cam);
        renderer.render(&scene);
    }

    film.write_image();
}

fn color(r: &Ray) -> Spectrum {
    let s = build_sphere(Point3f::new(0.0, 0.0, -1.0), 0.5);
    if s.intersect_p(r) {
        return Spectrum::new(1.0, 0.0, 0.0);
    }

    let unit_direction = r.d.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    return (1.0 - t) * Spectrum::white() + t * Spectrum::new(0.5, 0.7, 1.0);
}

fn build_scene() -> Scene {
    let mut primitives: Vec<Box<Primitive>> = vec!(
        Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(0.0, 0.0, -1.0), 0.5)), dummy_material())),
//        Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(1.0, 0.0, -1.0), 0.2)), dummy_material()))
    );

    for _ in 0..10 {
        primitives.push(Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(-1.0 + 2.0 * random::<Float>(), 0.0, -1.0 + 2.0 * random::<Float>()), 0.2)), dummy_material())));
    }

    let lights: Vec<Box<Light>> = vec![
        Box::new(PointLight::new(Point3f::new(-0.5, -0.5, 0.0), Spectrum::new(0.0, 0.5, 0.0))),
        Box::new(PointLight::new(Point3f::new(0.5, 0.5, 0.0), Spectrum::new(0.0, 0.0, 0.5))),
        Box::new(PointLight::new(Point3f::new(0.2, 0.2, 0.0), Spectrum::new(0.7, 0.7, 0.7))),
//        Box::new(PointLight::new(Point3f::new(1.5, -0.0, 4.0), Spectrum::white())),
//        Box::new(PointLight::new(Point3f::new(-1.5, -0.0, -4.0), Spectrum::white())),
//        Box::new(PointLight::new(Point3f::new(-1.5, -0.5, 4.0), Spectrum::white())),
    ];

    Scene::new(Box::new(CompoundPrimitive::new(primitives)), lights)
}

fn dummy_material() -> Box<Material> {
    let white = Arc::new(ConstantTexture::new(Spectrum::white()));
    let blue = Arc::new(ConstantTexture::new(Spectrum::blue()));

    let checker = Checkerboard2DTexture::new(
        Box::new(UVMapping2D::new(10.0, 10.0, 0.0, 0.0)), white, blue, AAMethod::None);
//    let checker = Checkerboard3DTexture::new(
//        Box::new(IdentityMapping3D::new(scale(10.0, 10.0, 10.0))), white, blue);

    Box::new(MatteMaterial::new(
        Arc::new(checker),
        Arc::new(ConstantTexture::new(0.0)),
        None
    ))
}

fn build_sphere(center: Point3f, radius: Float) -> Sphere {
    let object_to_world = translate(&center.to_vec());
    let world_to_object = object_to_world.invert();

    Sphere::new(object_to_world, world_to_object, radius)
}
