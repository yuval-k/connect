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
extern crate bitflags;

use std::sync::mpsc;
use std::io::BufRead;
use serial::SerialPort;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
mod ledscape;

mod anim;
mod animations;
mod opc;
mod osc;

use anim::Drawer;

#[derive(Clone,Copy,Debug)]
enum EventTypes {
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
    use anim::LedArray;
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
    l
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn get_led_array() -> opc::OPCLedArray {
    get_opc_array("127.0.0.1:7890").expect("can't connect")
}

fn get_opc_array(adrr: &str) -> std::io::Result<opc::OPCLedArray> {
    Ok(opc::OPCLedArray::new(LEDS_PER_STRING * NUM_POLES, adrr)?)
}

trait Eventer {
    fn get_events(&mut self, sender: std::sync::mpsc::Sender<Events>);

    fn get_timeout(&self) -> std::time::Duration;
}

struct StdinEventSource;


impl Eventer for StdinEventSource {
    fn get_events(&mut self, mut sender: std::sync::mpsc::Sender<Events>) {
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

            //            let ourtimeout = std::time::Duration::from_secs(1000);
            match (stop, pole1, pole2) {
                (false, Ok(p1), Ok(p2)) => {
                    println!("sending touch event {} {}", p1, p2);
                    sender.send(Events::Start(EventTypes::Connect(p1, p2)));
                }
                (true, Ok(p1), Ok(p2)) => {
                    println!("sending stop touch event {} {}", p1, p2);
                    sender.send(Events::Stop(EventTypes::Connect(p1, p2)));
                }
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

struct SerialEventSource;
impl Eventer for SerialEventSource {
    fn get_events(&mut self, mut sender: std::sync::mpsc::Sender<Events>) {
        loop {
            self.eventloop(&mut sender);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

    }

    fn get_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1)
    }
}

impl SerialEventSource {
    fn eventloop(&mut self, sender: &mut std::sync::mpsc::Sender<Events>) -> std::io::Result<()> {
        let device = "/dev/ttyACM0";

        let mut port = serial::open(device)?;
        port.reconfigure(&|settings| {
                settings.set_baud_rate(serial::Baud115200)?;
                settings.set_char_size(serial::Bits8);
                settings.set_parity(serial::ParityNone);
                settings.set_stop_bits(serial::Stop1);
                settings.set_flow_control(serial::FlowNone);
                Ok(())
            })?;

        port.set_timeout(std::time::Duration::from_millis(100))?;

        let mut reader = std::io::BufReader::new(port);

        let mut line = String::new();

        loop {
            line.clear();
            reader.read_line(&mut line)?;
            let indexes: Result<Vec<usize>, _> =
                line.split(':').map(|s| s.parse::<usize>()).collect();
            match indexes {
                Err(_) => {
                    continue;
                }
                Ok(v) => {
                    if !v.is_empty() {
                        self.send_events(sender, v[0], &v[1..]);
                    }
                }
            }
        }
    }

    fn send_events(&mut self,
                   sender: &mut std::sync::mpsc::Sender<Events>,
                   senderindex: usize,
                   touches: &[usize]) {

        for i in 0..NUM_POLES {
            let event = match touches.iter().find(|&&x| x == i) {
                None => Events::Stop(EventTypes::Connect(senderindex, i)),
                Some(_) => Events::Start(EventTypes::Connect(senderindex, i)),
            };
            sender.send(event);
        }

    }
}

fn get_eventer() -> StdinEventSource {
    StdinEventSource
}

fn main() {

    env_logger::init().unwrap();

    info!("hello");
    let mut ledscapecontroller: Box<anim::LedArray> = match std::env::var("OPCSERVER") {
        Ok(val) => Box::new(get_opc_array(&val).expect("can't connect")),
        Err(_) => Box::new(get_led_array()),
    };

    // TODO add OPCCLient

    let poles = vec![Pole::new(); NUM_POLES];
    let (tx, rx) = mpsc::channel();

    // 30 fps
    let fps_duration = std::time::Duration::from_secs(1) / 30;
    let newtx = tx.clone();
    std::thread::spawn(move || {
        let tx = newtx;
        loop {
            tx.send(Events::Draw).expect("drawing event failed to send!");
            std::thread::sleep(fps_duration);
        }
    });

    let eventer = get_eventer();
    let timeout = eventer.get_timeout();

    let newtx = tx.clone();
    std::thread::spawn(move || {
        let tx = newtx;
        let mut eventer = eventer;
        eventer.get_events(tx);
        panic!("event loop should be endless")
    });

    work(move |poles| draw_poles_to_array(ledscapecontroller.as_mut(), poles),
         poles,
         timeout,
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
}

fn work<F>(mut draw_poles: F,
           mut poles: Vec<Pole>,
           timeout: std::time::Duration,
           receiver: mpsc::Receiver<Events>)
    where F: FnMut(&mut [Pole])
{

    let mut touches = TouchMap::new(timeout);
    let mut animator = animations::Animator::new(osc::OSCManager::new("192.168.1.22:8100"));

    let mut last_anim_time = std::time::Instant::now();
    for event in receiver.into_iter() {
        match event {
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
    Exoloding,
}

#[derive(Clone,Debug)]
pub struct Pole {
    pub level: f32,
    pub touch_level: f32,
    pub base_color: palette::Hsl,
    pub current_color: palette::Hsl,
    pub leds: Vec<palette::Hsl>,

    pub state: PoleState,
    pub anim: Option<PoleAnimations>,
}

impl Pole {
    fn new() -> Self {
        Pole {
            level: 0.,
            touch_level: 0.,
            leds:
                vec![palette::Hsl::new(palette::RgbHue::from_radians(0.),1.,0.5); LEDS_PER_STRING],
            //            pole_state : PoleState::Untouched,
            base_color: palette::Hsl::new(palette::RgbHue::from_radians(0.), 1., 0.5),

            current_color: palette::Hsl::new(palette::RgbHue::from_radians(1.), 1., 0.5),

            anim: None,
            state: PoleState::NotTouched,
        }
    }
}

fn draw_poles_to_array(c: &mut anim::LedArray, poles: &[Pole]) {
    for (i, pole) in poles.iter().enumerate() {
        // with ledscape, anim array  is a big array. each LEDS_PER_STRING are one pole.
        let mut adaper = PoleLedArrayAdapter::new(c, LEDS_PER_STRING, i);
        pole.draw(&mut adaper);
    }
    if let Err(e) = c.show() {
        error!("Error showing leds {:?} ", e);
    }
}

impl anim::Drawer for Pole {
    fn draw(&self, array: &mut anim::LedArray) {
        // ?!
        for i in 0..array.len() {
            anim::set_color(array, i, self.leds[i]);
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
