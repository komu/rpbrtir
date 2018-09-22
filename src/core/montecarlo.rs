use core::geometry::Vector3f;
use core::types::Float;
use core::types::PI;
use cgmath::vec3;

pub fn cosine_sample_hemisphere(u1: Float, u2: Float) -> Vector3f {
    let (x, y) = concentric_sample_disk(u1, u2);
    let z = (1.0 - x * x - y * y).max(0.0).sqrt();
    vec3(x, y, z)
}


pub fn concentric_sample_disk(u1: Float, u2: Float) -> (Float, Float) {
    // Map uniform random numbers to $[-1,1]^2$
    let sx = 2.0 * u1 - 1.0;
    let sy = 2.0 * u2 - 1.0;

    // Map square to $(r,\theta)$

    // Handle degeneracy at the origin
    if sx == 0.0 && sy == 0.0 {
        return (0.0, 0.0);
    }

    let (r, mut theta) = if sx >= -sy {
        if sx > sy {
            // Handle first region of disk
            if sy > 0.0 {
                (sx, sy / sx)
            } else {
                (sx, 8.0 + sy / sx)
            }
        } else {
            // Handle second region of disk
            (sy, 2.0 - sx / sy)
        }
    } else {
        if sx <= sy {
            // Handle third region of disk
            (-sx, 4.0 - sy / -sx)
        } else {
            // Handle fourth region of disk
            (-sy, 6.0 + sx / -sy)
        }
    };

    theta *= PI / 4.0;

    (r * theta.cos(), r * theta.sin())
}
