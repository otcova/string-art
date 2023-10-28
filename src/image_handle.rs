use bevy::prelude::*;

pub struct MutImage<'a> {
    pub data: &'a mut Vec<u8>,
    pub width: u32,
    pub heigth: u32,
}

impl<'a> MutImage<'a> {
    pub fn from_handle(handle: &Handle<Image>, assets: &'a mut Assets<Image>) -> Option<Self> {
        assets.get_mut(&handle).map(|raw_image| Self {
            width: raw_image.texture_descriptor.size.width,
            heigth: raw_image.texture_descriptor.size.height,
            data: &mut raw_image.data,
        })
    }

    fn pixel_index(&self, x: u32, y: u32) -> usize {
        (x + y * self.width) as usize
    }

    pub fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut u8 {
        let index = self.pixel_index(x, y);
        &mut self.data[index]
    }
}

pub struct RefImage<'a> {
    pub data: &'a Vec<u8>,
    pub width: u32,
    pub heigth: u32,
}

impl<'a> RefImage<'a> {
    pub fn from_handle(handle: &Handle<Image>, assets: &'a Assets<Image>) -> Option<Self> {
        assets.get(&handle).map(|raw_image| Self {
            width: raw_image.texture_descriptor.size.width,
            heigth: raw_image.texture_descriptor.size.height,
            data: &raw_image.data,
        })
    }

    fn pixel_index(&self, x: u32, y: u32) -> usize {
        (x + y * self.width) as usize
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> u8 {
        self.data[self.pixel_index(x, y)]
    }
}
