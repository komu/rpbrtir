use core::camera::{Camera, ProjectiveCamera};
use core::geometry::Ray;
use core::sampler::CameraSample;
use core::types::Float;
use core::film::Film;
use core::geometry::{Point3f, Vector3f};
use core::transform::{Transform, scale, translate};
use cgmath::{vec3, prelude::*};
use core::transform::perspective;
use core::geometry::RayDifferential;
use core::types::INFINITY;
use core::montecarlo::concentric_sample_disk;

pub struct PerspectiveCamera<'a> {
    film: &'a mut Film,
    shutter_open: Float,
    shutter_close: Float,
    camera_to_world: Transform,
    camera_to_screen: Transform,
    raster_to_camera: Transform,
    screen_to_raster: Transform,
    raster_to_screen: Transform,
    lens_radius: Float,
    focal_distance: Float,
    dx_camera: Vector3f,
    dy_camera: Vector3f,
}

impl<'a> PerspectiveCamera<'a> {
    pub fn new<'b>(camera_to_world: &Transform,
                   screen_window: [Float; 4],
                   shutter_open: Float,
                   shutter_close: Float,
                   lens_radius: Float,
                   focal_distance: Float,
                   fov: Float,
                   film: &'b mut Film) -> PerspectiveCamera<'b> {
        // Compute projective camera transformations
        let camera_to_screen = perspective(fov, 1e-2, 1000.0);

        // Compute projective camera screen transformations
        let (res_x, res_y) = film.resolution();
        let screen_to_raster =
            &scale(res_x as Float, res_y as Float, 1.0) *
                &scale(1.0 / (screen_window[1] - screen_window[0]), 1.0 / (screen_window[2] - screen_window[3]), 1.0) *
                &translate(&vec3(-screen_window[0], -screen_window[3], 0.0));
        let raster_to_screen = screen_to_raster.invert();
        let raster_to_camera = camera_to_screen.invert() * &raster_to_screen;

        let dx_camera = &raster_to_camera.transform_point(Point3f::new(1.0, 0.0, 0.0)) - raster_to_camera.transform_point(Point3f::new(0.0, 0.0, 0.0));
        let dy_camera = &raster_to_camera.transform_point(Point3f::new(0.0, 1.0, 0.0)) - raster_to_camera.transform_point(Point3f::new(0.0, 0.0, 0.0));
        PerspectiveCamera {
            film,
            shutter_open,
            shutter_close,
            camera_to_world: camera_to_world.clone(),
            camera_to_screen,
            raster_to_camera,
            screen_to_raster,
            raster_to_screen,
            lens_radius,
            focal_distance,
            dx_camera,
            dy_camera,
        }
    }
}

impl<'a> Camera for PerspectiveCamera<'a> {
    fn generate_ray(&self, sample: &CameraSample) -> (Ray, Float) {
        // Generate raster and camera samples
        let pras = Point3f::new(sample.image_x, sample.image_y, 0.0);
        let pcamera = self.raster_to_camera.transform_point(pras);

        let mut ray = Ray::new(Point3f::new(0.0, 0.0, 0.0), pcamera.to_vec().normalize(), 0.0, INFINITY, sample.time);

        // Modify ray for depth of field
        if self.lens_radius > 0.0 {
            // Sample point on lens
            let (mut lens_u, mut lens_v) = concentric_sample_disk(sample.lens_u, sample.lens_v);
            lens_u *= self.lens_radius;
            lens_v *= self.lens_radius;

            // Compute point on plane of focus
            let ft = self.focal_distance / ray.d.z;
            let pfocus = ray.point_at(ft);

            // Update ray for effect of lens
            ray.o = Point3f::new(lens_u, lens_v, 0.0);
            ray.d = (pfocus - ray.o).normalize();
        }

        return (self.camera_to_world.transform_ray(&ray), 1.0);
    }

    fn generate_ray_differential(&self, sample: &CameraSample) -> (RayDifferential, Float) {
        // Generate raster and camera samples
        let pras = Point3f::new(sample.image_x, sample.image_y, 0.0);
        let pcamera = self.raster_to_camera.transform_point(pras);

        let dir = pcamera.to_vec().normalize();
        let mut rd = RayDifferential::new(Point3f::new(0.0, 0.0, 0.0), dir, 0.0, INFINITY, sample.time);

        // Modify ray for depth of field
        if self.lens_radius > 0.0 {
            // Sample point on lens
            let (mut lens_u, mut lens_v) = concentric_sample_disk(sample.lens_u, sample.lens_v);
            lens_u *= self.lens_radius;
            lens_v *= self.lens_radius;

            // Compute point on plane of focus
            let ft = self.focal_distance / rd.ray.d.z;
            let pfocus = rd.ray.point_at(ft);

            // Update ray for effect of lens
            rd.ray.o = Point3f::new(lens_u, lens_v, 0.0);
            rd.ray.d = (pfocus - rd.ray.o).normalize();
        }

        // Compute offset rays for _PerspectiveCamera_ ray differentials
        if self.lens_radius > 0.0 {
            // Compute _PerspectiveCamera_ ray differentials with defocus blur

            // Sample point on lens
            let (mut lens_u, mut lens_v) = concentric_sample_disk(sample.lens_u, sample.lens_v);
            lens_u *= self.lens_radius;
            lens_v *= self.lens_radius;

            let dx = (pcamera + self.dx_camera).to_vec().normalize();
            let mut ft = self.focal_distance / dx.z;
            let mut p_focus = Point3f::new(0.0, 0.0, 0.0) + (ft * dx);
            rd.rx_origin = Point3f::new(lens_u, lens_v, 0.0);
            rd.rx_direction = (p_focus - rd.rx_origin).normalize();

            let dy = (pcamera + self.dy_camera).to_vec().normalize();
            ft = self.focal_distance / dy.z;
            p_focus = Point3f::new(0.0, 0.0, 0.0) + (ft * dy);
            rd.ry_origin = Point3f::new(lens_u, lens_v, 0.0);
            rd.ry_direction = (p_focus - rd.ry_origin).normalize();
        } else {
            rd.rx_origin = rd.ray.o;
            rd.ry_origin = rd.ray.o;
            rd.rx_direction = (pcamera.to_vec() + self.dx_camera).normalize();
            rd.ry_direction = (pcamera.to_vec() + self.dy_camera).normalize();
        }

        rd.ray = self.camera_to_world.transform_ray(&rd.ray);
        rd.has_differentials = true;
        return (rd, 1.0);
    }

    fn get_film(&mut self) -> &mut Film {
        self.film
    }
}

impl<'a> ProjectiveCamera for PerspectiveCamera<'a> {}
