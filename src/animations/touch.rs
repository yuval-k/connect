use std;
use palette;

use std::ops::Rem;

const LED_ANIM_DURATION: u64 = 10;


pub trait SinglePoleAnimation {
    fn is_done(&super::super::Pole) -> bool;
    fn animate_pole(poles: &mut super::super::Pole, delta: std::time::Duration);
}

#[derive(Clone,Debug)]
pub struct TouchAnim;

impl SinglePoleAnimation for TouchAnim {
    fn is_done(p: &super::super::Pole) -> bool {
        false
    }

    fn animate_pole(pole: &mut super::super::Pole, delta: std::time::Duration) {
        // five secs to get to 30% hight
        let delta: f32 = 0.3 * super::to_float(delta) / 5.;
        // remove touch level if ut is there
        if pole.touch_level > 0. {
            pole.touch_level -= delta;
            if pole.touch_level < 0. {
                pole.level += pole.touch_level;
                pole.touch_level = 0.;
            }
        } else {
            if pole.level < 0.3 {
                pole.level += delta;
            }
            if pole.level > 0.3 {
                pole.level -= delta;
            }
        }
        // TODO: hadle the case when we are close to 0.3
       
    }
}


pub struct ReverseTouchAnim;

impl SinglePoleAnimation for ReverseTouchAnim {
    fn is_done(pole: &super::super::Pole) -> bool {
        pole.level == 0.
    }

    fn animate_pole(pole: &mut super::super::Pole, delta: std::time::Duration) {
        // five secs to get to 30% hight
        let delta: f32 = 0.3 * super::to_float(delta) / 5.;
        // remove touch level if ut is there
        if pole.touch_level > 0. {
            pole.touch_level -= delta;
            if pole.touch_level < 0. {
                pole.level += pole.touch_level;
                pole.touch_level = 0.;
            }
        } else {

            if pole.level > 0. {
                pole.level -= delta;
                if pole.level < 0. {
                    pole.level = 0.
                }
            }
        }
        
        // TODO: hadle the case when we are close to 0.3
       
    }
}
pub struct ConnectedAnim;

impl SinglePoleAnimation for ConnectedAnim {
    fn is_done(p: &super::super::Pole) -> bool {
        false
    }

    fn animate_pole(pole: &mut super::super::Pole, delta: std::time::Duration) {

        let delta: f32 = super::to_float(delta) / 5.;
        // remove touch level if ut is there
        pole.level += delta;
        if pole.level > 1. {
            pole.level = 1.;
        }

    }
}

pub struct ReverseConnectedAnim {
    phase: super::AnimPhase,
}

pub struct ExplodingAnim;


impl SinglePoleAnimation for ExplodingAnim {
    fn is_done(p: &super::super::Pole) -> bool {
        false
    }

    fn animate_pole(pole: &mut super::super::Pole, delta: std::time::Duration) {

        let delta: f32 = super::to_float(delta) / 5.;
        // remove touch level if ut is there
        pole.level += delta;
        if pole.level > 1. {
            pole.touch_level += pole.level - 1.;
            pole.level = 1.;
        }

        if pole.touch_level > 1. {
            pole.touch_level = 1.;
        }

    }
}


pub struct ReverseExplodingAnim {
    phase: super::AnimPhase,
}
