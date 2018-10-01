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

use rand::random;
use std::sync::Arc;
use cgmath::{vec3, prelude::*};
use core::{
    scene::Scene,
    geometry::{Ray, Point3f},
    types::Float,
    primitive::{GeometricPrimitive, CompoundPrimitive, Primitive},
    film::Film,
    spectrum::Spectrum,
    light::Light,
    transform::{Transform, look_at, translate, scale},
    shape::Shape,
    material::Material,
    texture::{UVMapping2D, IdentityMapping3D},
};
use shapes::Sphere;
use lights::PointLight;
use renderers::SamplerRenderer;
use films::ImageFilm;
use cameras::PerspectiveCamera;
use materials::MatteMaterial;
use textures::{ConstantTexture, Checkerboard2DTexture, Checkerboard3DTexture, AAMethod};

fn main() {
    let scene = build_scene();

    let eye = Point3f::new(13.0, 2.0, -3.0);
    let center = Point3f::new(0.0, 0.0, 0.0);
    let up = vec3(0.0, 1.0, 0.0);
    let aperture = 0.0;
    let focal_distance = 1e30;
    let fov = 20.0;

    let cam_to_world = look_at(&eye, &center, &up);

    let mut film = ImageFilm::new(String::from("images/output.png"), 800, 400);
    let aspect_ratio = film.aspect_ratio();
    let screen = if aspect_ratio > 1.0 {
        [-aspect_ratio, aspect_ratio, -1.0, 1.0]
    } else {
        [-1.0, 1.0, -1.0 / aspect_ratio, 1.0 / aspect_ratio]
    };

    {
        let mut cam = PerspectiveCamera::new(&cam_to_world, screen, 0.0, 1.0, aperture, focal_distance, fov, &mut film);

        let mut renderer = SamplerRenderer::new(&mut cam);
        renderer.render(&scene);
    }

    film.write_image();
}

fn build_scene() -> Scene {
    let mut primitives: Vec<Box<Primitive>> = Vec::new();

    primitives.push(Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(0.0, -1000.0, 0.0), 1000.0)), dummy_material(5000.0))));
    primitives.push(Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(0.0, 1.0, 0.0), 1.0)), dummy_material(10.0))));
    primitives.push(Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(-4.0, 1.0, 0.0), 1.0)), dummy_material(10.0))));
    primitives.push(Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(4.0, 1.0, 0.0), 1.0)), dummy_material(10.0))));

    for _ in 0..20 {
        primitives.push(Box::new(GeometricPrimitive::new(Box::new(build_sphere(Point3f::new(-4.0 + 8.0 * random::<Float>(), 0.5, -4.0 + 8.0 * random::<Float>()), 0.5)), dummy_material(10.0))));
    }

    let lights: Vec<Box<Light>> = vec![
        Box::new(PointLight::new(Point3f::new(0.0, 5.0, 0.0), 15.0 * Spectrum::new(1.0, 1.0, 1.0))),
        Box::new(PointLight::new(Point3f::new(-10.0, 5.0, 0.0), 15.0 * Spectrum::new(1.0, 1.0, 1.0))),
        Box::new(PointLight::new(Point3f::new(10.0, 5.0, 0.0), 15.0 * Spectrum::new(1.0, 1.0, 1.0))),
//        Box::new(PointLight::new(Point3f::new(0.0, 3.0, 0.0), Spectrum::new(0.0, 0.8, 0.0))),
//        Box::new(PointLight::new(Point3f::new(4.0, 3.0, 0.0), Spectrum::new(0.0, 0.8, 0.0))),
//        Box::new(PointLight::new(Point3f::new(14.0, 3.0, 0.0), Spectrum::new(0.0, 0.8, 0.0))),
//        Box::new(PointLight::new(Point3f::new(14.0, 3.0, 3.0), Spectrum::new(0.0, 0.8, 0.0))),
//        Box::new(PointLight::new(Point3f::new(14.0, 3.0, 7.0), Spectrum::new(0.0, 0.8, 1.0))),
//        Box::new(PointLight::new(Point3f::new(14.0, 4.0, 3.0), Spectrum::new(0.0, 0.8, 1.0))),
//        Box::new(PointLight::new(Point3f::new(-4.0, 3.0, 0.0), Spectrum::new(0.0, 0.8, 0.0))),
//        Box::new(PointLight::new(Point3f::new(-4.0, 4.0, 4.0), Spectrum::new(0.0, 0.8, 0.0))),
//        Box::new(PointLight::new(Point3f::new(-4.0, 4.0, -4.0), Spectrum::new(0.0, 0.8, 0.0))),
//        Box::new(PointLight::new(Point3f::new(-14.0, 10.0, -14.0), Spectrum::new(0.0, 0.8, 0.0))),
//        Box::new(PointLight::new(Point3f::new(-13.0, -2.0, -3.0), Spectrum::new(0.0, 0.0, 1.0))),
//        Box::new(PointLight::new(Point3f::new(0.0, -2.0, 14.0), Spectrum::new(0.0, 0.0, 1.0))),
//        Box::new(PointLight::new(Point3f::new(0.0, -2.0, -14.0), Spectrum::new(0.0, 0.0, 1.0))),
//        Box::new(PointLight::new(Point3f::new(13.0, 2.0, 3.0), Spectrum::new(0.0, 0.0, 0.2))),
//        Box::new(PointLight::new(Point3f::new(-13.0, -2.0, -3.0), Spectrum::new(0.0, 0.0, 0.2))),
//        Box::new(PointLight::new(Point3f::new(0.5, 0.5, 0.0), Spectrum::new(0.0, 0.0, 0.5))),
//        Box::new(PointLight::new(Point3f::new(0.2, 0.2, 0.0), Spectrum::new(0.7, 0.7, 0.7))),
//        Box::new(PointLight::new(Point3f::new(1.5, -0.0, 4.0), Spectrum::white())),
//        Box::new(PointLight::new(Point3f::new(-1.5, -0.0, -4.0), Spectrum::white())),
//        Box::new(PointLight::new(Point3f::new(-1.5, -0.5, 4.0), Spectrum::white())),
    ];

    Scene::new(Box::new(CompoundPrimitive::new(primitives)), lights)
}

fn dummy_material(scale: Float) -> Box<Material> {
    let white = Arc::new(ConstantTexture::new(Spectrum::white()));
    let blue = Arc::new(ConstantTexture::new(Spectrum::blue()));

    let checker = Checkerboard2DTexture::new(
        Box::new(UVMapping2D::new(scale, scale, 0.0, 0.0)), white, blue, AAMethod::None);
//    let checker = Checkerboard3DTexture::new(
//        Box::new(IdentityMapping3D::new(scale(10.0, 10.0, 10.0))), white, blue);

    Box::new(MatteMaterial::new(
        Arc::new(checker),
        Arc::new(ConstantTexture::new(0.0)),
        None,
    ))
}

fn build_sphere(center: Point3f, radius: Float) -> Sphere {
    let object_to_world = translate(&center.to_vec());
    let world_to_object = object_to_world.invert();

    Sphere::new(object_to_world, world_to_object, radius)
}
