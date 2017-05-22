use std;
use palette;

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


pub struct IdleAnim {
    add_circle_phase: AnimPhase,
}

impl IdleAnim {
    pub fn new() -> Self {
        IdleAnim { add_circle_phase: AnimPhase::new(std::time::Duration::from_secs(1)) }
    }

    pub fn animate_poles<F>(&mut self,
                            mut animator: F,
                            poles: &mut [super::super::Pole],
                            delta: std::time::Duration)
        where F: FnMut(Box<super::PoleAnimation>)
    {


        self.add_circle_phase.update(delta);
        if self.add_circle_phase.is_done() {
            (animator)(Box::new(CircleAnim::new(palette::Hsl::new(palette::RgbHue::from_radians(0.),
                                                               1.,
                                                               0.5),
                                             std::time::Duration::from_secs(LED_ANIM_DURATION))));
        }
        self.add_circle_phase.cycle();

        let color_background = palette::Hsl::new(palette::RgbHue::from_radians(0.), 1., 0.05);

        for p in poles.iter_mut() {
            // darkness
            for pixel in p.leds().iter_mut() {
                *pixel = color_background;
            }
        }

    }
}
