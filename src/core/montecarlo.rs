use core::geometry::Vector3f;
use core::types::Float;
use core::types::PI;
use cgmath::vec3;
use superslice::*;

pub fn cosine_sample_hemisphere(u1: Float, u2: Float) -> Vector3f {
    let (x, y) = concentric_sample_disk(u1, u2);
    let z = (1.0 - x * x - y * y).max(0.0).sqrt();
    vec3(x, y, z)
}

pub fn uniform_sample_sphere(u1: Float, u2: Float) -> Vector3f {
    let z = 1.0 - 2.0 * u1;
    let r = (1.0 - z*z).max(0.0).sqrt();
    let phi = 2.0 * PI * u2;
    let x = r * phi.cos();
    let y = r * phi.sin();
    return vec3(x, y, z);
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

pub struct Distribution1D {
    func: Vec<Float>,
    cdf: Vec<Float>,
    func_int: Float
}

impl Distribution1D {
    pub fn new(f: &[Float]) -> Distribution1D {
        let n = f.len();
        let func = f.to_vec();
        let mut cdf = Vec::with_capacity(f.len() + 1);

        // Compute integral of step function at $x_i$
        cdf.push(0.0);
        for i in 0..n {
            let value = cdf[i] + func[i] / (n as Float);
            cdf.push(value);
        }

        // Transform step function integral into CDF
        let func_int = cdf[n];
        if func_int == 0.0 {
            for i in 1..(n + 1) {
                cdf[i] = (i as Float) / (n as Float);
            }
        } else {
            for i in 1..(n + 1) {
                cdf[i] /= func_int;
            }
        }

        Distribution1D { func, cdf, func_int }
    }

    pub fn sample_continuous(&self, u: Float) -> (Float, Float, usize) {
        // Find surrounding CDF segments and _offset_
        let offset = self.cdf.upper_bound_by(|y| y.partial_cmp(&u).unwrap());

        debug_assert!(offset < self.func.len());
        debug_assert!(u >= self.cdf[offset] && u < self.cdf[offset+1]);

        // Compute offset along CDF segment
        let du = (u - self.cdf[offset]) / (self.cdf[offset+1] - self.cdf[offset]);
        debug_assert!(!du.is_nan());

        // Compute PDF for sampled offset
        let pdf = self.func[offset] / self.func_int;

        // Return $x\in{}[0,1)$ corresponding to sample
        let s = (offset as Float + du) / (self.func.len() as Float);

        (s, pdf, offset)
    }

    pub fn sample_discrete(&self, u: Float) -> (usize, Float) {
        // Find surrounding CDF segments and _offset_
        let offset = self.cdf.upper_bound_by(|y| y.partial_cmp(&u).unwrap());
        debug_assert!(offset < self.func.len());
        debug_assert!(u >= self.cdf[offset] && u < self.cdf[offset+1]);
        let pdf = self.func[offset] / (self.func_int * (self.func.len() as Float));

        (offset, pdf)
    }
}