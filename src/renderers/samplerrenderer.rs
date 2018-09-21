use core::integrator::SurfaceIntegrator;
use integrators::whitted::WhittedIntegrator;
use core::renderer::Renderer;
use core::scene::Scene;
use core::geometry::{Point3f, RayDifferential};
use core::sampler::Sample;
use core::rng::RNG;
use core::spectrum::Spectrum;
use core::types::{Float};
use core::film::Film;
use cgmath::{vec3, prelude::*};

pub struct SamplerRenderer<'a> {
    integrator: Box<SurfaceIntegrator>,
    film: &'a mut Film
}

impl <'a> SamplerRenderer<'a> {
    pub fn new(film: &mut Film) -> SamplerRenderer {
        SamplerRenderer {
            integrator: Box::new(WhittedIntegrator::new(50)),
            film
        }
    }

    pub fn render(&mut self, scene: &Scene) {
        let ny = self.film.y_resolution();
        let nx = self.film.x_resolution();
        let lower_left_corner = Point3f::new(-2.0, -1.0, -1.0);
        let horizontal = vec3(4.0, 0.0, 0.0);
        let vertical = vec3(0.0, 2.0, 0.0);
        let origin = Point3f::new(0.0, 0.0, 0.0);
        let mut rng = RNG::new();

        for y in 0..ny {
            for x in 0..nx {
                let i = x;
                let j = ny - y;

                let u = (i as Float) / (nx as Float);
                let v = (j as Float) / (ny as Float);

                let mut r = RayDifferential::new_simple(origin, lower_left_corner.to_vec() + u * horizontal + v * vertical);
                let li = self.li(&scene, &mut r, None, &mut rng);
                self.film.put_pixel(x, y, &li);
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