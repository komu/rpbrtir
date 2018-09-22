use core::integrator::SurfaceIntegrator;
use integrators::whitted::WhittedIntegrator;
use core::renderer::Renderer;
use core::scene::Scene;
use core::geometry::{Point3f, RayDifferential};
use core::sampler::Sample;
use core::rng::RNG;
use core::spectrum::Spectrum;
use core::types::{Float};
use cgmath::vec3;
use core::camera::Camera;
use core::sampler::CameraSample;

pub struct SamplerRenderer<'a> {
    integrator: Box<SurfaceIntegrator>,
    camera: &'a mut Camera
}

impl <'a> SamplerRenderer<'a> {
    pub fn new(camera: &mut Camera) -> SamplerRenderer {
        SamplerRenderer {
            integrator: Box::new(WhittedIntegrator::new(50)),
            camera
        }
    }

    pub fn render(&mut self, scene: &Scene) {
        let (nx, ny) = self.camera.get_film().resolution();
        let lower_left_corner = Point3f::new(-2.0, -1.0, -1.0);
        let horizontal = vec3(4.0, 0.0, 0.0);
        let vertical = vec3(0.0, 2.0, 0.0);
        let origin = Point3f::new(0.0, 0.0, 0.0);
        let mut rng = RNG::new();

        for y in 0..ny {
            for x in 0..nx {
                let i = x;
                let j = ny - y;

                let sample = CameraSample {
                    image_x: x as Float,
                    image_y: y as Float,
                    lens_u: 0.0, // TODO
                    lens_v: 0.0,
                    time: 0.0
                };

                let (mut r, _) = self.camera.generate_ray_differential(&sample);
//                let mut r = RayDifferential::new_simple(origin, lower_left_corner.to_vec() + u * horizontal + v * vertical);
                r.has_differentials = false; // TODO disable differentials because they are not fully supported
                let li = self.li(&scene, &mut r, None, &mut rng);

//                self.camera.get_film().add_sample(&sample, &li);
                self.camera.get_film().put_pixel(x, y, &li);
            }
        }
    }
}

impl <'a> Renderer for SamplerRenderer<'a> {
    fn li(&self, scene: &Scene, rd: &mut RayDifferential, sample: Option<&Sample>, rng: &mut RNG) -> Spectrum {
        if let Some(mut isect) = scene.intersect(&mut rd.ray) {
            self.integrator.li(scene, self, rd, &mut isect, sample, rng)
        } else {
            scene.lights.iter().map(|l| { l.le(rd) }).sum()
        }

        // TODO: add contribution from volume integrator
    }

    fn transmittance(&self, _scene: &Scene, _rd: &RayDifferential, _sample: Option<&Sample>, _rng: &RNG) -> Float {
        1.0 // TODO
    }
}