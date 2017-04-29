use rosc;

struct OSCManager;

#[derive(Copy,Clone,Debug)]
enum OSCEvent {
    Touch(bool),
    HiTouch(bool),
    Riser(bool),
    Explosion,
}

// execute immediately
const MIN_VALUE : rosc::OscType = rosc::OscType::Time(0,1);

impl OSCManager {

fn to_osc_msg(&mut self, poleindex : usize, states : & [OSCEvent]) ->  rosc::OscPacket  {
    // Go over pole state. if high touch on is requested send touch off; if untouch happens, check
    // if it is high touch or low touch

    let packets : Vec<rosc::OscPacket> = states.iter().map(|state|{
    let msg = match *state {
        OSCEvent::Touch(true) => {
            rosc::OscMessage{
                addr : "touch".to_string(), 
                args : Some(vec![rosc::OscType::Int(poleindex as i32)]),
                }
        }
        OSCEvent::Touch(false) => {
            rosc::OscMessage{
                addr : "untouch".to_string(), 
                args : Some(vec![rosc::OscType::Int(poleindex as i32)]),
                }
        }

        _ => panic!("What?!")
    };
    rosc::OscPacket::Message(msg)
    }).collect();

    rosc::OscPacket::Bundle(rosc::OscBundle {
    timetag: MIN_VALUE,
    content: packets
    })

}

}   