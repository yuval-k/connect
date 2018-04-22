use std;
use super::pixels;
use kiss3d;
use serde_json;
use nalgebra;

use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::scene::SceneNode;

#[derive(Copy,Clone,Debug, Deserialize)]
struct Point {
    point: [f32; 3],
}

#[derive(Copy,Clone,Debug)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}
struct GuiLedArray {
    pixels: Vec<Pixel>,
    tx: std::sync::mpsc::Sender<Vec<Pixel>>,
}

impl GuiLedArray {
    fn new(size: usize, tx: std::sync::mpsc::Sender<Vec<Pixel>>) -> Self {
        let p = Pixel { r: 0, g: 0, b: 0 };
        GuiLedArray {
            pixels: vec![p; size],
            tx: tx,
        }
    }
}


impl pixels::LedArray for GuiLedArray {
    fn len(&self) -> usize {
        self.pixels.len()
    }

    fn set_color_rgba(&mut self, lednum: usize, r: u8, g: u8, b: u8, a: u8) {
        self.pixels[lednum].r = r;
        self.pixels[lednum].g = g;
        self.pixels[lednum].b = b;
    }
    fn show(&mut self) -> std::io::Result<()> {
        self.tx.send(self.pixels.clone()).unwrap();
        Ok(())
    }
}

pub struct UI{
    rx: std::sync::mpsc::Receiver<Vec<Pixel>>,
    window : Window,
    cubes: Vec<SceneNode>,
}

impl UI{
    fn new(rx: std::sync::mpsc::Receiver<Vec<Pixel>>) -> Self {

    let file = std::fs::File::open(std::env::var("LAYOUT").unwrap()).unwrap();
    let pts: Vec<Point> = serde_json::from_reader(file).unwrap();

    let mut window = Window::new("Kiss3d: cube");
    let mut cubes: Vec<SceneNode> = vec![];
    for p in pts {
        let mut cube = window.add_cube(0.03, 0.03, 0.03);
        cube.set_color(0.0, 0.0, 0.0);
        let translate: nalgebra::Translation3<f32> =
            nalgebra::Translation3::new(p.point[0], p.point[1], p.point[2]);
        cube.set_local_translation(translate);
        cubes.push(cube);
    }


    window.set_light(Light::StickToCamera);
    UI {
        rx:rx,
        window:window,
         cubes:  cubes,
    }

    }
pub fn start_ui(&mut self) {

    while self.window.render() {
        if let Ok(v) = self.rx.try_recv() {
            for (pixel, cube) in v.iter().zip(self.cubes.iter_mut()) {
                cube.set_color(pixel.r as f32 / 255.0,
                               pixel.g as f32 / 255.0,
                               pixel.b as f32 / 255.0);
            }
        }
    }
}

}

pub fn create_gui() -> (Option<Box<pixels::LedArray+Send>> , Option<UI>){
    let (tx, rx) = std::sync::mpsc::channel();
    (Some(Box::new(GuiLedArray::new(super::NUM_POLES * super::LEDS_PER_STRING, tx))), Some(UI::new(rx)))
}

