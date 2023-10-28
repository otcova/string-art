use crate::*;
use image::{imageops, DynamicImage, GrayImage};

pub struct SourceImagePlugin;

impl Plugin for SourceImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, process_source_image);
    }
}

#[derive(Component)]
pub struct ProcessedImage(pub Option<GrayImage>);

fn process_source_image(mut query: Query<(&mut ProcessedImage, &Settings), Changed<Settings>>) {
    for (mut processed_image, settings) in &mut query {
        let image_path = IMAGES_PATHS[settings.source_image_index];

        let mut image = image::open(image_path)
            .unwrap()
            .grayscale()
            .resize_to_fill(
                settings.diameter,
                settings.diameter,
                imageops::FilterType::Triangle,
            )
            .into_luma8();

        for pixel in image.pixels_mut() {
            pixel.0[0] = u8::saturating_sub(pixel.0[0], settings.darken);
        }

        processed_image.0 = Some(image);
    }
}
