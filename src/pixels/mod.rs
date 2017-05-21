use std;
use num::Float;
use palette::IntoColor;

pub struct Led {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub trait LedArray {
    fn len(&self) -> usize;
    fn set_color_rgba(&mut self, lednum: usize, r: u8, g: u8, b: u8, a: u8);
    fn show(&mut self) -> std::io::Result<()>;
}

pub struct PixelArray<T:LedArray> {
    l : T,
}

impl<T:LedArray> PixelArray<T> {

pub fn set_color<U: Float, C: IntoColor<U>>(&mut self, lednum: usize, color: C) {
    let rgb = color.into_rgb();
    let (r, g, b, a): (u8, u8, u8, u8) = rgb.to_pixel();
    self.l.set_color_rgba(lednum, r, g, b, a);
}


}


#[derive(Copy,Clone,Debug)]
pub enum RgbOrder {
    Rgb,
    Rbg,
    Gbr,
    Grb,
    Bgr,
    Brg,
}

impl RgbOrder {
    pub fn new(rgb: &str) -> Result<RgbOrder, ()> {
        if rgb == "rgb" {
            return Ok(RgbOrder::Rgb);
        }
        if rgb == "rbg" {
            return Ok(RgbOrder::Rbg);
        }
        if rgb == "bgr" {
            return Ok(RgbOrder::Bgr);
        }
        if rgb == "brg" {
            return Ok(RgbOrder::Brg);
        }
        if rgb == "gbr" {
            return Ok(RgbOrder::Gbr);
        }
        if rgb == "grb" {
            return Ok(RgbOrder::Grb);
        }
        Err(())
    }

    pub fn transform(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        match *self {
            RgbOrder::Rgb => (r, g, b),
            RgbOrder::Rbg => (r, b, g),
            RgbOrder::Gbr => (g, b, r),
            RgbOrder::Grb => (g, r, b),
            RgbOrder::Bgr => (b, g, r),
            RgbOrder::Brg => (b, r, g),
        }
    }
}

pub struct RgbLedArray<T: AsRef<LedArray> + AsMut<LedArray>> {
    leds: T,
    rgb: RgbOrder,
}
impl<T: AsRef<LedArray> + AsMut<LedArray>> RgbLedArray<T> {
    pub fn new(leds: T, rgb: RgbOrder) -> Self {
        RgbLedArray {
            leds: leds,
            rgb: rgb,
        }
    }
}
impl<T: AsRef<LedArray> + AsMut<LedArray>> LedArray for RgbLedArray<T> {
    fn len(&self) -> usize {
        self.leds.as_ref().len()
    }

    fn set_color_rgba(&mut self, lednum: usize, r: u8, g: u8, b: u8, a: u8) {
        let (r, g, b) = self.rgb.transform(r, g, b);
        self.leds.as_mut().set_color_rgba(lednum, r, g, b, a)
    }

    fn show(&mut self) -> std::io::Result<()> {
        self.leds.as_mut().show()
    }
}


pub fn set_color<U: Float, T: IntoColor<U>>(l: &mut LedArray, lednum: usize, color: T) {
    let rgb = color.into_rgb();
    let (r, g, b, a): (u8, u8, u8, u8) = rgb.to_pixel();
    l.set_color_rgba(lednum, r, g, b, a);
}
