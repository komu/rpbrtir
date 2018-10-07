use core::{
    spectrum::Spectrum,
    geometry::{Point3f, Vector3f},
    types::{Float, INFINITY},
    scene::Scene,
};
use core::sampler::Sample;
use core::renderer::Renderer;
use core::rng::RNG;
use core::geometry::{Ray, RayDifferential};
use core::geometry::distance;
use core::geometry::Normal;
use core::shape::Shape;
use std::sync::Arc;
use core::montecarlo::Distribution1D;
use core::sampler::{SampleOffset1d, SampleOffset2d};

pub trait Light {
    fn sample_l(
        &self,
        p: &Point3f,
        p_epsilon: Float,
        ls: &LightSample,
        time: Float,
        visibility: &mut VisibilityTester)
        -> (Spectrum, Vector3f, Float);

    fn le(&self, ray: &RayDifferential) -> Spectrum {
        Spectrum::black()
    }

    fn pdf(&self, p: &Point3f, wi: &Vector3f) -> Float;

    fn power(&self, scene: &Scene) -> Spectrum;

    fn num_samples(&self) -> u32;

    fn is_delta_light(&self) -> bool;
}

pub trait AreaLight : Light {
    fn l(&self, p: &Point3f, n: &Normal, w: &Vector3f) -> Spectrum;
}

pub struct LightSample {
    pub u_component: Float,
    pub u_pos: [Float; 2],
}

impl LightSample {
    pub fn new(sample: &Sample, offsets: &LightSampleOffsets, n: usize) -> LightSample {
        let u_pos = [
            sample[offsets.pos_offset][2 * n],
            sample[offsets.pos_offset][2 * n + 1]
        ];
        let u_component = sample[offsets.component_offset][n];

        debug_assert!(u_pos[0] >= 0.0 && u_pos[0] < 1.0);
        debug_assert!(u_pos[1] >= 0.0 && u_pos[1] < 1.0);
        debug_assert!(u_component >= 0.0 && u_component < 1.0);

        LightSample { u_pos, u_component }
    }

    pub fn gen(rng: &mut RNG) -> LightSample {
        LightSample {
            u_component: rng.random_float(),
            u_pos: [rng.random_float(), rng.random_float()]
        }
    }
}

pub struct LightSampleOffsets {
    pub n_samples: u32,
    pub component_offset: SampleOffset1d,
    pub pos_offset: SampleOffset2d,
}

impl LightSampleOffsets {
    pub fn new(count: u32, sample: &mut Sample) -> LightSampleOffsets {
        LightSampleOffsets {
            n_samples: count,
            component_offset: sample.add_1d(count),
            pos_offset: sample.add_2d(count),
        }
    }
}

pub struct VisibilityTester {
    ray: Option<Ray>
}

impl VisibilityTester {
    pub fn new() -> VisibilityTester {
        VisibilityTester { ray: None }
    }

    pub fn unoccluded(&self, scene: &Scene) -> bool {
        !scene.intersect_p(self.ray())
    }

    pub fn transmittance(&self, scene: &Scene, renderer: &Renderer, sample: Option<&Sample>, rng: &RNG) -> Float {
        renderer.transmittance(scene, &RayDifferential::from_ray(self.ray().clone()), sample, rng)
    }

    fn ray(&self) -> &Ray {
        self.ray.as_ref().expect("no ray for VisibilityTester")
    }

    pub fn set_segment(&mut self, p1: Point3f, eps1: Float, p2: Point3f, eps2: Float, time: Float) {
        let dist = distance(&p1, &p2);
        let r = Ray::new(p1, (p2 - p1) / dist, eps1, dist * (1.0 - eps2), time);
        debug_assert!(!r.has_nans());
        self.ray = Some(r);
    }
}

pub struct ShapeSet {
    shapes: Vec<Arc<Shape>>,
    areas: Vec<Float>,
    sum_area: Float,
    area_distribution: Distribution1D
}

impl ShapeSet {

    pub fn new(shape: Arc<Shape>) -> ShapeSet {
        let mut shapes = vec![];
        let mut todo = vec![shape];

        while !todo.is_empty() {
            let sh = todo.pop().unwrap();
            if sh.can_intersect() {
                shapes.push(sh);
            } else {
                sh.refine(&mut todo);
            }
        }

        let areas: Vec<Float> = shapes.iter().map(|s| s.area()).collect();
        let sum_area = areas.iter().sum();
        let area_distribution = Distribution1D::new(&areas);

        ShapeSet { shapes, areas, sum_area, area_distribution }
    }

    pub fn area(&self) -> Float {
        self.sum_area
    }

    pub fn sample_point(&self, p: &Point3f, ls: &LightSample) -> (Point3f, Normal) {
        let (sn, _) = self.area_distribution.sample_discrete(ls.u_component);
        let (pt, mut nn) = self.shapes[sn].as_ref().sample_point(p, ls.u_pos[0], ls.u_pos[1]);

        // Find closest intersection of ray with shapes in _ShapeSet_
        let r = Ray::new(*p, pt-p, 1e-3, INFINITY, 0.0);
        let mut thit = 1.0;

        for sh in &self.shapes {
            if let Some((dg, th, _)) = sh.intersect(&r) {
                nn = dg.nn;
                thit = th;
            }
        }

        return (r.point_at(thit), nn);
    }

    pub fn sample(&self, ls: &LightSample, ns: &Normal) -> (Point3f, Normal) {
        let (sn, _) = self.area_distribution.sample_discrete(ls.u_component);
        self.shapes[sn].as_ref().sample(ls.u_pos[0], ls.u_pos[1])
    }

    pub fn pdf(&self, p: &Point3f, wi: &Vector3f) -> Float {
        let pdf: Float = self.shapes.iter().zip(&self.areas)
            .map(|(sh,area)| area * sh.pdf(p, wi))
            .sum();

        pdf / self.sum_area
    }
}
