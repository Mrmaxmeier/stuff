#![feature(test)]
extern crate test;
extern crate num;
extern crate image;
extern crate rand;

use std::fs::File;
use std::path::Path;

use rand::{thread_rng, Rng};


pub struct PointGen {
    max_iterations: usize,
    min_iterations: usize,
}

#[inline(always)]
fn maybe_outside(x: f64, y: f64) -> bool {
    // returns true if the point if possibly outside the mandelbrot set
    // returns false if the point ist guaranteed to be inside the set
    let q = y * y + (x - 0.25) * (x - 0.25);

    q * (q + x - 0.25) > 0.25 * y * y && // point outside the main caridoid
	(1.0 + x) * (1.0 + x) + y * y > 1.0 / 16.0 // point outside the period-2-bulb
}

impl Iterator for PointGen {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<(f64, f64)> {
        let offset = 0.5;
        loop {
            let (x, y) = thread_rng().gen::<(f64, f64)>();
            let x = x * 2.0 - 1.0 - offset;
            let y = y * 2.0 - 1.0;
            if !maybe_outside(x, y) {
                continue;
            }
            let mut xtemp = x;
            let mut ytemp = y;
            for iter in 1..self.max_iterations {
                let xtem = xtemp * xtemp - ytemp * ytemp + x;
                ytemp = 2f64 * xtemp * ytemp + y;
                xtemp = xtem;
                if xtemp * xtemp + ytemp * ytemp >= 4f64 && iter >= self.min_iterations {
                    return Some((x + offset, y));
                }
            }
        }
    }
}


fn main() {
    let imgx = 800;
    let imgy = 800;

    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    let max_points = std::u16::MAX as usize;
    let point_generator = PointGen {
        max_iterations: 10_000,
        min_iterations: 50,
    };
    for (i, (x, y)) in point_generator.take(max_points).enumerate() {
        let x = x / 2.0 + 0.5;
        let y = y / 2.0 + 0.5;
        let ix = (x * imgx as f64) as u32;
        let iy = (y * imgy as f64) as u32;
        // let pixel: image::Luma<u8> = imgbuf[(ix, iy)];
        // imgbuf.put_pixel(ix, iy, image::Luma([pixel.data[0] as u8 + 100u8]));
        imgbuf.put_pixel(ix, iy, image::Luma([255u8]));
        if i % 1000 == 0 || i == max_points {
            println!("{} / {}", i, max_points);
        }
    }

    // for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
    // pixel = image::Luma([(x - y) as u8]);
    // }
    //


    let mut fout = File::create(&Path::new("fractal.png")).unwrap();

    let _ = image::ImageLuma8(imgbuf).save(&mut fout, image::PNG);
}



#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use rand::{thread_rng, Rng};

    #[bench]
    fn bench_05_100(b: &mut Bencher) {
        let mut pg = PointGen {
            min_iterations: 5,
            max_iterations: 100,
        };
        b.iter(|| pg.next());
    }

    #[bench]
    fn bench_15_300(b: &mut Bencher) {
        let mut pg = PointGen {
            min_iterations: 15,
            max_iterations: 300,
        };
        b.iter(|| pg.next());
    }

    #[bench]
    fn bench_50_1000(b: &mut Bencher) {
        let mut pg = PointGen {
            min_iterations: 50,
            max_iterations: 1000,
        };
        b.iter(|| pg.next());
    }

    #[bench]
    fn bench_rng_floats(b: &mut Bencher) {
        b.iter(|| thread_rng().gen::<(f64, f64)>());
    }

    #[bench]
    fn bench_rng_floats2(b: &mut Bencher) {
        let mut rng = thread_rng();
        b.iter(|| (rng.next_f64(), rng.next_f64()));
    }
}
