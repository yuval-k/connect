extern crate libc;
extern crate num;
extern crate palette;
extern crate tk_opc;

use std::sync::mpsc;
use std::io::BufRead;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
mod ledscape;

mod anim;
mod animations;
mod opc;

use anim::Animation;
use anim::Drawer;

#[derive(Clone,Copy,Debug)]
enum EventTypes {
    Touch(usize),
    Connect(usize, usize),
}

#[derive(Clone,Copy,Debug)]
enum Events {
    Start(EventTypes),
    Stop(EventTypes),
    Draw,
}

const LEDS_PER_STRING: usize = 100;
const NUM_POLES: usize = 20;

struct PoleLedArrayAdapter<'a> {
    ls: &'a mut anim::LedArray,
    pole_offset: usize,
    size: usize,
}


impl<'a> PoleLedArrayAdapter<'a> {
    fn new(ls: &'a mut anim::LedArray, pole_strip_size: usize, pole_strip_index: usize) -> Self {
        PoleLedArrayAdapter {
            ls: ls,
            pole_offset: pole_strip_size * pole_strip_index,
            size: pole_strip_size,
        }
    }
}

impl<'a> anim::LedArray for PoleLedArrayAdapter<'a> {
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
fn get_led_array() -> ledscape::LedscapeLedArray {
    ledscape::LedscapeLedArray::new(LEDS_PER_STRING)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn get_led_array() -> opc::OPCLedArray {
    get_opc_array("127.0.0.1:7890").expect("can't connect")
}

fn get_opc_array(adrr: &str) -> std::io::Result<opc::OPCLedArray> {
    Ok(opc::OPCLedArray::new(LEDS_PER_STRING * NUM_POLES, adrr)?)
}

struct StdinEventSource;

impl StdinEventSource {
    fn get_event(&mut self) -> Events {
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        loop {

            let mut buffer = String::new();
            handle.read_line(&mut buffer).expect("can't read");

            let mut words: Vec<&str> = buffer.split_whitespace().collect();

            if (words.len() != 2) && (words.len() != 3) {
                println!("invalid input - exactly two or three!");
                continue;
            }
            let mut stop = false;
            if words.len() == 3 {
                words.remove(0);
                stop = true;
            }
            let word1 = words[0];
            let word2 = words[1];
            let (pole1, pole2) = (word1.parse::<usize>(), word2.parse::<usize>());

            let ourtimeout = std::time::Duration::from_secs(1000);            
            match (stop, pole1, pole2) {
                (false, Ok(p1), Ok(p2)) => {
                    println!("sending touch event {} {}", p1, p2);
                    return Events::Start(EventTypes::Connect(p1, p2));
                },
                (true, Ok(p1), Ok(p2)) => {
                    println!("sending stop touch event {} {}", p1, p2);
                    return Events::Stop(EventTypes::Connect(p1, p2));
                },
                _ => {
                    println!("invalid input! - two numbers please");
                }
            }

        }
    }

    fn get_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1000)
    }
}

fn get_eventer() -> StdinEventSource {
    StdinEventSource
}

fn main() {

    let mut ledscapecontroller = match std::env::var("OPCSERVER") {
        Ok(val) => get_opc_array(&val).expect("can't connect"),
        Err(e) => get_led_array(),
    };

    // TODO add OPCCLient

    let poles = vec![Pole::new(); NUM_POLES];
    let (tx, rx) = mpsc::channel();

    let fps_duration = std::time::Duration::from_secs(1) / 30;
    let newtx = tx.clone();
    std::thread::spawn(move || {
        let tx = newtx;
        loop {
            tx.send(Events::Draw);
            // 30 fps
            std::thread::sleep(fps_duration);
        }
    });

    let eventer = get_eventer();
    let timeout = eventer.get_timeout();

    let newtx = tx.clone();
    std::thread::spawn(move || {
        let mut tx = newtx;
        let mut eventer = eventer;
        loop {
            tx.send(eventer.get_event());
        }
    });

    work(move |poles| draw_poles_to_array(&mut ledscapecontroller, poles),
         poles,
         timeout,
         rx);

    println!("Hello, world!");
}

#[derive(Copy,Clone,Debug)]
struct TouchState {
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

struct TouchMap {
    touches: [[Option<TouchState>; NUM_POLES]; NUM_POLES],
    timeout_period: std::time::Duration,
}

impl TouchMap {
    fn new(timeout : std::time::Duration) -> Self {
        TouchMap {
            touches: [[None; NUM_POLES]; NUM_POLES],
            timeout_period: timeout,
        }
    }


    fn clean_timeout(&mut self) {

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

    fn normalize(pole1: usize, pole2: usize) -> (usize, usize) {
        if pole2 < pole1 {
            (pole2, pole1)
        } else {
            (pole1, pole2)
        }

    }

    fn connect(&mut self, pole1: usize, pole2: usize) {
        let (pole1, pole2) = Self::normalize(pole1, pole2);

        let mut newtouch = self.touches[pole1][pole2].unwrap_or(TouchState::new());
        newtouch.update();
        self.touches[pole1][pole2] = Some(newtouch);
    }

    fn disconnect(&mut self, pole1: usize, pole2: usize) {
        let (pole1, pole2) = Self::normalize(pole1, pole2);
        self.touches[pole1][pole2] = None;
    }

    fn get_touches_for(&self, pole1: usize) -> &[Option<TouchState>; NUM_POLES] {
        &self.touches[pole1]
    }

    fn is_idle(&self) -> bool {
        for tmp in self.touches.iter() {
            for t in tmp {
                if t.is_some() {
                    return false;
                }
            }
        }
        true
    }
}

fn work<F>(mut draw_poles: F, mut poles: Vec<Pole>, timeout : std::time::Duration, receiver: mpsc::Receiver<Events>)
    where F: FnMut(&mut [Pole])
{

    let mut touches = TouchMap::new(timeout);
    let mut animator = Animator::new();

    let mut last_anim_time = std::time::Instant::now();
    for event in receiver.into_iter() {
        match event {
            Events::Start(EventTypes::Touch(pole)) => {
                touches.connect(pole, pole);
            }
            Events::Stop(EventTypes::Touch(pole)) => {
                touches.disconnect(pole, pole);
            }
            Events::Start(EventTypes::Connect(pole1, pole2)) => {
                touches.connect(pole1, pole2);
            }
            Events::Stop(EventTypes::Connect(pole1, pole2)) => {
                touches.disconnect(pole1, pole2);
            }
            Events::Draw => {
                touches.clean_timeout();
                let now = std::time::Instant::now();
                animator.animate_poles(&mut poles, &touches, now - last_anim_time);
                draw_poles(&mut poles);
                last_anim_time = now;
            }
        }
    }
}



#[derive(Clone,Debug)]
enum PoleState {
    Untouched,
    Toched,
    Connected(Vec<(std::time::Instant, usize)>),
}



#[derive(Clone,Debug)]
struct Pole {
    leds: Vec<palette::Hsl>,
}

impl Pole {
    fn new() -> Self {
        Pole {
            leds:
                vec![palette::Hsl::new(palette::RgbHue::from_radians(0.),1.,0.5); LEDS_PER_STRING], 
//            pole_state : PoleState::Untouched,
        }
    }
}

fn draw_poles_to_array(c: &mut anim::LedArray, poles: &[Pole]) {
    for (i, pole) in poles.iter().enumerate() {
        // with ledscape, anim array  is a big array. each LEDS_PER_STRING are one pole.
        let mut adaper = PoleLedArrayAdapter::new(c, LEDS_PER_STRING, i);
        pole.draw(&mut adaper);
    }
    c.show();
}

impl anim::Drawer for Pole {
    fn draw(&self, array: &mut anim::LedArray) {
        // ?!
        for i in 0..array.len() {
            anim::set_color(array, i, self.leds[i]);
        }
    }
}

struct Animator {
    idle_anim: Option<animations::IdleAnim>,
}

impl Animator {
    fn new() -> Self {
        Animator { idle_anim: None }
    }

    fn animate_poles(&mut self,
                     poles: &mut [Pole],
                     touches: &TouchMap,
                     delta: std::time::Duration) {

        if touches.is_idle() {
            if self.idle_anim.is_none() {
                self.idle_anim = Some(animations::IdleAnim::new())
            }
            self.idle_anim.as_mut().unwrap().animate_poles(poles, delta)
        } else {

            for (i, pole) in poles.iter_mut().enumerate() {
                // find out what animation we need:

                // with ledscape, anim array  is a big array. each LEDS_PER_STRING are one pole.
                pole.update_animation(delta);
            }
        }
    }
}

impl anim::Animation for Pole {
    fn update_animation(&mut self, delta: std::time::Duration) {
        // ?!
        let size = self.leds.len() as f32;
        for (i, pixel) in self.leds.iter_mut().enumerate() {
            let newhue: f32 = pixel.hue.to_radians() + delta.as_secs() as f32 +
                              delta.subsec_nanos() as f32 / 1_000_000_000.0;
            pixel.hue = palette::RgbHue::from_radians(newhue);
            let factor: f32 = i as f32 / size;
            pixel.lightness = 0.5 * factor;
        }
    }
}
