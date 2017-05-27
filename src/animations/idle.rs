use std;
use palette;
use rand;

use std::ops::Rem;

const LED_ANIM_DURATION: u64 = 10;

fn to_float(t: std::time::Duration) -> f32 {
    t.as_secs() as f32 + t.subsec_nanos() as f32 / 1_000_000_000.0
}

#[derive(Copy,Clone,Debug)]
pub struct AnimPhase {
    total_time: f32,
    current_pos: f32, // between 0 and 1
}

impl AnimPhase {
    pub fn new(total_time: std::time::Duration) -> AnimPhase {
        AnimPhase {
            total_time: to_float(total_time),
            current_pos: 0.,
        }
    }

    pub fn update(&mut self, delta: std::time::Duration) -> f32 {
        let delta = to_float(delta) / self.total_time;
        self.current_pos += delta;
        self.current_pos
    }

    pub fn cycle(&mut self) -> f32 {
        self.current_pos = self.current_pos.rem(1.0);
        self.current_pos
    }

    pub fn cyclic_update(&mut self, delta: std::time::Duration) -> f32 {
        self.update(delta);
        self.cycle()
    }

    pub fn current(&self) -> f32 {
        self.current_pos
    }

    pub fn is_done(&self) -> bool {
        self.current_pos >= 1.0
    }
}

pub struct CircleAnim {
    pub phase: AnimPhase,
    pub color: palette::Hsl,
}

impl CircleAnim {
    pub fn new(color: palette::Hsl, total_time: std::time::Duration) -> Self {
        CircleAnim {
            phase: AnimPhase::new(total_time),
            color: color,
        }
    }
}

impl super::PoleAnimation for CircleAnim {
    fn update(&mut self, delta: std::time::Duration) {
        self.phase.update(delta);
    }
    fn is_done(&self) -> bool {
        self.phase.is_done()
    }

    fn animate_poles(&self, poles: &mut [super::super::Pole]) {

        for p in poles {
            let len = p.leds().len();

            let circl_index: usize = (self.phase.current() * len as f32) as usize;

            for (i, pixel) in p.leds().iter_mut().enumerate() {

                if (i + 1) == circl_index {
                    *pixel = self.color;
                } else if i == circl_index {
                    *pixel = self.color;
                } else if i == (circl_index + 1) {
                    *pixel = self.color;
                }

            }

        }
    }
}





pub struct ExplosionAnim {
    pub phase: AnimPhase,
    pub color: palette::Hsl,
}

impl ExplosionAnim {
    pub fn new() -> Self {
        ExplosionAnim {
            phase: AnimPhase::new(std::time::Duration::from_millis(500)),
            color: palette::Hsl::new(palette::RgbHue::from_radians(0.),1.0,1.0),
        }
    }
}

impl super::PoleAnimation for ExplosionAnim {
    fn update(&mut self, delta: std::time::Duration) {
        self.phase.update(delta);
    }
    fn is_done(&self) -> bool {
        self.phase.is_done()
    }

    fn animate_poles(&self, poles: &mut [super::super::Pole]) {
        use std::f32;

let curpos = self.phase.current();
        let current_pixels = (8.0f32*(curpos*std::f32::consts::PI).sin()) as usize; //if curpos  <= 0.5 {   (10.0*2.0* curpos) as usize } else { (10.0*2.0* (1.0-curpos)) as usize } ;

        for p in poles {
            let len = std::cmp::min(current_pixels, p.leds().len());

            for pixel in p.leds().iter_mut().take(len) {
                *pixel = self.color;
            }

        }
    }
}



pub struct TwinkleAnim {
    pub phase: AnimPhase,
    pub color: palette::Hsl,
    pub index : Vec<(usize,usize)>,
}

impl TwinkleAnim {
    pub fn new(color: palette::Hsl, total_time: std::time::Duration) -> Self {
        let mut index : Vec<(usize,usize)> = vec![];

        let rand_pixel = rand::random::<usize>() % 32;

        for i in 0..5 {
        let rand_pole = rand::random::<usize>() % super::NUM_POLES;
        // TODO make sure rand_pole is differnet each time., and rand pixel is not on the same line.
            index.push((rand_pole, rand_pixel));
        }

        TwinkleAnim {
            phase: AnimPhase::new(total_time),
            color: color,
            index: index
        }
    }
}


impl super::PoleAnimation for TwinkleAnim {
    fn update(&mut self, delta: std::time::Duration) {
        self.phase.update(delta);
    }
    fn is_done(&self) -> bool {
        self.phase.is_done()
    }

    fn animate_poles(&self, poles: &mut [super::super::Pole]) {
        use std::f32;

        let curpos = self.phase.current();
        let current_pixels = (8.0f32*(curpos*std::f32::consts::PI).sin()) as usize; //if curpos  <= 0.5 {   (10.0*2.0* curpos) as usize } else { (10.0*2.0* (1.0-curpos)) as usize } ;

        for &(pole, led) in self.index.iter() {
            poles[pole].leds()[led] = self.color;
        }
    }
}





pub struct IdleAnim {
    add_circle_phase: AnimPhase,
}

impl IdleAnim {
    pub fn new() -> Self {
        IdleAnim { add_circle_phase: AnimPhase::new(std::time::Duration::from_millis(200)) }
    }

    pub fn animate_poles<F>(&mut self,
                            mut animator: F,
                            poles: &mut [super::super::Pole],
                            delta: std::time::Duration)
        where F: FnMut(Box<super::PoleAnimation>)
    {


        self.add_circle_phase.update(delta);
        if self.add_circle_phase.is_done() {
            (animator)(Box::new(TwinkleAnim::new(palette::Hsl::new(palette::RgbHue::from_radians(0.0),
                                                               1.0,
                                                               1.0),
                                             std::time::Duration::from_millis(200))));
        }
        self.add_circle_phase.cycle();

        let color_background = palette::Hsl::new(palette::RgbHue::from_radians(248.0*std::f32::consts::PI/180.0),
                                                               0.98,
                                                               0.15);

        for p in poles.iter_mut() {
            // darkness
            for pixel in p.leds().iter_mut() {
                *pixel = color_background;
            }
        }

    }
}
