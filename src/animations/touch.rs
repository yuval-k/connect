use std;
const LED_ANIM_DURATION: f32 = 5.;


pub trait SinglePoleAnimation {
    fn animate_pole(poles: &mut super::super::Pole, delta: std::time::Duration);
}

#[derive(Clone,Debug)]
pub struct TouchAnim;

impl SinglePoleAnimation for TouchAnim {
    fn animate_pole(pole: &mut super::super::Pole, delta: std::time::Duration) {
        // five secs to get to 30% hight
        let delta: f32 = 0.3 * super::to_float(delta) / LED_ANIM_DURATION;
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
    fn animate_pole(pole: &mut super::super::Pole, delta: std::time::Duration) {
        // five secs to get to 30% hight
        let delta: f32 = 0.3 * super::to_float(delta) / LED_ANIM_DURATION;
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
    fn animate_pole(pole: &mut super::super::Pole, delta: std::time::Duration) {

        let delta: f32 = super::to_float(delta) / LED_ANIM_DURATION;
        // remove touch level if ut is there
        pole.level += delta;
        if pole.level > 1. {
            pole.level = 1.;
        }

    }
}

pub struct ExplodingAnim;

impl SinglePoleAnimation for ExplodingAnim {
    fn animate_pole(pole: &mut super::super::Pole, delta: std::time::Duration) {
        // TODO: add sprites some how
        let delta: f32 = super::to_float(delta) / LED_ANIM_DURATION;
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
