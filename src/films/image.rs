use core::film::Film;
use core::film::Extent;
use core::sampler::CameraSample;
use core::spectrum::Spectrum;
use core::types::Float;
use image::{ImageBuffer, Rgb};
use core::math::clamp;
use core::filter::Filter;
use array_init::array_init;

const FILTER_TABLE_SIZE: usize = 16;

type FilterTable = [Float; FILTER_TABLE_SIZE * FILTER_TABLE_SIZE];

pub struct ImageFilm {
    path: String,
    x_pixel_start: usize,
    y_pixel_start: usize,
    x_pixel_count: usize,
    y_pixel_count: usize,
    img: Vec<Pixel>,
    filter: Box<Filter>,
    filter_table: FilterTable,
}

#[derive(Default)]
struct Pixel {
    r: Float,
    g: Float,
    b: Float,
    weight_sum: Float,
}

impl Pixel {
    fn to_rgb(&self) -> Rgb<u8> {
        let ir = image_component(self.r, self.weight_sum);
        let ig = image_component(self.g, self.weight_sum);
        let ib = image_component(self.b, self.weight_sum);

        Rgb([ir, ig, ib])
    }
}

#[inline]
fn image_component(x: Float, weight_sum: Float) -> u8 {
    if weight_sum == 0.0 { return 0; }
    let r = clamp(x / weight_sum, 0.0, 1.0);
    (255.99 * r.sqrt()) as u8
}

impl ImageFilm {
    pub fn new(path: String, x_resolution: u32, y_resolution: u32, filter: Box<Filter>) -> ImageFilm {
        let filter_table = precompute_filter_table(&filter);
        ImageFilm {
            path,
            x_pixel_start: 0,
            x_pixel_count: x_resolution as usize,
            y_pixel_start: 0,
            y_pixel_count: y_resolution as usize,
            filter,
            img: (0..x_resolution * y_resolution).map(|_| Pixel::default()).collect(),
            filter_table,
        }
    }
}

fn precompute_filter_table(filter: &Box<Filter>) -> FilterTable {
    let dimensions = filter.dimensions();

    array_init(|i| {
        let y = i / FILTER_TABLE_SIZE;
        let x = i % FILTER_TABLE_SIZE;
        let fy = ((y as Float) + 0.5) * dimensions.y_width / (FILTER_TABLE_SIZE as Float);
        let fx = ((x as Float) + 0.5) * dimensions.x_width / (FILTER_TABLE_SIZE as Float);
        filter.evaluate(fx, fy)
    })
}

impl Film for ImageFilm {
    fn add_sample(&mut self, sample: &CameraSample, l: &Spectrum) {
        let filter = self.filter.dimensions();
        let dimage_x = sample.image_x - 0.5;
        let dimage_y = sample.image_y - 0.5;
        let x0 = (dimage_x - filter.x_width).ceil() as usize;
        let x1 = (dimage_x + filter.x_width).floor() as usize;
        let y0 = (dimage_y - filter.y_width).ceil() as usize;
        let y1 = (dimage_y + filter.y_width).floor() as usize;
        let x0 = x0.max(self.x_pixel_start);
        let x1 = x1.min(self.x_pixel_start + self.x_pixel_count - 1);
        let y0 = y0.max(self.y_pixel_start);
        let y1 = y1.min(self.y_pixel_start + self.y_pixel_count - 1);
        if x1 < x0 || y1 < y0 {
            //PBRT_SAMPLE_OUTSIDE_IMAGE_EXTENT(const_cast<CameraSample *>(&sample));
            return;
        }

        let r = l.r;
        let g = l.g;
        let b = l.b;

        // Precompute $x$ and $y$ filter table offsets

        let mut ifx: Vec<usize> = Vec::with_capacity((x1 - x0 + 1) as usize);
        for x in x0..(x1 + 1) {
            let fx = ((x as Float - dimage_x) * filter.inv_x_width * FILTER_TABLE_SIZE as Float).abs();
            ifx.push((fx.floor() as usize).min(FILTER_TABLE_SIZE - 1));
        }

        let mut ify: Vec<usize> = Vec::with_capacity((y1 - y0 + 1) as usize);
        for y in y0..(y1 + 1) {
            let fy = ((y as Float - dimage_y) * filter.inv_y_width * FILTER_TABLE_SIZE as Float).abs();
            ify.push((fy.floor() as usize).min(FILTER_TABLE_SIZE - 1));
        }

        for y in y0..(y1 + 1) {
            for x in x0..(x1 + 1) {
                // Evaluate filter value at $(x,y)$ pixel
                let offset = ify[y - y0] * FILTER_TABLE_SIZE + ifx[x - x0];
                let filter_wt = self.filter_table[offset];

                // Update pixel values with filtered sample contribution
                let index = ((y - self.y_pixel_start) * self.x_pixel_count + (x - self.x_pixel_start)) as usize;
                let pixel = &mut self.img[index];
                pixel.r += filter_wt * r;
                pixel.g += filter_wt * g;
                pixel.b += filter_wt * b;
                pixel.weight_sum += filter_wt;
            }
        }
    }

    fn splat(&mut self, sample: &CameraSample, l: &Spectrum) {
        unimplemented!()
    }

    fn get_sample_extent(&self) -> Extent {
        unimplemented!()
    }

    fn get_pixel_extent(&self) -> Extent {
        unimplemented!()
    }

    fn write_image_with_scale(&self, splat_scale: Float) {
        let buf = ImageBuffer::from_fn(self.x_pixel_count as u32, self.y_pixel_count as u32, |x, y| {
            let index = (y * self.x_pixel_count as u32 + x) as usize;
            self.img[index].to_rgb()
        });
        buf.save(&self.path).expect("failed to write image");
    }

    fn resolution(&self) -> (u32, u32) {
        (self.x_pixel_count as u32, self.y_pixel_count as u32)
    }
}
