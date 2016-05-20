#![feature(test)]
extern crate test;
extern crate num;
extern crate image;
extern crate rand;

use std::fs::File;
use std::path::Path;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

use rand::{thread_rng, Rng};

pub type Point = (f64, f64);

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
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        let offset = 0.5;
        loop {
            let (x, y) = thread_rng().gen::<Point>();
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


pub struct ThreadedPointGen {
    rx: Receiver<Point>,
    count: usize,
    max_count: usize,
}

fn threaded_point_gen(max_iterations: usize,
                      min_iterations: usize,
                      thread_count: usize,
                      point_amount: usize)
                      -> ThreadedPointGen {
    let (tx, rx): (Sender<Point>, Receiver<Point>) = mpsc::channel();

    for thread_index in 0..thread_count {
        let thread_tx = tx.clone();
        thread::spawn(move || {
            let amount = match thread_index {
                0 => point_amount - (point_amount / thread_count) * (thread_count - 1),
                _ => point_amount / thread_count,
            };
            let point_generator = PointGen {
                max_iterations: max_iterations,
                min_iterations: min_iterations,
            };
            for point in point_generator.take(amount) {
                thread_tx.send(point).unwrap();
            }
        });
    }
    ThreadedPointGen {
        rx: rx,
        count: 0,
        max_count: point_amount,
    }
}


impl Iterator for ThreadedPointGen {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.count == self.max_count {
            None
        } else {
            self.count += 1;
            Some(self.rx.recv().unwrap())
        }
    }
}

fn main() {
    let imgx = 1024 * 4;
    let imgy = 1024 * 4;

    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    let point_amount = std::u16::MAX as usize * 4;
    let max_iterations = 10_000;
    let min_iterations = 50;
    let threads = 8;
    let point_generator = threaded_point_gen(max_iterations, min_iterations, threads, point_amount);

    for (i, (x, y)) in point_generator.enumerate() {
        let x = x / 2.0 + 0.5;
        let y = y / 2.0 + 0.5;
        let ix = (x * imgx as f64) as u32;
        let iy = (y * imgy as f64) as u32;
        let pixel: image::Luma<u8> = imgbuf[(ix, iy)];
        imgbuf.put_pixel(ix, iy, image::Luma([pixel.data[0] + 1u8]));
        let e = i + 1;
        if e % 1000 == 0 || e == point_amount {
            println!("{} / {}", e, point_amount);
        }

        if e % 50_000 == 0 || e == point_amount {
            let path = &*format!("images/fractal_{}.png", e / 10_000);
            println!("saving image '{}'", path);
            let orig = imgbuf.clone();
            let mut imgbuf = orig.clone();
            for (_, _, pixel) in imgbuf.enumerate_pixels_mut() {
                let val = pixel.data[0];
                if val == 0 {
                    continue;
                }
                let l = val as usize * 256 / (e / 10_000);
                // println!("{} = {} * 256 / {}", l, val, (e / 10_000));
                if l > std::u8::MAX as usize {
                    *pixel = image::Luma([std::u8::MAX]);
                } else {
                    *pixel = image::Luma([l as u8]);
                }
            }
            let mut glowcopy = imgbuf.clone();
            for (x, y, pixel) in glowcopy.enumerate_pixels_mut() {
                let glow_amount = 1;
                if x <= glow_amount || x + glow_amount >= imgx {
                    continue;
                }
                if y <= glow_amount || y + glow_amount >= imgy {
                    continue;
                }
                let mut glow = 0.0;
                for ox in num::iter::range_step(x - glow_amount, x + glow_amount, 1) {
                    for oy in num::iter::range_step(y - glow_amount, y + glow_amount, 1) {
                        let val = orig[(ox, oy)].data[0];
                        glow += val as f64;
                    }
                }
                let glow = (glow * 20.0) as u8;
                let val = pixel.data[0];
                if (val as usize + glow as usize) < std::u8::MAX as usize {
                    *pixel = image::Luma([val + glow]);
                }
            }
            let mut fout = File::create(&Path::new(path)).unwrap();
            let _ = image::ImageLuma8(glowcopy).save(&mut fout, image::PNG);
        }
    }
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
        b.iter(|| thread_rng().gen::<Point>());
    }

    #[bench]
    fn bench_rng_floats2(b: &mut Bencher) {
        let mut rng = thread_rng();
        b.iter(|| (rng.next_f64(), rng.next_f64()));
    }
}
