use std;
use palette;
use bit_set;


use std::ops::Rem;

extern crate rand;
use rand::Rng;

pub mod idle;
pub mod touch;

use super::NUM_POLES;

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

pub trait PoleAnimation {
    fn update(&mut self, delta: std::time::Duration);
    fn is_done(&self) -> bool;
    fn animate_poles(&self, poles: &mut [super::Pole]);
}


pub struct Animator {
    idle_anim: idle::IdleAnim,
    backgroundsprites: Vec<Box<PoleAnimation>>,
    sprites: Vec<Box<PoleAnimation>>,
    osc: super::osc::OSCManager,

    heart_phase: AnimPhase,
    flower_phase: AnimPhase,
    disco_phase: AnimPhase,
    disco_state : f32,
    anim1_phase: AnimPhase,
    anim1_poles: [i32;5],
    anim1_color: f32,
    anim1_direction: i32,
    anim1_cleared_poles:[i32;NUM_POLES]
}

impl Animator {
    pub fn new(osc: super::osc::OSCManager) -> Self {
        Animator {
            idle_anim: idle::IdleAnim::new(),
            sprites: vec![],
            backgroundsprites: vec![],
            osc: osc,
            heart_phase: AnimPhase::new(std::time::Duration::from_secs(5)),
            
            flower_phase: AnimPhase::new(std::time::Duration::from_secs(10)),
            

            disco_phase: AnimPhase::new(std::time::Duration::from_millis(500)),
            disco_state: 0.0,
            anim1_phase: AnimPhase::new(std::time::Duration::from_millis(2000)),
            anim1_poles: [0,-1,-1,-1,-1],
            anim1_color: 0.0,
            anim1_direction: -1,
            anim1_cleared_poles:[0;NUM_POLES],
        }
    }

    pub fn animate_anim1(&mut self, poles: &mut [super::Pole], delta: std::time::Duration) {
        let current = self.anim1_phase.update(delta);

        let step: i32 = 10;

        //5 leds and clean trail
        let hue = [palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.1),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.3),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.5),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.6),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.8),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.8),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.6),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.5),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.3),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.1),
            palette::Hsl::new(palette::RgbHue::from_radians(self.anim1_color), 1., 0.0)];


        if self.anim1_phase.is_done() {
            //clear the pole
            for &active_pole in self.anim1_poles.iter() {
                if active_pole >= 0 {
                    for pixel in poles[active_pole as usize].leds() {
                        *pixel = hue[step as usize];
                    }
                    self.anim1_cleared_poles[active_pole as usize] = 1;
                }
            }
            // choose new poles
            let num_active_poles = rand::thread_rng().gen_range(1, self.anim1_poles.len()+1);
            for i in 0..num_active_poles {
                self.anim1_poles[i] =  rand::thread_rng().gen_range(0, NUM_POLES as i32);
            }
            for i in num_active_poles..self.anim1_poles.len() {
                self.anim1_poles[i] = -1;
            }
            //if all poles been cleared, light it all up
            match self.anim1_cleared_poles.iter().min() {
                Some(&1) => {
                    for pole_i in poles.iter_mut() {
                        for pixel in pole_i.leds() {
                             *pixel = hue[3];
                        }
                    }
                    self.anim1_cleared_poles = [0;20];
                },
                Some(&_) => {},
                None => {},
            }

            self.anim1_color = rand::thread_rng().gen_range(0, 360) as f32/60. as f32;
            self.anim1_direction = self.anim1_direction*-1;
            self.anim1_phase.cycle();
        }

        for &active_pole in self.anim1_poles.iter() {
            if active_pole >= 0 {
                let pole = poles[active_pole as usize].leds();

                //update lighting pixels 0 --> 159
                if self.anim1_direction >0 {
                    let leadp =  (current*(149.0+step as f32)) as i32;
                    for pixel_i in 0..(step) {
                        if (leadp - pixel_i >= 0) && (leadp - pixel_i < 150) {
                            let p = pole.iter_mut().nth((leadp - pixel_i + 1) as usize);
                            p.map(|p| *p = hue[pixel_i as usize]);
                            //*p = hue[pixel_i];
                        }
                    }
                    for clean_pixel in 0..150 {
                        if clean_pixel > leadp || clean_pixel < leadp - step{
                            let p = pole.iter_mut().nth((clean_pixel + 1) as usize);
                            p.map(|p| *p = hue[step as usize]);
                        }
                    }
                } else {
                    //update lighting pixels 149 --> -10
                    let leadp = ((1.0-current)*(149.0+step as f32)) as i32  -step as i32;

                    for pixel_i in 0..(step) {
                        if (leadp + pixel_i as i32  >= 0) && ((leadp + pixel_i as i32) < 150) {
                            let p = pole.iter_mut().nth((leadp + pixel_i as i32 + 1) as usize);
                            p.map(|p| *p = hue[(step- pixel_i) as usize]);
                        }
                    }
                    //clear trail
                    for clean_pixel in 0..150 {
                        if clean_pixel < leadp || clean_pixel > leadp + step{
                            let p = pole.iter_mut().nth((clean_pixel + 1) as usize);
                            p.map(|p| *p = hue[step as usize]);
                        }
                    }
                }
            }
        }
    }

    pub fn animate_disco(&mut self, poles: &mut [super::Pole], delta: std::time::Duration) {
         self.disco_phase.update(delta);

         if self.disco_phase.is_done() {
             self.disco_state += 1.0;
             self.disco_phase.cycle();
         } else {
             return;
         }

        for i in 0..(NUM_POLES / 2) {
            // TODO: randomize this
            let currentangle = 2. * std::f32::consts::PI  *0.3245251*(self.disco_state+i as f32);
            let hue = palette::Hsl::new(palette::RgbHue::from_radians(currentangle), 1., 0.5);

            Self::draw_petal_cp2(poles, i , hue);
        }

        for i in 0..(NUM_POLES / 2) {
            // TODO: randomize this
            let currentangle = 2. * std::f32::consts::PI  *0.3245251*(self.disco_state+5.+i as f32);
            let hue = palette::Hsl::new(palette::RgbHue::from_radians(currentangle), 1., 0.5);

            Self::draw_petal_cp1(poles, i , hue);
        }
    }



    pub fn animate_flower(&mut self, poles: &mut [super::Pole], delta: std::time::Duration) {
        let current = self.flower_phase.cyclic_update(delta);

        for i in 0..(NUM_POLES / 2) {
            let string1 = i * 2;

            let currentangle = 2. * std::f32::consts::PI *
                               (current + (string1 as f32) / (NUM_POLES as f32));
            let oppositeangle = currentangle - std::f32::consts::PI;
            let ophue = palette::Hsl::new(palette::RgbHue::from_radians(oppositeangle), 1., 0.5);

            Self::draw_petal_cp2(poles, i , ophue);
        }

        for i in 0..(NUM_POLES / 2) {

            // first = 1p, second = 1m, third = 2p; petal = 1m + 2p
            let string1 = i * 2;

            let currentangle = 2. * std::f32::consts::PI *
                               (current + (string1 as f32) / (NUM_POLES as f32));
            let curhue = palette::Hsl::new(palette::RgbHue::from_radians(currentangle), 1., 0.5);
            Self::draw_petal_cp1(poles, i , curhue);

        }
    }
    
    pub fn draw_petal_cp1(poles: &mut [super::Pole],index : usize, c : palette::Hsl) {
 
            // first = 1p, second = 1m, third = 2p; petal = 1m + 2p
            let string1 = index * 2;
            let string2 = if index == 0 { NUM_POLES - 1 } else { string1 - 1 };

            {
                let pole1 = poles[string1].leds_cp1();
                for p in pole1.iter_mut() {
                    *p = c;
                }
            }
            {
                let pole2 = poles[string2].leds_cp1();
                for p in pole2.iter_mut() {
                    *p = c;
                }
            }   
}

pub fn draw_petal_cp2(poles: &mut [super::Pole],index : usize, c : palette::Hsl) {
 
            // first = 1p, second = 1m, third = 2p; petal = 1m + 2p
            let string1 = index * 2;
            let string2 = match index {
                0...1 => NUM_POLES + string1 - 3,
                _ => string1 - 3,
            };

            {
                let pole1 = poles[string1].leds_cp2();
                for p in pole1.iter_mut() {
                    *p = c;
                }
            }
            {
                let pole2 = poles[string2].leds_cp2();
                for p in pole2.iter_mut() {
                    *p = c;
                }
            }   
}

    pub fn animate_hearts(&mut self,
                         poles: &mut [super::Pole],
                         delta: std::time::Duration) {
        let val = self.heart_phase.cyclic_update(delta);

        let breath_phase = if val <= 0.5 {val} else {2.*0.5-val};
        let breath_phase = breath_phase * 2.0;

        for i in 0..NUM_POLES {

            let mut curhue = poles[i].base_color;
            curhue.lightness = 0.1+0.5*breath_phase;
            for p in  poles[i].heart().iter_mut() {
             *p = curhue;
            }
        }
    }

    pub fn animate_poles(&mut self,
                         poles: &mut [super::Pole],
                         touches: &super::TouchMap,
                         delta: std::time::Duration) {
        use animations::touch::SinglePoleAnimation;


        // update sprites
        for sprite in self.sprites.iter_mut().chain(self.backgroundsprites.iter_mut()) {
            sprite.update(delta);
        }


        // because of borrow checker i can't pass self, so pass this temp vector instead.
        let mut newsprites: Vec<Box<PoleAnimation>> = vec![];
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
        let mut oscmessages: Vec<super::osc::OSCEvent> = vec![];
        for (i, row) in touches.touches.iter().enumerate() {
            oscmessages.truncate(0);
            let mut current_touches: bit_set::BitSet =
                row.iter().enumerate().filter_map(|(ind, &e)| e.map(|_| ind)).collect();

            let is_self_touching = current_touches.remove(i);

            // if they are both empty
            let (old_state, new_state) = {

                let cur_pole = &mut poles[i];
                let new_state = {
                    if !is_self_touching && current_touches.is_empty() {
                        super::PoleState::NotTouched
                    } else if is_self_touching && current_touches.is_empty() {
                        super::PoleState::Touched
                    } else {
                        super::PoleState::ConnectedTo(current_touches)
                    }
                };
                (std::mem::replace(&mut cur_pole.state, new_state.clone()), new_state)
            };


            let old_anim = poles[i].anim;
            let mut new_anim = old_anim;

            let mut sound_state_changed = false;

            //            if self.transition(i, &mut oscmessages, &old_state, &new_state) {
            if old_state != new_state {
                // change animation
                new_anim = match new_state {
                    super::PoleState::NotTouched => {
                        sound_state_changed = true;
                        None
                    }
                    super::PoleState::Touched => {
                        sound_state_changed = true;
                        Some(super::PoleAnimations::Touching)
                    }
                    // if one of the other poles in the change has level == 1 then exploding
                    super::PoleState::ConnectedTo(_) => {
                        /* TODO: re calc colors */
                        match old_state {
                            super::PoleState::ConnectedTo(_) => {}
                            _ => {
                                sound_state_changed = true;
                            }
                        };

                        Some(super::PoleAnimations::Connecting)
                        // if old state != connect; should send = true
                    }
                };
            }


            // do this every frame.
            if let super::PoleState::ConnectedTo(ref others) = new_state {
                let was_exploding = poles[i].anim == Some(super::PoleAnimations::Exploding);
                let is_exploding = poles[i].level == 1. &&
                                   others.iter().any(|i| poles[i].level == 1.);
                if is_exploding {
                    new_anim = Some(super::PoleAnimations::Exploding);
                }
                if was_exploding != is_exploding {
                    // TODO: send midi signal should_send = true;
                    sound_state_changed = true;

                    if is_exploding {
                        self.sprites.push(Box::new(idle::ExplosionAnim::new()))
                    }
                }
            }

            poles[i].anim = new_anim;

            if sound_state_changed {
                self.osc.update_sound(i, old_anim, new_anim);
            }

        }

        //send midi



        // if out all that is toched is the pole being touched:

        for pole in poles.iter_mut() {

            // animate pole
            let old_level = pole.level;
            let old_touch_level = pole.touch_level;
            match pole.anim {
                Some(super::PoleAnimations::Touching) => {
                    self::touch::TouchAnim::animate_pole(pole, delta);
                }
                Some(super::PoleAnimations::Connecting) => {
                    self::touch::ConnectedAnim::animate_pole(pole, delta);
                }
                Some(super::PoleAnimations::Exploding) => {
                    self::touch::ExplodingAnim::animate_pole(pole, delta);
                }
                None => {
                    self::touch::ReverseTouchAnim::animate_pole(pole, delta);
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

            // draw pole animation
            let ledslen = pole.leds().len();
            if pole.level > 0. {
                let circl_index: usize = (pole.level * ledslen as f32) as usize;
                let color = pole.base_color;


                for pixel in pole.leds().iter_mut().rev().take(circl_index) {
                    *pixel = color;
                }
            }
            if pole.touch_level > 0. {
                let circl_index: usize = (pole.touch_level * ledslen as f32) as usize;
                let color = pole.current_color;

                for pixel in pole.leds_cp1().iter_mut().take(circl_index) {
                    *pixel = color;
                }
            }
            // TODO: draw heart animation

        }


        // output sprites
        self.sprites.retain(|ref x| !x.is_done());
        for sprite in self.sprites.iter() {
            sprite.animate_poles(poles);
        }


    }


    fn transition(&self,
                  i: usize,
                  generatedevents: &mut Vec<super::osc::OSCEvent>,
                  old_state: &super::PoleState,
                  new_state: &super::PoleState)
                  -> bool {

        match *old_state {
            super::PoleState::NotTouched => {
                match *new_state {
                    super::PoleState::NotTouched => false,
                    super::PoleState::Touched => {
                        /* send midi! */
                        generatedevents.push(super::osc::OSCEvent::Touch(i));
                        true
                    }
                    super::PoleState::ConnectedTo(_) => {
                        /* send midi! */
                        generatedevents.push(super::osc::OSCEvent::Touch(i));
                        generatedevents.push(super::osc::OSCEvent::Riser(i));
                        true
                    }
                }
            }
            super::PoleState::Touched => {
                match *new_state {
                    super::PoleState::NotTouched => {
                        /* send midi!  both high and low off if possible.*/
                        true
                    }
                    super::PoleState::Touched => false,
                    super::PoleState::ConnectedTo(_) => {
                        /* send midi! */
                        true
                    }
                }
            }
            super::PoleState::ConnectedTo(ref x) => {
                match *new_state {
                    super::PoleState::NotTouched => {
                        /* send midi! */
                        true
                    }
                    super::PoleState::Touched => {
                        /* send midi! */
                        true
                    }
                    super::PoleState::ConnectedTo(ref y) => x != y, 
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

pub trait Animation {
    fn update_animation(&mut self, delta: std::time::Duration);
}

pub trait Drawer {
    fn draw(&self, array: &mut super::pixels::LedArray);
}
