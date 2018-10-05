use core::integrator::SurfaceIntegrator;
use integrators::whitted::WhittedIntegrator;
use core::renderer::Renderer;
use core::scene::Scene;
use core::geometry::RayDifferential;
use core::sampler::{Sample, Sampler};
use core::rng::RNG;
use core::spectrum::Spectrum;
use core::types::Float;
use core::camera::Camera;
use core::integrator::VolumeIntegrator;
use core::integrator::NoOpVolumeIntegrator;
use samplers::RandomSampler;
use core::sampler::SamplerWindow;

pub struct SamplerRenderer<'a> {
    integrator: Box<SurfaceIntegrator>,
    volume_integrator: Box<VolumeIntegrator>,
    camera: &'a mut Camera
}

impl <'a> SamplerRenderer<'a> {
    pub fn new(camera: &mut Camera) -> SamplerRenderer {
        SamplerRenderer {
            integrator: Box::new(WhittedIntegrator::new(50)),
            volume_integrator: Box::new(NoOpVolumeIntegrator {}),
            camera
        }
    }

    pub fn render(&mut self, scene: &Scene) {
        let (nx, ny) = self.camera.get_film().resolution();
        let mut rng = RNG::new();

        let win = SamplerWindow::from_dimensions(nx, ny);
        let mut sampler = RandomSampler::new(win, 4, 0.0, 1.0);

        let mut sample = Sample::default();

        loop {
            let count = sampler.get_more_samples(&mut sample, &mut rng);
            if count == 0 {
                break
            }

            let (mut r, _) = self.camera.generate_ray_differential(&sample.cam);
            let li = self.li(&scene, &mut r, None, &mut rng);

            self.camera.get_film().add_sample(&sample.cam, &li);
        }
    }
}

impl <'a> Renderer for SamplerRenderer<'a> {
    fn li(&self, scene: &Scene, rd: &mut RayDifferential, sample: Option<&Sample>, rng: &mut RNG) -> Spectrum {
        let li = if let Some(mut isect) = scene.intersect(&mut rd.ray) {
            self.integrator.li(scene, self, rd, &mut isect, sample, rng)
        } else {
            scene.lights.iter().map(|l| { l.le(rd) }).sum()
        };

        let (lvi, t) = self.volume_integrator.li(scene, self, rd, sample, rng);

        t * li + lvi
    }

    fn transmittance(&self, scene: &Scene, ray: &RayDifferential, sample: Option<&Sample>, rng: &RNG) -> Float {
        self.volume_integrator.transmittance(scene, self, ray, sample, rng)
    }
}