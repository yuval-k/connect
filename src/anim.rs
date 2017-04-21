use std;
use num::Float;
use palette::IntoColor;

pub trait LedArray {
    fn len(&self) -> usize;
    fn set_color_rgba(&mut self, lednum: usize, r: u8, g: u8, b: u8, a: u8);
    fn show(&mut self) -> std::io::Result<()>;
}


pub fn set_color<U: Float, T: IntoColor<U>>(l: &mut LedArray, lednum: usize, color: T) {

    let rgb = color.into_rgb();
    let (r, g, b, a): (u8, u8, u8, u8) = rgb.to_pixel();
    l.set_color_rgba(lednum, r, g, b, a);
}


pub trait Animation {
    fn animate(&mut self, array: &mut LedArray, delta: std::time::Duration);
}
