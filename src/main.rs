#![allow(unused_imports)]

use std::f32::consts::PI;
use std::mem::swap;
use std::path::Path;
use std::thread;
use std::time::Duration;

use image::{GrayImage, Luma};
use imageproc::*;
use rayon::prelude::*;
use show_image::create_window;

const DIAMETER: u32 = 500; // mm
const NODES: u16 = 200;
const ALPHA: f32 = 0.3;
const MAX_LINES: usize = 6000;
const REPEAT_LINES: bool = false;

const DARKEN: u8 = 100; // 0..256
const CONTRAST: f32 = 0.5; // 0.0..=1.0

struct LineSet([u8; 256 * 256 / 8]);

impl LineSet {
    pub fn new() -> Self {
        Self([0; 256 * 256 / 8])
    }
    pub fn add(&mut self, a: u16, b: u16) {
        if b > a {
            self.add(b, a);
        } else {
            let hash = (a << 8) | b;
            let index = (hash >> 3) as usize;
            self.0[index] = self.0[index] | (1 << (hash & 7));
        }
    }

    pub fn has(&self, a: u16, b: u16) -> bool {
        if b > a {
            self.has(b, a)
        } else {
            let hash = (a << 8) | b;
            let index = (hash >> 3) as usize;
            self.0[index] & (1 << (hash & 7)) != 0
        }
    }
}

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window = create_window("image", Default::default())?;

    // let image = load_image("sample.png");
    // let image = load_image("a.jpb.jpg");
    // let image = load_image("photo.jpg");
    let image = load_image("e.jpg");
    // let target_image = image.clone();

    window.set_image("image", image.clone())?;
    thread::sleep(Duration::from_secs(2));

    let mut canvas = GrayImage::from_pixel(DIAMETER, DIAMETER, Luma([255]));

    let mut node_index = 0;
    let mut traced_nodes = Vec::with_capacity(1024);
    traced_nodes.push(node_index);

    let mut line_set = LineSet::new();

    while let Some(next_node_index) = best_string(node_index, &canvas, &image, &line_set) {
        trace_line(
            node_pos(node_index),
            node_pos(next_node_index),
            |point, alpha| {
                let pixel = canvas.get_pixel_mut(point.0, point.1);
                *pixel = overlay_string(pixel, alpha);
            },
        );

        if !REPEAT_LINES {
            line_set.add(node_index, next_node_index);
        }

        node_index = next_node_index;
        traced_nodes.push(node_index);
        let count = traced_nodes.len();

        if count > 1000 && count % 300 == 0 {
            window.set_image("image", canvas.clone())?;
        }

        if count > MAX_LINES {
            break;
        }
    }

    window.set_image("image", canvas.clone())?;
    println!("Strings: {}", traced_nodes.len() - 1);
    window.wait_until_destroyed()?;

    Ok(())
}

fn best_string(
    node_index: u16,
    canvas: &GrayImage,
    image: &GrayImage,
    line_set: &LineSet,
) -> Option<u16> {
    let origin = node_pos(node_index);

    (0..NODES - 1)
        .into_par_iter()
        .filter_map(|i| {
            let next_idx = if i < node_index { i } else { i + 1 };
            if !REPEAT_LINES && line_set.has(node_index, next_idx) {
                return None;
            }

            let mut performance = 0;
            let mut count = 1;

            trace_line(origin, node_pos(next_idx), |point, alpha| {
                let target = *image.get_pixel(point.0, point.1);
                let pixel = canvas.get_pixel(point.0, point.1);
                let new_pixel = overlay_string(pixel, alpha);

                let pixel_err = color_dist(target, *pixel);
                let new_pixel_err = color_dist(target, new_pixel);

                performance += pixel_err - new_pixel_err;
                count += 1;
            });

            if performance > 0 {
                Some((performance / count, next_idx))
            } else {
                None
            }
        })
        .max_by_key(|(performance, _)| *performance)
        .map(|(_, next_node_index)| next_node_index)
}

fn color_dist(a: Luma<u8>, b: Luma<u8>) -> i32 {
    let r = a.0[0] as i32 - b.0[0] as i32;
    // r.abs()
    r * r
}

fn node_pos(node_index: u16) -> (f32, f32) {
    let angle_step = 2. * PI / NODES as f32;

    let mut angle = angle_step * node_index as f32;
    angle += f32::cos(node_index as f32) * angle_step;

    let (s, c) = f32::sin_cos(angle);
    let r = (DIAMETER - 2) as f32 / 2.;
    (r + r * c, r + r * s)
}

fn load_image(path: &str) -> GrayImage {
    let path = Path::new("assets").join(path);
    let image = image::open(path).unwrap().grayscale().resize_to_fill(
        DIAMETER,
        DIAMETER,
        image::imageops::FilterType::Triangle,
    );

    let mut image = image.into_luma8();

    image = contrast::stretch_contrast(
        &image,
        DARKEN,
        DARKEN + 1 + ((254. - DARKEN as f32) * (1. - CONTRAST)) as u8,
    );
    for pixel in image.pixels_mut() {
        pixel.0[0] = u8::saturating_sub(pixel.0[0], DARKEN);
    }
    image
}

fn overlay_string(pixel: &Luma<u8>, string_alpha: f32) -> Luma<u8> {
    let c = 1. - string_alpha * ALPHA;
    Luma([(pixel.0[0] as f32 * c) as u8])
}

/// Xiaolin Wu’s line algorithm.
fn trace_line<F: FnMut((u32, u32), f32)>(
    mut a: (f32, f32),
    mut b: (f32, f32),
    mut pixel_callback: F,
) {
    let xy_inv = (b.1 - a.1).abs() > (b.0 - a.0).abs();
    if xy_inv {
        a = (a.1, a.0);
        b = (b.1, b.0);
    }

    if a.0 > b.0 {
        swap(&mut a, &mut b);
    }

    let dx = b.0 - a.0;
    let gradient = if dx == 0. { 1. } else { (b.1 - a.1) / dx };

    let mut x = a.0 as u32;
    let mut y = a.1 as f32;

    while x <= b.0 as u32 {
        let y_fract = y - y.floor();

        let y_int = y as u32;
        let point = if xy_inv { (y_int, x) } else { (x, y_int) };
        pixel_callback(point, 1. - y_fract);

        if y_fract <= 0. {
            x += 1;
            y += gradient;
            continue;
        }

        let y_int = y_int + 1;
        let point = if xy_inv { (y_int, x) } else { (x, y_int) };
        pixel_callback(point, y_fract);

        x += 1;
        y += gradient;
    }
}

#[test]
fn tests() {
    let xiaolin_wu = |a: (f32, f32), b: (f32, f32)| {
        let mut data = vec![];
        trace_line((a.0, a.1), (b.0, b.1), |pt, d| data.push((pt, d)));
        data
    };

    assert_eq!(
        xiaolin_wu((0.0, 0.0), (6.0, 3.0)),
        [
            ((0, 0), 1.0),
            ((1, 0), 0.5),
            ((1, 1), 0.5),
            ((2, 1), 1.0),
            ((3, 1), 0.5),
            ((3, 2), 0.5),
            ((4, 2), 1.0),
            ((5, 2), 0.5),
            ((5, 3), 0.5),
            ((6, 3), 1.0)
        ]
    );

    assert_eq!(
        xiaolin_wu((4.0, 2.0), (4.0, 6.0)),
        [
            ((4, 2), 1.0),
            ((4, 3), 1.0),
            ((4, 4), 1.0),
            ((4, 5), 1.0),
            ((4, 6), 1.0),
        ]
    );

    assert_eq!(
        xiaolin_wu((2.0, 4.0), (6.0, 4.0)),
        [
            ((2, 4), 1.0),
            ((3, 4), 1.0),
            ((4, 4), 1.0),
            ((5, 4), 1.0),
            ((6, 4), 1.0),
        ]
    );

    // The algorithm reorders the points to be left-to-right

    assert_eq!(
        xiaolin_wu((340.5, 290.77), (110.0, 170.0)),
        xiaolin_wu((110.0, 170.0), (340.5, 290.77))
    );
}
