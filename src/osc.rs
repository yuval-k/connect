use std;
use rosc;
use std::io::Write;


#[derive(Copy,Clone,Debug)]
pub enum OSCEvent {
    UnTouch(usize),
    Touch(usize),
    HiTouch(usize),
    UnHiTouch(usize),
    Riser(usize),
    UnRiser(usize),
    Explosion,
}

// execute immediately
const MIN_VALUE: rosc::OscType = rosc::OscType::Time(0, 1);

const TOUCH_EVENT: &str = "touch";
const UNTOUCH_EVENT: &str = "untouch";

const HI_TOUCH_EVENT: &str = "hitouch";
const HI_UNTOUCH_EVENT: &str = "unhitouch";

const RISER_EVENT: &str = "reiser";
const UNRISER_EVENT: &str = "unreiser";

const EXPLODE_EVENT: &str = "explode";


bitflags! {
    flags SoundState: u32 {
        const Touch       = 1 << 0,
        const HighTouch   = 1 << 1,
        const Riser       = 1 << 2,
    }
}


pub struct OSCManager {
    sender: std::sync::mpsc::Sender<rosc::OscPacket>,
    risers : usize,
    sound_state : SoundState,
}


impl OSCManager {
    pub fn new(addr: &'static str) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || Self::sendmsg(addr, rx));

        OSCManager { sender: tx , risers : 0, sound_state : SoundState::empty()}
    }

    pub fn update_sound(&mut self, i : usize, old_state : Option<super::PoleAnimations>, current_state : Option<super::PoleAnimations>) {
        let current_sound_state = self.sound_state;
        let mut desired_state = SoundState::empty();
        // Cancel old state
        match  current_state {
            None => {/* nothing to do */ },
            Some(super::PoleAnimations::Touching) => {desired_state = Touch;},
            Some(super::PoleAnimations::Connecting) => {desired_state = Riser | Touch ;/* send off to rise event. potentially to touch event as well if new state is none?! */ },
            Some(super::PoleAnimations::Exoloding) => {desired_state = HighTouch;},
        }

        let to_remove = desired_state - current_sound_state;
        let to_add =  current_sound_state - desired_state;
        let is_exploding = (old_state !=  Some(super::PoleAnimations::Exoloding)) && (current_state == Some(super::PoleAnimations::Exoloding));
       
        let mut events : Vec<OSCEvent> = vec![];
        // remove old state
        if to_remove.contains(Touch) {
            events.push(OSCEvent::UnTouch(i));
        }
        if to_remove.contains(HighTouch) {
            events.push(OSCEvent::UnHiTouch(i));
        }
        if to_remove.contains(Riser) {
            self.remove_riser(i).map(|x|events.push(x));
        }

        // explode?
        if is_exploding {
            events.push(OSCEvent::Explosion);
        }

        // apply new state

        if to_add.contains(Touch) {
            events.push(OSCEvent::Touch(i));
        }
        if to_add.contains(HighTouch) {
            events.push(OSCEvent::HiTouch(i));
        }
        if to_add.contains(Riser) {
            self.add_riser(i).map(|x|events.push(x));
        }

        self.sound_state = desired_state;

        // create and send the packet
        let packet = Self::to_osc_msg(&events);

        self.sender.send(packet);

    
    }


    fn add_riser(&mut self, i : usize) -> Option<OSCEvent> {
        self.risers += 1;
        // 2 poles in a chain.. we can notified twice for each
        let risers = self.risers >> 1;

        if risers == 0 {
            return None;
        }
        // no more than 5
        if risers > 5 {
            return None;
        }

        Some(OSCEvent::Riser(risers))

    }


    fn remove_riser(&mut self, i : usize) -> Option<OSCEvent> {
        // 2 poles in a chain.. we can notified twice for each
        let risers = self.risers >> 1;


        if risers == 0 {
            return None;
        }
        self.risers -= 1;
        // no more than 5
        if risers > 5 {
            return None;
        }

        Some(OSCEvent::UnRiser(risers))
    }


    fn sendmsg(addr: &'static str, mut rx: std::sync::mpsc::Receiver<rosc::OscPacket>) {
        loop {
            Self::sconnect(addr, &mut rx);
        }
    }

    fn sconnect(addr: &'static str, rx: &mut std::sync::mpsc::Receiver<rosc::OscPacket>) {
        let mut stream = match std::net::TcpStream::connect(addr) {
            Ok(stream) => stream,
            Err(_) => return,
        };


        if stream.set_nodelay(true).is_err() {
            warn!("set_nodelay call failed");
        }

        for msg in rx.iter() {
            let mut data = match rosc::encoder::encode(&msg) {
                Ok(data) => data,
                Err(_) => return,
            };
            stream.write_all(&data);
        }
    }

    pub fn send_event(&mut self, poleindex: usize, states: &[OSCEvent]) {}

    fn to_osc_msg(states: &[OSCEvent]) -> rosc::OscPacket {
        // Go over pole state. if high touch on is requested send touch off; if untouch happens, check
        // if it is high touch or low touch

        let mut packets: Vec<rosc::OscPacket> = vec![];
        for state in states.iter() {
            let msg = match *state {
                OSCEvent::Touch(i) => {
                    rosc::OscMessage {
                        addr: TOUCH_EVENT.to_string(),
                        args: Some(vec![rosc::OscType::Int(i as i32)]),
                    }
                }
                OSCEvent::UnTouch(i) => {
                    rosc::OscMessage {
                        addr: UNTOUCH_EVENT.to_string(),
                        args: Some(vec![rosc::OscType::Int(i as i32)]),
                    }
                }

                OSCEvent::HiTouch(i) => {
                    rosc::OscMessage {
                        addr: HI_TOUCH_EVENT.to_string(),
                        args: Some(vec![rosc::OscType::Int(i as i32)]),
                    }
                }
                OSCEvent::UnHiTouch(i) => {
                    rosc::OscMessage {
                        addr: HI_UNTOUCH_EVENT.to_string(),
                        args: Some(vec![rosc::OscType::Int(i as i32)]),
                    }
                }
                OSCEvent::Riser(i) => {
                    rosc::OscMessage {
                        addr: RISER_EVENT.to_string(),
                        args: Some(vec![rosc::OscType::Int(i as i32)]),
                    }
                }
                OSCEvent::UnRiser(i) => {
                    rosc::OscMessage {
                        addr: UNRISER_EVENT.to_string(),
                        args: Some(vec![rosc::OscType::Int(i as i32)]),
                    }
                }
                OSCEvent::Explosion => {
                    rosc::OscMessage {
                        addr: EXPLODE_EVENT.to_string(),
                        args: None,
                    }
                }

            };

            packets.push(rosc::OscPacket::Message(msg));
        }

        rosc::OscPacket::Bundle(rosc::OscBundle {
            timetag: MIN_VALUE,
            content: packets,
        })

    }
}
