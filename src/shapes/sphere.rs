use core::{
    differential_geometry::DifferentialGeometry,
    geometry::{Ray, Normal},
    math::{clamp, radians, solve_quadratic},
    shape::Shape,
    transform::Transform,
    types::{Float, PI},
};
use cgmath::{prelude::*, vec3};
use core::geometry::Vector3f;
use core::geometry::Point3f;
use core::montecarlo::uniform_sample_sphere;

#[derive(Debug)]
pub struct Sphere {
    object_to_world: Transform,
    world_to_object: Transform,
    radius: Float,
    zmin: Float,
    zmax: Float,
    theta_min: Float,
    theta_max: Float,
    phi_max: Float,
    reverse_orientation: bool
}

impl Sphere {
    pub fn new(object_to_world: Transform,
               world_to_object: Transform,
               radius: Float) -> Sphere {
        Sphere::new_clipped(object_to_world, world_to_object, radius, -radius, radius, 360.0)
    }

    pub fn new_clipped(object_to_world: Transform,
                       world_to_object: Transform,
                       radius: Float,
                       z0: Float,
                       z1: Float,
                       pm: Float) -> Sphere {
        let zmin = clamp(z0.min(z1), -radius, radius);
        let zmax = clamp(z0.max(z1), -radius, radius);
        Sphere {
            object_to_world,
            world_to_object,
            radius,
            zmin,
            zmax,
            theta_min: clamp(zmin / radius, -1.0, 1.0).acos(),
            theta_max: clamp(zmax / radius, -1.0, 1.0).acos(),
            phi_max: radians(clamp(pm, 0.0, 360.0)),
            reverse_orientation: false
        }
    }
}

impl Shape for Sphere {

    #[allow(non_snake_case)]
    fn intersect(&self, r: &Ray) -> Option<(DifferentialGeometry, Float, Float)> {
        let ray = self.world_to_object.transform_ray(r);
        let oc = ray.o.to_vec();
        let a = ray.d.dot(ray.d);
        let b = 2.0 * ray.d.dot(oc);
        let c = oc.dot(oc) - self.radius * self.radius;

        // Solve quadratic equation for _t_ values
        if let Some((t0, t1)) = solve_quadratic(a, b, c) {
            // Compute intersection distance along ray
            if t0 > ray.maxt || t1 < ray.mint {
                return None;
            }

            let mut t_hit = t0;
            if t0 < ray.mint {
                t_hit = t1;
                if t_hit > ray.maxt {
                    return None;
                }
            }

            // Compute sphere hit position and $\phi$
            let mut phit = ray.point_at(t_hit);
            if phit.x == 0.0 && phit.y == 0.0 {
                phit.x = 1e-5 * self.radius;
            }

            let mut phi = phit.y.atan2(phit.x);
            if phi < 0.0 {
                phi += 2.0 * PI;
            }

            // Test sphere intersection against clipping parameters
            if (self.zmin > -self.radius && phit.z < self.zmin) || (self.zmax < self.radius && phit.z > self.zmax || phi > self.phi_max) {
                if t_hit == t1 || t1 > ray.maxt {
                    return None;
                }

                t_hit = t1;

                // Compute sphere hit position and $\phi$
                phit = ray.point_at(t_hit);

                if phit.x == 0.0 && phit.y == 0.0 {
                    phit.x = 1e-5 * self.radius;
                }
                phi = phit.y.atan2(phit.x);
                if phi < 0.0 {
                    phi += 2.0 * PI;
                }
                if (self.zmin > -self.radius && phit.z < self.zmin) || (self.zmax < self.radius && phit.z > self.zmax) || (phi > self.phi_max) {
                    return None;
                }
            }

            // Find parametric representation of sphere hit
            let u = phi / self.phi_max;
            let theta = clamp(phit.z / self.radius, -1.0, 1.0).acos();
            let v = (theta - self.theta_min) / (self.theta_max - self.theta_min);

            // Compute sphere $\dpdu$ and $\dpdv$
            let zradius = (phit.x * phit.x + phit.y * phit.y).sqrt();
            let invzradius = 1.0 / zradius;
            let cosphi = phit.x * invzradius;
            let sinphi = phit.y * invzradius;
            let dpdu = vec3(-self.phi_max * phit.y, self.phi_max * phit.x, 0.0);
            let dpdv = (self.theta_max - self.theta_min) *
                vec3(phit.z * cosphi, phit.z * sinphi, -self.radius * theta.sin());

            // Compute sphere $\dndu$ and $\dndv$
            let d2pduu = -self.phi_max * self.phi_max * vec3(phit.x, phit.y, 0.0);
            let d2pduv = (self.theta_max - self.theta_min) * phit.z * self.phi_max * vec3(-sinphi, cosphi, 0.0);
            let d2pdvv = -(self.theta_max - self.theta_min) * (self.theta_max - self.theta_min) * phit.to_vec();

            // Compute coefficients for fundamental forms
            let E = dpdu.dot(dpdu);
            let F = dpdu.dot(dpdv);
            let G = dpdv.dot(dpdv);
            let N = dpdu.cross(dpdv).normalize();
            let e = N.dot(d2pduu);
            let f = N.dot(d2pduv);
            let g = N.dot(d2pdvv);

            // Compute $\dndu$ and $\dndv$ from fundamental form coefficients
            let inv_egf2 = 1.0 / (E * G - F * F);
            let dndu = Normal::from_vector((f * F - e * G) * inv_egf2 * dpdu + (e * F - f * E) * inv_egf2 * dpdv);
            let dndv = Normal::from_vector((g * F - f * G) * inv_egf2 * dpdu + (f * F - g * E) * inv_egf2 * dpdv);

            // Initialize _DifferentialGeometry_ from parametric information

            let dg = DifferentialGeometry::new(
                self.object_to_world.transform_point(phit),
                self.object_to_world.transform_vector(dpdu),
                self.object_to_world.transform_vector(dpdv),
                self.object_to_world.transform_normal(dndu),
                self.object_to_world.transform_normal(dndv),
                u,
                v,
                self);

            // Compute _rayEpsilon_ for quadric intersection
            let ray_epsilon = 5e-4 * t_hit;

            Some((dg, t_hit, ray_epsilon))
        } else {
            None
        }
    }

    fn pdf(&self, p: &Point3f, wi: &Vector3f) -> Float {
        unimplemented!()
        /*
        let Pcenter = (*ObjectToWorld)(Point(0,0,0));
        // Return uniform weight if point inside sphere
        if (distance_squared(p, Pcenter) - radius*radius < 1e-4)
        return Shape::Pdf(p, wi);

        // Compute general sphere weight
        float sinThetaMax2 = radius*radius / DistanceSquared(p, Pcenter);
        float cosThetaMax = sqrtf(max(0.f, 1.f - sinThetaMax2));
        return UniformConePdf(cosThetaMax);
        */
    }

    fn get_object_to_world(&self) -> &Transform {
        &self.object_to_world
    }

    fn area(&self) -> Float {
        self.phi_max * self.radius * (self.zmax - self.zmin)
    }

    fn sample_point(&self, p: &Point3f, u1: Float, u2: Float) -> (Point3f, Normal) {
        /*
        Point Sphere::Sample(const Point &p, float u1, float u2,
                     Normal *ns) const {
    // Compute coordinate system for sphere sampling
    Point Pcenter = (*ObjectToWorld)(Point(0,0,0));
    Vector wc = Normalize(Pcenter - p);
    Vector wcX, wcY;
    CoordinateSystem(wc, &wcX, &wcY);

    // Sample uniformly on sphere if $\pt{}$ is inside it
    if (DistanceSquared(p, Pcenter) - radius*radius < 1e-4f)
        return Sample(u1, u2, ns);

    // Sample sphere uniformly inside subtended cone
    float sinThetaMax2 = radius*radius / DistanceSquared(p, Pcenter);
    float cosThetaMax = sqrtf(max(0.f, 1.f - sinThetaMax2));
    DifferentialGeometry dgSphere;
    float thit, rayEpsilon;
    Point ps;
    Ray r(p, UniformSampleCone(u1, u2, cosThetaMax, wcX, wcY, wc), 1e-3f);
    if (!Intersect(r, &thit, &rayEpsilon, &dgSphere))
        thit = Dot(Pcenter - p, Normalize(r.d));
    ps = r(thit);
    *ns = Normal(Normalize(ps - Pcenter));
    if (ReverseOrientation) *ns *= -1.f;
    return ps;
}
*/
        unimplemented!()
    }

    fn sample(&self, u1: Float, u2: Float) -> (Point3f, Normal) {
        let p = Point3f::new(0.0, 0.0, 0.0) + self.radius * uniform_sample_sphere(u1, u2);
        let mut ns = self.object_to_world.transform_normal(Normal::new(p.x, p.y, p.z)).normalize();
        if self.reverse_orientation {
            ns *= -1.0;
        }

        (self.object_to_world.transform_point(p), ns)
    }
}
