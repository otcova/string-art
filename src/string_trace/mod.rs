mod line;

use crate::*;
use bevy::render::render_resource::*;
use image::{GrayImage, Luma};
use line::*;
use rayon::prelude::*;
use std::f32::consts::PI;

pub struct StringTracePlugin;

impl Plugin for StringTracePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_string_trace, trace_string));
    }
}

#[derive(Component)]
pub struct StringTrace {
    pub traced_nodes: Vec<u16>,
    pub canvas: Handle<Image>,
    pub line_set: LineSet,
    pub done: bool,
}

fn update_string_trace(
    mut query: Query<(&mut StringTrace, &mut UiImage, &Settings), Changed<ProcessedImage>>,
    mut assets: ResMut<Assets<Image>>,
) {
    for (mut trace, mut image, settings) in &mut query {
        *trace = StringTrace::new(settings, &mut assets);
        image.texture = trace.canvas.clone();
    }
}

fn trace_string(
    mut query: Query<(&mut StringTrace, &ProcessedImage, &Settings)>,
    mut assets: ResMut<Assets<Image>>,
) {
    for (mut trace, target_image, settings) in &mut query {
        // if tracker
        if let Some(target_image) = &target_image.0 {
            if target_image.width() == settings.diameter {
                for _ in 0..128 {
                    if trace.done {
                        break;
                    }

                    trace.trace_best(target_image, settings, &mut assets);
                }
            }
        }
    }
}

impl StringTrace {
    pub fn new(settings: &Settings, assets: &mut Assets<Image>) -> Self {
        let image = Image::new_fill(
            Extent3d {
                width: settings.diameter,
                height: settings.diameter,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[255],
            TextureFormat::R8Unorm,
        );
        Self {
            traced_nodes: vec![0],
            // canvas: GrayImage::from_pixel(settings.diameter, settings.diameter, Luma([255])),
            canvas: assets.add(image),
            line_set: LineSet::new(),
            done: false,
        }
    }

    fn trace_best(
        &mut self,
        target_image: &GrayImage,
        settings: &Settings,
        assets: &mut Assets<Image>,
    ) {
        let Some(next_node_index) = self.best_next_string(target_image, settings, &assets) else {
            self.done = true;
            return;
        };

        let node_index = *self.traced_nodes.last().unwrap();
        let mut canvas = MutImage::from_handle(&self.canvas, assets).unwrap();
        trace_line(
            settings.node_pos(node_index),
            settings.node_pos(next_node_index),
            |point, alpha| {
                let pixel = canvas.get_pixel_mut(point.0, point.1);
                *pixel = overlay_string(*pixel, alpha, settings);
            },
        );

        if !settings.repeat_lines {
            self.line_set.add(node_index, next_node_index);
        }

        self.traced_nodes.push(next_node_index);

        if self.traced_nodes.len() > settings.max_lines {
            self.done = true;
        }
    }

    fn best_next_string(
        &self,
        image: &GrayImage,
        settings: &Settings,
        assets: &Assets<Image>,
    ) -> Option<u16> {
        let node_index = *self.traced_nodes.last().unwrap();
        let origin = settings.node_pos(node_index);
        let canvas = RefImage::from_handle(&self.canvas, assets).unwrap();

        (0..settings.nodes - 1)
            .into_par_iter()
            .filter_map(|i| {
                let next_idx = if i < node_index { i } else { i + 1 };

                if !settings.repeat_lines && self.line_set.has(node_index, next_idx) {
                    return None;
                }

                let mut performance = 0;
                let mut count = 1;

                trace_line(origin, settings.node_pos(next_idx), |point, alpha| {
                    let target = *image.get_pixel(point.0, point.1);
                    let pixel = canvas.get_pixel(point.0, point.1);
                    let new_pixel = overlay_string(pixel, alpha, settings);

                    let pixel_err = settings.color_dist(target, Luma([pixel]));
                    let new_pixel_err = settings.color_dist(target, Luma([new_pixel]));

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
}

impl Settings {
    fn color_dist(&self, a: Luma<u8>, b: Luma<u8>) -> i32 {
        let r = a.0[0] as i32 - b.0[0] as i32;
        match &self.color_distance_fn {
            ColorDistanceFn::ABS => r.abs(),
            ColorDistanceFn::SQUARE => r * r,
        }
    }

    fn node_pos(&self, node_index: u16) -> (f32, f32) {
        let angle_step = 2. * PI / self.nodes as f32;

        let mut angle = angle_step * node_index as f32;
        angle += f32::cos(node_index as f32) * angle_step * self.nodes_offset;

        let (s, c) = f32::sin_cos(angle);
        let r = (self.diameter - 2) as f32 / 2.;
        (r + r * c, r + r * s)
    }
}
