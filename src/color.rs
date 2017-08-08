extern crate image;

use vector::Vector3;
use image::{Rgb, Rgba};
use math::saturate;

pub type Color = Vector3;

pub fn color_to_rgb(color: Color) -> Rgb<u8> {
    image::Rgb([
       (255.0 * saturate(color.x)) as u8,
       (255.0 * saturate(color.y)) as u8,
       (255.0 * saturate(color.z)) as u8,
    ])
}

pub fn rgba_to_color(color: Rgba<u8>) -> Color {
    Color::new(
        color.data[0] as f64 / 255.0,
        color.data[1] as f64 / 255.0,
        color.data[2] as f64 / 255.0,
    )
}
