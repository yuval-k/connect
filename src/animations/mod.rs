use std;
use palette;

use std::ops::Rem;

const LED_ANIM_DURATION: u64 = 10;

fn to_float(t: std::time::Duration) -> f32 {
    t.as_secs() as f32 + t.subsec_nanos() as f32 / 1_000_000_000.0
}

struct AnimPhase {
    total_time: f32,
    current_pos: f32, // between 0 and 1
}

impl AnimPhase {
    fn new(total_time: std::time::Duration) -> AnimPhase {
        AnimPhase {
            total_time: to_float(total_time),
            current_pos: 0.,
        }
    }

    fn update(&mut self, delta: std::time::Duration) -> f32 {
        let delta = to_float(delta) / self.total_time;
        self.current_pos += delta;
        self.current_pos
    }

    fn cycle(&mut self) -> f32 {
        self.current_pos = self.current_pos.rem(1.0);
        self.current_pos
    }

    fn cyclic_update(&mut self, delta: std::time::Duration) -> f32 {
        self.update(delta);
        self.cycle()
    }

    fn current(&self) -> f32 {
        self.current_pos
    }
}

pub struct IdleAnim {
    phases: Vec<AnimPhase>,
    add_circle_phase: AnimPhase,
}

impl IdleAnim {
    pub fn new() -> Self {
        IdleAnim {
            phases: vec![AnimPhase::new(std::time::Duration::from_secs(LED_ANIM_DURATION))],
            add_circle_phase: AnimPhase::new(std::time::Duration::from_secs(1)),
        }
    }

    pub fn animate_poles(&mut self, poles: &mut [super::Pole], delta: std::time::Duration) {


        for phase in self.phases.iter_mut() {
            phase.cyclic_update(delta);
        }

        self.phases.retain(|ref x| x.current() <= 1.0);


        if self.add_circle_phase.update(delta) > 1. {
            self.phases.push(AnimPhase::new(std::time::Duration::from_secs(LED_ANIM_DURATION)));
        }
        self.add_circle_phase.cycle();

        for p in poles {
            let len = p.leds.len();

            // darkness
            for (i, pixel) in p.leds.iter_mut().enumerate() {
                *pixel = palette::Hsl::new(palette::RgbHue::from_radians(0.), 0., 0.);
            }

            for current_phase in self.phases.iter() {
                let circlIndex: usize = (current_phase.current() * len as f32) as usize;

                for (i, pixel) in p.leds.iter_mut().enumerate() {

                    if (i+1) == circlIndex {
                        *pixel = palette::Hsl::new(palette::RgbHue::from_radians(0.), 1., 0.25);
                    } else if i == circlIndex {
                        *pixel = palette::Hsl::new(palette::RgbHue::from_radians(0.), 1., 0.5);
                    } else if i == (circlIndex+1) {
                        *pixel = palette::Hsl::new(palette::RgbHue::from_radians(0.), 1., 0.25);
                    }
                }
            }

        }
    }
}