extern crate rpbtrir;
extern crate cgmath;
extern crate rand;

use rand::random;
use std::sync::Arc;
use cgmath::{vec3, prelude::*};
use rpbtrir::{
    core::{
        film::Film,
        geometry::Point3f,
        light::Light,
        material::Material,
        primitive::{GeometricPrimitive, CompoundPrimitive, Primitive},
        scene::Scene,
        shape::Shape,
        spectrum::Spectrum,
        texture::UVMapping2D,
        transform::{look_at, translate},
        types::Float,
    },
    cameras::PerspectiveCamera,
    films::ImageFilm,
    filters::MitchellFilter,
    lights::PointLight,
    materials::{MatteMaterial, MetalMaterial},
    renderers::SamplerRenderer,
    shapes::Sphere,
    textures::{ConstantTexture, Checkerboard2DTexture, AAMethod},
};
use rpbtrir::core::light::AreaLight;
use rpbtrir::lights::DiffuseAreaLight;
use rpbtrir::materials::GlassMaterial;

fn main() {
    let scene = build_scene();

    let eye = Point3f::new(13.0, 2.0, -3.0);
    let center = Point3f::new(0.0, 0.0, 0.0);
    let up = vec3(0.0, 1.0, 0.0);
    let aperture = 0.05;
    let focal_distance = 15.0;
    let fov = 20.0;
    let samples_per_pixel = 8;

    let cam_to_world = look_at(&eye, &center, &up);

    let mut film = ImageFilm::new(String::from("images/output.png"), 800, 400, Box::new(MitchellFilter::default()));
    let aspect_ratio = film.aspect_ratio();
    let screen = if aspect_ratio > 1.0 {
        [-aspect_ratio, aspect_ratio, -1.0, 1.0]
    } else {
        [-1.0, 1.0, -1.0 / aspect_ratio, 1.0 / aspect_ratio]
    };

    {
        let mut cam = PerspectiveCamera::new(&cam_to_world, screen, 0.0, 1.0, aperture, focal_distance, fov, &mut film);

        let mut renderer = SamplerRenderer::new(&mut cam, samples_per_pixel);
        renderer.render(&scene);
    }

    film.write_image();
}

fn build_scene() -> Scene {
    let area_light = Some(diffuse_area_light(Point3f::new(0.0, 10.0, 0.0), 0.01));

    let mut primitives = vec![
        geometric_primitive(sphere(Point3f::new(0.0, -1000.0, 0.0), 1000.0), checker_matte(5000.0), area_light.clone()),
        geometric_primitive(sphere(Point3f::new(0.0, 1.0, 0.0), 1.0), glass(), area_light.clone()),
        geometric_primitive(sphere(Point3f::new(-4.0, 1.0, 0.0), 1.0), checker_matte(10.0), area_light.clone()),
        geometric_primitive(sphere(Point3f::new(4.0, 1.0, 0.0), 1.0), metal(), area_light.clone())
    ];

    for _ in 0..20 {
        let r = random::<Float>();
        let material = if r < 0.4 { metal() } else if r < 0.8 { glass() } else { checker_matte(10.0) };
        primitives.push(geometric_primitive(sphere(Point3f::new(-4.0 + 8.0 * random::<Float>(), 0.5, -4.0 + 8.0 * random::<Float>()), 0.5), material, area_light.clone()));
    }

    let lights = vec![
        point_light_white(Point3f::new(0.0, 5.0, 0.0), 15.0),
        point_light_white(Point3f::new(-10.0, 5.0, 0.0), 15.0),
        point_light_white(Point3f::new(10.0, 5.0, 0.0), 15.0),
        point_light_white(Point3f::new(7.0, 3.0, 0.0), 15.0),
    ];

    let root_primitive = Box::new(CompoundPrimitive::new(primitives));

    Scene::new(root_primitive, lights)
}

fn diffuse_area_light(center: Point3f, intensity: Float) -> Arc<AreaLight> {
    let object_to_world = translate(&center.to_vec());
    Arc::new(DiffuseAreaLight::new(object_to_world, intensity * Spectrum::white(), 8, sphere2(Point3f::new(0.0, 0.0, 0.0), 4.0)))
}

fn metal() -> Box<Material> {
    let eta = Arc::new(ConstantTexture::new(Spectrum::new(0.4, 0.2, 0.4)));
    let k = Arc::new(ConstantTexture::new(Spectrum::new(0.9, 0.4, 0.5)));
    let roughness = Arc::new(ConstantTexture::new(0.05));
    Box::new(MetalMaterial::new(eta, k, roughness, None))
}

fn glass() -> Box<Material> {
    Box::new(GlassMaterial::default())
}

fn checker_matte(scale: Float) -> Box<Material> {
    let white = Arc::new(ConstantTexture::new(Spectrum::white()));
    let blue = Arc::new(ConstantTexture::new(Spectrum::blue()));

    let checker = Checkerboard2DTexture::new(
        Box::new(UVMapping2D::new(scale, scale, 0.0, 0.0)), white, blue, AAMethod::None);

    Box::new(MatteMaterial::new(
        Arc::new(checker),
        Arc::new(ConstantTexture::new(0.0)),
        None,
    ))
}

fn geometric_primitive(shape: Box<Shape>, material: Box<Material>, area_light: Option<Arc<AreaLight>>) -> Box<Primitive> {
    Box::new(GeometricPrimitive::new(shape, material, area_light))
}

fn point_light_white(point: Point3f, intensity: Float) -> Box<Light> {
    Box::new(PointLight::new(point, intensity * Spectrum::white()))
}

fn sphere(center: Point3f, radius: Float) -> Box<Shape> {
    let object_to_world = translate(&center.to_vec());
    let world_to_object = object_to_world.invert();

    Box::new(Sphere::new(object_to_world, world_to_object, radius))
}

fn sphere2(center: Point3f, radius: Float) -> Arc<Shape> {
    let object_to_world = translate(&center.to_vec());
    let world_to_object = object_to_world.invert();

    Arc::new(Sphere::new(object_to_world, world_to_object, radius))
}
