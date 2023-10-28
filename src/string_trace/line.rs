use super::*;

pub struct LineSet([u8; 256 * 256 / 8]);

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

pub fn overlay_string(pixel: u8, string_alpha: f32, settings: &Settings) -> u8 {
    let c = 1. - string_alpha * settings.string_alpha;
    (pixel as f32 * c) as u8
}

/// Xiaolin Wu’s line algorithm.
pub fn trace_line<F: FnMut((u32, u32), f32)>(
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
        std::mem::swap(&mut a, &mut b);
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
