extern crate libc;
extern crate num;
extern crate palette;
extern crate tk_opc;
extern crate bit_set;
extern crate rosc;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serial;
#[macro_use]
extern crate clap;

#[macro_use]
extern crate bitflags;

use std::sync::mpsc;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
mod ledscape;

mod pixels;
mod animations;
mod opc;
mod osc;
mod events;
mod config;

#[cfg(feature = "gui")]
extern crate kiss3d;
#[cfg(feature = "gui")]
extern crate serde;
#[cfg(feature = "gui")]
extern crate serde_json;
#[cfg(feature = "gui")]
extern crate nalgebra;
#[cfg(feature = "gui")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "gui")]
mod gui;
#[cfg(feature = "gui")]
fn create_gui() -> Option<Box<pixels::LedArray>> {
    gui::create_gui()
}
#[cfg(not(feature = "gui"))]
fn create_gui() -> Option<Box<pixels::LedArray>> {
    None
}

use animations::Drawer;

#[derive(Clone,Copy,Debug)]
pub enum EventTypes {
    Connect(usize, usize),
}

#[derive(Clone,Copy,Debug)]
pub enum Events {
    Start(EventTypes),
    Stop(EventTypes),
    Reset,
    Draw,
    ConfigChanged,
    ModeChanged(Modes),
}


#[derive(Clone,Copy,Debug)]
pub enum Modes {
    Regular,
    Disco,
    Flower,
}

/// touch goes up to cp1 and twinkels / breathes like the heart, the hight it is the higher the lum.

const LEDS_PER_STRING: usize = 100;
const NUM_POLES: usize = 20;

struct PoleLedArrayAdapter<'a> {
    ls: &'a mut pixels::LedArray,
    pole_offset: usize,
    size: usize,
}


impl<'a> PoleLedArrayAdapter<'a> {
    fn new(ls: &'a mut pixels::LedArray, pole_strip_size: usize, pole_strip_index: usize) -> Self {
        PoleLedArrayAdapter {
            ls: ls,
            pole_offset: pole_strip_size * pole_strip_index,
            size: pole_strip_size,
        }
    }
}

impl<'a> pixels::LedArray for PoleLedArrayAdapter<'a> {
    fn len(&self) -> usize {
        self.size
    }

    fn set_color_rgba(&mut self, lednum: usize, r: u8, g: u8, b: u8, a: u8) {
        self.ls.set_color_rgba(self.pole_offset + lednum, r, g, b, a);
    }

    fn show(&mut self) -> std::io::Result<()> {
        // nothing here..
        Ok(())
    }
}


#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn get_led_array() -> Box<pixels::LedArray> {
    use pixels::LedArray;
    let mut l = ledscape::LedscapeLedArray::new(LEDS_PER_STRING);
    for i in 0..l.len() {
        l.set_color_rgba(i, 255, 0, 0, 255);
    }
    l.show();
    std::thread::sleep_ms(1000);
    for i in 0..l.len() {
        l.set_color_rgba(i, 0, 255, 0, 255);
    }
    l.show();
    std::thread::sleep_ms(1000);
    for i in 0..l.len() {
        l.set_color_rgba(i, 0, 0, 255, 255);
    }
    l.show();
    std::thread::sleep_ms(1000);
    Box::new(l)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn get_led_array() -> Box<pixels::LedArray> {
    match create_gui() {
        Some(l) => l,
        None => Box::new(get_opc_array("127.0.0.1:7890").expect("can't connect")),
    }
}

fn get_opc_array(adrr: &str) -> std::io::Result<opc::OPCLedArray> {
    Ok(opc::OPCLedArray::new(LEDS_PER_STRING * NUM_POLES, adrr))
}


fn main() {

    let matches = clap::App::new("connect server")
        .about("connects people")
       // use crate_version! to pull the version number
       .version(crate_version!()).arg(clap::Arg::with_name("device")
                                    .short("d")
                                    .long("device")
                                    .value_name("FILE")
                                    .help("The device to program")
                                    .takes_value(true))
                                  .arg(clap::Arg::with_name("osc_server")
                                    .short("o")
                                    .long("osc_server")
                                    .value_name("OSC_SERVER")
                                    .help("The open sound ccontrol server to send osc signals to")
                                    .takes_value(true))
                                .arg(clap::Arg::with_name("opc_server")
                                    .short("p")
                                    .long("opc_server")
                                    .value_name("OPC_SERVER")
                                    .help("The open pixel control to send osc signals to")
                                    .takes_value(true))
                                .arg(clap::Arg::with_name("rgb")
                                    .long("rgb")
                                    .value_name("RGB")
                                    .help("RGB order")
                                    .takes_value(true))
                               .get_matches();

    let device = matches.value_of("device")
        .map(|s| s.to_string())
        .unwrap_or(std::env::var("DEVICE").unwrap_or(String::new()));
    let osc_server = matches.value_of("osc_server")
        .map(|s| s.to_string())
        .unwrap_or(std::env::var("OSC_SERVER").unwrap_or(String::new()));
    let opc_server = matches.value_of("opc_server")
        .map(|s| s.to_string())
        .unwrap_or(std::env::var("OPC_SERVER").unwrap_or(String::new()));
    let rgb = matches.value_of("rgb")
        .map(|s| pixels::RgbOrder::new(s).expect("Invalid rgb value!"))
        .unwrap_or(pixels::RgbOrder::Rgb);


    env_logger::init().unwrap();

    info!("hello");
    let ledscapecontroller: Box<pixels::LedArray> = if opc_server.is_empty() {
        get_led_array()
    } else {
        Box::new(get_opc_array(&opc_server).expect("can't connect"))
    };

    let mut ledscapecontroller = Box::new(pixels::RgbLedArray::new(ledscapecontroller, rgb));

    // TODO add OPCCLient

    let poles: Vec<Pole> = (0..NUM_POLES)
        .map(|x| 2_f32 * std::f32::consts::PI * (x as f32) / (NUM_POLES as f32))
        .map(|x| Pole::new(x))
        .collect();

    let (tx, rx) = mpsc::channel();

    // 20 fps
    let fps_duration = std::time::Duration::from_secs(1) / 20;
    let newtx = tx.clone();
    std::thread::spawn(move || {
        let tx = newtx;
        loop {
            tx.send(Events::Draw).expect("drawing event failed to send!");
            std::thread::sleep(fps_duration);
        }
    });

    let eventer = events::get_eventer(&device);
    let timeout = eventer.get_timeout();

    let newtx = tx.clone();
    std::thread::spawn(move || {
        let tx = newtx;
        let mut eventer = eventer;
        eventer.get_events(tx);
        panic!("event loop should be endless")
    });

    let animator = animations::Animator::new(osc::OSCManager::new(&osc_server));
    let config = config::Config::new(tx.clone());

    work(config,
         move |poles| draw_poles_to_array(ledscapecontroller.as_mut(), poles),
         poles,
         timeout,
         animator,
         rx);

    println!("Hello, world!");
}

#[derive(Copy,Clone,Debug)]
pub struct TouchState {
    first: std::time::Instant,
    last: std::time::Instant,
}

impl TouchState {
    fn new() -> Self {
        let n = std::time::Instant::now();
        TouchState {
            first: n,
            last: n,
        }
    }
    fn update(&mut self) {
        self.last = std::time::Instant::now();
    }
}
pub struct TouchMap {
    pub touches: [[Option<TouchState>; NUM_POLES]; NUM_POLES],
    timeout_period: std::time::Duration,
}

impl TouchMap {
    pub fn new(timeout: std::time::Duration) -> Self {
        TouchMap {
            touches: [[None; NUM_POLES]; NUM_POLES],
            timeout_period: timeout,
        }
    }

    pub fn clean_timeout(&mut self) {

        let now = std::time::Instant::now();
        for tmp in self.touches.iter_mut() {
            for t in tmp.iter_mut() {
                if let Some(oldinst) = *t {
                    if (now - oldinst.last) > self.timeout_period {
                        *t = None;
                    }
                }
            }
        }
    }

    pub fn connect(&mut self, pole1: usize, pole2: usize) {
        //     let (pole1, pole2) = Self::normalize(pole1, pole2);

        let mut newtouch = self.touches[pole1][pole2].unwrap_or(TouchState::new());
        newtouch.update();
        self.touches[pole1][pole2] = Some(newtouch);
        self.touches[pole2][pole1] = Some(newtouch);
    }

    pub fn disconnect(&mut self, pole1: usize, pole2: usize) {
        // let (pole1, pole2) = Self::normalize(pole1, pole2);
        self.touches[pole1][pole2] = None;
        self.touches[pole2][pole1] = None;
    }
    pub fn reset(&mut self) {
        for tmp in self.touches.iter_mut() {
            for t in tmp.iter_mut() {
                *t = None;
            }
        }
    }
}

fn work<F>(config: config::Config,
           mut draw_poles: F,
           mut poles: Vec<Pole>,
           timeout: std::time::Duration,
           mut animator: animations::Animator,
           receiver: mpsc::Receiver<Events>)
    where F: FnMut(&mut [Pole])
{

    let mut touches = TouchMap::new(timeout);

    let mut mode = Modes::Regular;
    let mut last_anim_time = std::time::Instant::now();
    for event in receiver.into_iter() {
        match event {
            Events::Start(EventTypes::Connect(pole1, pole2)) => {
                touches.connect(pole1, pole2);
            }
            Events::Stop(EventTypes::Connect(pole1, pole2)) => {
                touches.disconnect(pole1, pole2);
            }
            Events::Reset => {
                touches.reset();
            }
            Events::Draw => {
                touches.clean_timeout();
                let now = std::time::Instant::now();

                let delta = now - last_anim_time;
                match mode {
                    Modes::Regular => {
                        // darkness
                        clear(&mut poles);
                        animator.animate_poles(&mut poles, &touches, delta);
                    }
                    Modes::Flower => {
                        animator.animate_flower(&mut poles, delta);
                    }
                    Modes::Disco => {
                        animator.animate_disco(&mut poles, delta);
                    }
                }

                animator.animate_hearts(&mut poles, delta);

                draw_poles(&mut poles);

                last_anim_time = now;
            }
            Events::ConfigChanged => {

                let polen = config.get_num_leds_for_pole();
                let cp1 = config.get_cp1();
                let cp2 = config.get_cp2();
                let heart = config.get_heart();
                for p in poles.iter_mut() {
                    p.set_pole_length(polen);
                    p.set_cp1(cp1);
                    p.set_cp2(cp2);
                    p.set_heart(&heart);
                }
            }
            Events::ModeChanged(newmode) => {

                clear(&mut poles);
                mode = newmode;
            }            
        }
    }
}

fn clear(mut poles: &mut [Pole]) {

    let black = palette::Hsl::new(palette::RgbHue::from_radians(0.), 0., 0.);
    for p in poles.iter_mut() {
        for pixel in p.internal_leds.iter_mut() {
            *pixel = black;
        }
    }
}


#[derive(Clone,Debug,PartialEq)]
pub enum PoleState {
    NotTouched,
    Touched,
    ConnectedTo(bit_set::BitSet),
}

#[derive(Copy, Clone,Debug, PartialEq)]
pub enum PoleAnimations {
    Touching,
    Connecting,
    Exploding,
}

#[derive(Clone,Debug)]
pub struct Pole {
    pub level: f32,
    pub touch_level: f32,
    pub base_color: palette::Hsl,
    pub current_color: palette::Hsl,
    internal_leds: Vec<palette::Hsl>,

    pub state: PoleState,
    pub anim: Option<PoleAnimations>,

    pole_length: usize,
    cp1: usize,
    cp2: usize,
    heart: std::ops::Range<usize>,
}

impl Pole {
    pub fn leds(&mut self) -> &mut [palette::Hsl] {
        &mut self.internal_leds[..self.pole_length]
    }

    pub fn heart(&mut self) -> &mut [palette::Hsl] {
        &mut self.internal_leds[self.heart.clone()]
    }

    pub fn leds_cp1(&mut self) -> &mut [palette::Hsl] {
        &mut self.internal_leds[..self.cp1]
    }

    pub fn leds_cp2(&mut self) -> &mut [palette::Hsl] {
        &mut self.internal_leds[..self.cp2]
    }

    pub fn set_pole_length(&mut self, newl: usize) {
        self.pole_length = std::cmp::min(self.internal_leds.len(), newl);
    }
    pub fn set_cp1(&mut self, newl: usize) {
        self.cp1 = std::cmp::min(self.internal_leds.len(), newl);
    }
    pub fn set_cp2(&mut self, newl: usize) {
        self.cp2 = std::cmp::min(self.internal_leds.len(), newl);
    }
    pub fn set_heart(&mut self, newl: &std::ops::Range<usize>) {
        self.heart = std::cmp::max(0, newl.start)..std::cmp::min(self.internal_leds.len(), newl.end);
    }
    fn new(rads: f32) -> Self {
        Pole {
            level: 0.,
            touch_level: 0.,
            internal_leds:
                vec![palette::Hsl::new(palette::RgbHue::from_radians(0.),1.,0.5); LEDS_PER_STRING],
            //            pole_state : PoleState::Untouched,
            base_color: palette::Hsl::new(palette::RgbHue::from_radians(rads), 1., 0.5),

            current_color: palette::Hsl::new(palette::RgbHue::from_radians(1.), 1., 0.5),

            anim: None,
            state: PoleState::NotTouched,
            pole_length: LEDS_PER_STRING,
            cp1: LEDS_PER_STRING,
            cp2: LEDS_PER_STRING,
            heart: 0..0,
        }
    }
}

fn draw_poles_to_array(c: &mut pixels::LedArray, poles: &[Pole]) {
    for (i, pole) in poles.iter().enumerate() {
        // with ledscape, anim array  is a big array. each LEDS_PER_STRING are one pole.
        let mut adaper = PoleLedArrayAdapter::new(c, LEDS_PER_STRING, i);
        pole.draw(&mut adaper);
    }
    if let Err(e) = c.show() {
        error!("Error showing leds {:?} ", e);
    }
}

impl animations::Drawer for Pole {
    fn draw(&self, array: &mut pixels::LedArray) {
        // ?!
        for (i, c) in (0..array.len()).zip(self.internal_leds.iter()) {
            pixels::set_color(array, i, *c);
        }
    }
}

impl animations::Animation for Pole {
    fn update_animation(&mut self, delta: std::time::Duration) {
        // ?!
        let size = self.leds().len() as f32;
        for (i, pixel) in self.leds().iter_mut().enumerate() {
            let newhue: f32 = pixel.hue.to_radians() + delta.as_secs() as f32 +
                              delta.subsec_nanos() as f32 / 1_000_000_000.0;
            pixel.hue = palette::RgbHue::from_radians(newhue);
            let factor: f32 = i as f32 / size;
            pixel.lightness = 0.5 * factor;
        }
    }
}
