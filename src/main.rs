extern crate libc;
extern crate num;
extern crate palette;
extern crate tk_opc;
extern crate bit_set;
extern crate rosc;
#[macro_use]
extern crate log;


use std::sync::mpsc;
use std::io::BufRead;

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

//            let ourtimeout = std::time::Duration::from_secs(1000);
            match (stop, pole1, pole2) {
                (false, Ok(p1), Ok(p2)) => {
                    println!("sending touch event {} {}", p1, p2);
                    return Events::Start(EventTypes::Connect(p1, p2));
                }
                (true, Ok(p1), Ok(p2)) => {
                    println!("sending stop touch event {} {}", p1, p2);
                    return Events::Stop(EventTypes::Connect(p1, p2));
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

fn get_eventer() -> StdinEventSource {
    StdinEventSource
}

fn main() {

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
        loop {
            tx.send(eventer.get_event()).expect("failed to send event to logic!");;
        }
    });

    work(move |poles| draw_poles_to_array(ledscapecontroller.as_mut(), poles),
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
    fn new(timeout: std::time::Duration) -> Self {
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

    fn connect(&mut self, pole1: usize, pole2: usize) {
        //     let (pole1, pole2) = Self::normalize(pole1, pole2);

        let mut newtouch = self.touches[pole1][pole2].unwrap_or(TouchState::new());
        newtouch.update();
        self.touches[pole1][pole2] = Some(newtouch);
        self.touches[pole2][pole1] = Some(newtouch);
    }

    fn disconnect(&mut self, pole1: usize, pole2: usize) {
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
    let mut animator = Animator::new();

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



#[derive(Clone,Debug)]
pub enum PoleState {
    NotTouched,
    Touched,
    ConnectedTo(bit_set::BitSet),
}

#[derive(Copy, Clone,Debug, PartialEq)]
pub enum PoleAnimations {
    Touching,
    ReversedTouching,
    Connecting,
    ReversedConnecting,
    Exoloding,
    ReverseExoloding,
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

struct Animator {
    idle_anim: animations::IdleAnim,
    backgroundsprites: Vec<Box<animations::PoleAnimation>>,
    sprites: Vec<Box<animations::PoleAnimation>>,
}

impl Animator {
    fn new() -> Self {
        Animator {
            idle_anim: animations::IdleAnim::new(),
            sprites: vec![],
            backgroundsprites: vec![],
        }
    }

    fn animate_poles(&mut self,
                     poles: &mut [Pole],
                     touches: &TouchMap,
                     delta: std::time::Duration) {
        use animations::touch::SinglePoleAnimation;

        let black = palette::Hsl::new(palette::RgbHue::from_radians(0.), 0., 0.);

        // darkness
        for p in poles.iter_mut() {
            for pixel in p.leds.iter_mut() {
                *pixel = black;
            }
        }

        // update sprites
        for sprite in self.sprites.iter_mut().chain(self.backgroundsprites.iter_mut()) {
            sprite.update(delta);
        }


        // because of borrow checker i can't pass self, so pass this temp vector instead.
        let mut newsprites: Vec<Box<animations::PoleAnimation>> = vec![];
        self.idle_anim.animate_poles(|sprit| newsprites.push(sprit), poles, delta);
        self.backgroundsprites.extend(newsprites);

        // output background sprites
        self.backgroundsprites.retain(|ref x| !x.is_done());
        for sprite in self.backgroundsprites.iter() {
            sprite.animate_poles(poles);
        }

        // find out all connection.
        // each pole should have an animation assigned to it.
        //
        for (i, row) in touches.touches.iter().enumerate() {
            let mut current_touches: bit_set::BitSet =
                row.iter().enumerate().filter_map(|(ind, &e)| e.map(|_| ind)).collect();

            let is_self_touching = current_touches.remove(i);

            // if they are both empty
            let (old_state, new_state) = {

                let cur_pole = &mut poles[i];
                let new_state = {
                    if !is_self_touching && current_touches.is_empty() {
                        PoleState::NotTouched
                    } else if is_self_touching && current_touches.is_empty() {
                        PoleState::Touched
                    } else {
                        PoleState::ConnectedTo(current_touches)
                    }
                };
                // TODO check for explosions
                (std::mem::replace(&mut cur_pole.state, new_state.clone()), new_state)
            };


            let mut new_anim = poles[i].anim;

            if self.transition(&old_state, &new_state) {
                // change animation
                new_anim = match new_state {
                    PoleState::NotTouched => None,
                    PoleState::Touched => Some(PoleAnimations::Touching),
                    // if one of the other poles in the change has level == 1 then exploding
                    PoleState::ConnectedTo(_) => {
                        /* re calc colors */
                        Some(PoleAnimations::Connecting)
                    }
                };
            }


            if let PoleState::ConnectedTo(ref others) = new_state {
                let was_exploding = poles[i].anim == Some(PoleAnimations::Exoloding);
                let is_exploding = others.iter().any(|i| poles[i].level == 1.);
                if was_exploding != is_exploding {
                    if is_exploding {
                        // TODO: send midi signal
                        new_anim = Some(PoleAnimations::Exoloding);
                    } else {
                        // TODO: send midi signal
                        new_anim = Some(PoleAnimations::Connecting);
                    }
                }
            }

            poles[i].anim = new_anim;

        }


        // if out all that is toched is the pole being touched:

        for pole in poles.iter_mut() {
            //

            // find out what animation we need:

            // with ledscape, anim array  is a big array. each LEDS_PER_STRING are one pole.
            //          pole.update_animation(delta);

            // todo:: should the level updates happen here?!
            // self.update_state
            // self. update_levels.
            // bubling sprites etc..
/*
            match pole.state {
                PoleState::Untouched => {
                    
                }
            }
*/


            let old_level = pole.level;
            match pole.anim {
                Some(PoleAnimations::Touching) => {
                    animations::touch::TouchAnim::animate_pole(pole, delta);
                }
                Some(PoleAnimations::Connecting) => {
                    animations::touch::ConnectedAnim::animate_pole(pole, delta);
                }
                None => {
                    animations::touch::ReverseTouchAnim::animate_pole(pole, delta);
                }
                Some(PoleAnimations::Exoloding) => {
                    animations::touch::ExplodingAnim::animate_pole(pole, delta);
                }
                Some(_) => {
                    //animations::touch::ConnectedAnim::animate_pole(pole, delta);
                }
            };

            if old_level >= 1. && pole.level < 1. {
                // TODO: set low touch
                // stop riser.
                // this only happens when not touched, so need to high regular touch off.

                
                
                // TODO: send midi!
            } else if old_level < 1. && pole.level >= 1. {
                // TODO: set high touch / connected
                // this only happens when touched, so need to send regular touch off first.

                // TODO: send midi!
            }

            if pole.level > 0. {
                let len = pole.leds.len();

                let circl_index: usize = (pole.level * len as f32) as usize;

                for pixel in pole.leds.iter_mut().rev().take(circl_index) {
                    *pixel = pole.base_color;
                }
            }
            if pole.touch_level > 0. {
                let len = pole.leds.len();
                let circl_index: usize = (pole.touch_level * len as f32) as usize;

                for pixel in pole.leds.iter_mut().take(circl_index) {
                    *pixel = pole.current_color;
                }
            }



        }


        // output sprites
        self.sprites.retain(|ref x| !x.is_done());
        for sprite in self.sprites.iter() {
            sprite.animate_poles(poles);
        }


    }


    fn transition(&self, old_state: &PoleState, new_state: &PoleState) -> bool {

        match *old_state {
            PoleState::NotTouched => {
                match *new_state {
                    PoleState::NotTouched => false,
                    PoleState::Touched => {
                        /* send midi! */
                        true
                    }
                    PoleState::ConnectedTo(_) => {
                        /* send midi! */
                        true
                    }
                }
            }
            PoleState::Touched => {
                match *new_state {
                    PoleState::NotTouched => {
                        /* send midi! */
                        true
                    }
                    PoleState::Touched => false,
                    PoleState::ConnectedTo(_) => {
                        /* send midi! */
                        true
                    }
                }
            }
            PoleState::ConnectedTo(ref x) => {
                match *new_state {
                    PoleState::NotTouched => {
                        /* send midi! */
                        true
                    }
                    PoleState::Touched => {
                        /* send midi! */
                        true
                    }
                    PoleState::ConnectedTo(ref y) => x != y, 
                    /* send midi if needed! or maybe not! */
                }
            }
        }


        /*
        for new_touch in current_touches.difference(&old_tocuhes) {
            // TODO: signal new touch
            // TODO: if old state was nothing, create animation 

        }
        for removed_touch in old_tocuhes.difference(&current_touches) {
            // TODO signal removed touch                    
           for other_ind in v.into_iter() {
                let other_pole = &mut poles[other_ind]; 
                if let PoleState::ConnectedTo(ref mut otherv) = other_pole.state {
                    otherv.remove(i);
                }
            }
        }
  */
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
