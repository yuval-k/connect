use std;
use std::net::UdpSocket;
use rosc;

use super::{Events, Modes};

struct ConfigData {
    num_leds_for_pole: usize,
    cp1: usize,
    cp2: usize,

    heart: std::ops::Range<usize>
}

impl ConfigData {
    fn new() -> Self {
        ConfigData {
            num_leds_for_pole: 54,
            cp1: 20,
            cp2: 34,
            heart: 54..(54+14),
        }
    }
}

pub struct Config {
    data: std::sync::Arc<std::sync::RwLock<ConfigData>>,
    configfile : std::path::PathBuf,
}

impl Config {
    pub fn new(configfile : &std::path::Path,sender: std::sync::mpsc::Sender<Events>) -> Self {
        let configdata = std::sync::Arc::new(std::sync::RwLock::new(ConfigData::new()));
        let mut s = Config { data: configdata.clone() , configfile : std::path::PathBuf::from(configfile)};

        if s.configfile.exists() {
            s.load_config();
        }

        // generate config change event for the initial config
        sender.send(Events::ConfigChanged);

        std::thread::spawn(move || Self::start_config_server(sender, configdata));
        s
    }
    
    fn load_config(&mut self) {
        unimplemented!()
    }
    pub fn save_config(&mut self) {
        unimplemented!()
    }

    pub fn get_num_leds_for_pole(&self) -> usize {
        let data = self.data.read().unwrap();
        let numret = data.num_leds_for_pole;
        numret
    }

    pub fn get_cp1(&self) -> usize {
        self.data.read().unwrap().cp1
    }

    pub fn get_cp2(&self) -> usize {
        self.data.read().unwrap().cp2
    }

    pub fn get_heart(&self) -> std::ops::Range<usize> {
        self.data.read().unwrap().heart.clone()
    }

    fn start_config_server(sender: std::sync::mpsc::Sender<Events>,
                           mut data: std::sync::Arc<std::sync::RwLock<ConfigData>>) {
        let mut socket = UdpSocket::bind("0.0.0.0:8134").expect("this must work");
        info!("osc config server up");
        let mut buf = [0; 4096];
        loop {
            let res = socket.recv_from(&mut buf);
            if res.is_err() {
                // TODO log
                continue;
            }
            let (amt, src) = res.unwrap();
            let buf = &mut buf[..amt];

            let res = rosc::decoder::decode(&buf);
            if res.is_err() {
                // TODO log
                continue;
            }
            let msg = res.unwrap();

            debug!("got osc packet {:?}", &msg);
            Self::process(&mut data, &sender, msg);
        }

    }


    fn process(data: &mut std::sync::Arc<std::sync::RwLock<ConfigData>>,
               sender: &std::sync::mpsc::Sender<Events>,
               p: rosc::OscPacket) {
        match p {
            rosc::OscPacket::Message(m) => {
                Self::process_message(data, sender, m);
            }
            rosc::OscPacket::Bundle(b) => {
                // we ignore time tag. sorry.
                for inner in b.content {
                    Self::process(data, sender, inner);
                }
            }
        }
    }

    fn process_message(data: &mut std::sync::Arc<std::sync::RwLock<ConfigData>>,
                       sender: &std::sync::mpsc::Sender<Events>,
                       m: rosc::OscMessage) {

        match (m.addr.as_ref(), m.args) {
            ("/pole_leds", Some(ref args)) if args.len() == 1 => {
                let arg = &args[0];
                match *arg {
                    rosc::OscType::Int(num) => {
                        if num >= 0 {
                            data.write().unwrap().num_leds_for_pole = num as usize;
                        }
                    }
                    rosc::OscType::Float(num) => {
                        if num >= 0.0 {
                            data.write().unwrap().num_leds_for_pole = num as usize;
                        }
                    }
                    _ => {
                        warn!("got unexpect arg {:?}", *arg);
                        return;
                    }
                }

                sender.send(Events::ConfigChanged);
            }

            ("/flower", Some(ref args)) if args.len() == 1 => {
                let arg = &args[0];
                let enabled = Self::to_bool(arg);
                if let Some(enabled) = enabled {
                    if enabled {
                        sender.send(Events::ModeChanged(Modes::Flower));
                    } else {
                        sender.send(Events::ModeChanged(Modes::Regular));
                    }
                } else {
                    warn!("got unexpect argument {:?}", *arg);
                }
            }
            ("/mode/disco", _) => {
                sender.send(Events::ModeChanged(Modes::Disco));
            }

            ("/mode/reg", _) => {
                sender.send(Events::ModeChanged(Modes::Regular));
            }

            ("/mode/flower", _) => {
                sender.send(Events::ModeChanged(Modes::Flower));
            }
            ("/saveconfig", _) => {
                sender.send(Events::SaveConfig);
            }
            _ => {warn!("got unexpected msg {:?}", m.addr);}
        }
    }

    fn to_bool(t: &rosc::OscType) -> Option<bool> {
        match *t {
            rosc::OscType::Int(num) => Some(if num != 0 { true } else { false }),
            rosc::OscType::Float(num) => Some(if num != 0.0 { true } else { false }),
            rosc::OscType::Bool(b) => Some(b),
            _ => None,
        }
    }
}
