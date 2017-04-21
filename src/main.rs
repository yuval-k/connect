extern crate libc;
extern crate num;
extern crate palette;

use std::sync::mpsc;
mod ledscape;
mod anim;

use anim::Animation;

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

struct SimpleLedArrayAdapter {
    ls: ledscape::LEDScape,
    strip_size: usize,
    frame_index: ledscape::FramedIndex,
    frame: ledscape::LEDScapeFrame,
}


impl SimpleLedArrayAdapter {
    fn new(strip_size: usize) -> Self {
        let mut ls = unsafe { ledscape::LEDScape::new(strip_size) };
        let frame = unsafe { ls.frame(ledscape::FramedIndex::FirstFrame) };
        SimpleLedArrayAdapter {
            ls: unsafe { ledscape::LEDScape::new(strip_size) },
            strip_size: strip_size,
            frame_index: ledscape::FramedIndex::FirstFrame,
            frame: frame,
        }
    }
}

impl anim::LedArray for SimpleLedArrayAdapter {
    fn len(&self) -> usize {
        self.strip_size * ledscape::LEDSCAPE_NUM_STRIPS
    }

    fn set_color_rgba(&mut self, lednum: usize, r: u8, g: u8, b: u8, a: u8) {
        let (stripindex, ledindex) = (lednum / self.strip_size, lednum % self.strip_size);
        self.frame.set_pixel(stripindex, ledindex, r, g, b, a);
    }

    fn show(&mut self) -> std::io::Result<()> {
        self.ls.wait();
        self.ls.draw(self.frame_index);
        self.frame_index = self.frame_index.other_frame();
        self.frame = unsafe { self.ls.frame(self.frame_index) };
        Ok(())
    }
}

fn main() {

    //    let ledscapecontroller = unsafe{ledscape::LEDScape::new(LEDS_PER_STRING)};
    let mut ledscapecontroller = SimpleLedArrayAdapter::new(LEDS_PER_STRING);

// TODO add OPCCLient

    let poles = vec![];
    let (tx, rx) = mpsc::channel();

    let fps_duration = std::time::Duration::from_secs(1)/30;

    std::thread::spawn(move||{
        let tx = tx.clone();
        loop {
            tx.send(Events::Draw);
            // 30 fps
            std::thread::sleep(fps_duration);
        }
    });

    work(move |poles, duration| animate_poles_ledscape(&mut ledscapecontroller, poles, duration) , poles, rx);

    println!("Hello, world!");
}

fn work<F> (mut animate_poles: F, mut poles: Vec<Pole>, receiver: mpsc::Receiver<Events>) 
    where F: FnMut(&mut [Pole], std::time::Duration) {

    let mut last_anim_time = std::time::Instant::now();
    for event in receiver.into_iter() {
        match event {
            Events::Start(EventTypes::Touch(pole)) => {}
            Events::Stop(EventTypes::Touch(pole)) => {}
            Events::Start(EventTypes::Connect(pole1, pole2)) => {}
            Events::Stop(EventTypes::Connect(pole1, pole2)) => {}
            Draw => {
                let now = std::time::Instant::now();
                animate_poles(&mut poles, now - last_anim_time);
                last_anim_time = now;
            }
        }
    }
}

enum PoleState {
    Untouched,
    Touching,
    Connected(Vec<usize>),
}

struct Pole {
    pole_state: PoleState,
    
}



fn animate_poles_ledscape(c: &mut anim::LedArray, poles: &mut [Pole], delta: std::time::Duration) {
    for (i, pole) in poles.iter_mut().enumerate() {
    // with ledscape, anim array  is a big array. each LEDS_PER_STRING are one pole.
        let mut adaper = PoleLedArrayAdapter::new(c, LEDS_PER_STRING, i);
        pole.animate(&mut adaper, delta);
    }
    c.show();
}

impl anim::Animation for Pole {
    fn animate(&mut self, array: &mut anim::LedArray, delta: std::time::Duration) {
        // ?!
    }
}

