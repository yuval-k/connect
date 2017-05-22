use std;
use std::net::UdpSocket;
use rosc;

use super::Events;

struct ConfigData {
    num_leds_for_pole: usize,
}

impl ConfigData {
    fn new() -> Self {
        ConfigData { num_leds_for_pole: 54 }
    }
}

pub struct Config {
    data: std::sync::Arc<std::sync::RwLock<ConfigData>>,
}

impl Config {
    pub fn new(sender: std::sync::mpsc::Sender<Events>) -> Self {
        let configdata = std::sync::Arc::new(std::sync::RwLock::new(ConfigData::new()));
        let s = Config { data: configdata.clone() };
        std::thread::spawn(move || Self::start_config_server(sender, configdata));
        s
    }

    pub fn get_num_leds_for_pole(&self) -> usize {
        let data = self.data.read().unwrap();
        let numret = data.num_leds_for_pole;
        numret
    }

    fn start_config_server(sender: std::sync::mpsc::Sender<Events>, mut data: std::sync::Arc<std::sync::RwLock<ConfigData>>) {
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


    fn process(data: &mut std::sync::Arc<std::sync::RwLock<ConfigData>>, sender: &std::sync::mpsc::Sender<Events>, p : rosc::OscPacket ) {
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

    fn process_message(data: &mut std::sync::Arc<std::sync::RwLock<ConfigData>>,  sender: &std::sync::mpsc::Sender<Events>, m : rosc::OscMessage) {

        match (m.addr.as_ref(), m.args) {
            ("/pole_leds", Some(ref args) ) if args.len() == 1 => {
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
                        warn!("got unexpect message {:?}", *arg);
                        return;
                    }
                }

                sender.send(Events::ConfigChanged);
            }

            ("/disco", Some(ref args) ) if args.len() == 1 => {
                let arg = &args[0];
               let enabled = match *arg {
                    rosc::OscType::Int(num) => {
                        if num > 0 {
                            true
                        } else {
                            false
                        }
                    }
                    rosc::OscType::Float(num) => {
                        if num > 0.0 {
                            true
                        } else {
                            false
                        }
                    }
                    _ => {
                        warn!("got unexpect message {:?}", *arg);
                        return;
                    }
                };

                sender.send(Events::Disco(enabled));
            }
            _  => {
                
            }
        }
    }
}
