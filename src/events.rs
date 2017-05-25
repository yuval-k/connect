use std;
use super::{EventTypes, Events, Modes, NUM_POLES};
use serial;
use std::io::BufRead;
use serial::SerialPort;
use std::net::UdpSocket;
use rosc;

pub trait Eventer: std::marker::Send {
    fn get_events(&mut self, sender: std::sync::mpsc::Sender<Events>);

    fn get_timeout(&self) -> std::time::Duration;
}

struct StdinEventSource;


impl Eventer for StdinEventSource {
    fn get_events(&mut self, mut sender: std::sync::mpsc::Sender<Events>) {
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        let mut disco = false;
        loop {

            let mut buffer = String::new();
            handle.read_line(&mut buffer).expect("can't read");

            let buffer = buffer.trim();

            let mode = match buffer.as_ref()  {
                "disco" => Some(Modes::Disco),
                "flower" => Some(Modes::Flower),
                "reg" | "regular" => Some(Modes::Regular),
                _ => None,
            };
            if let Some(mode) = mode {
                sender.send(Events::ModeChanged(mode));
                continue;
            }

            let mut words: Vec<&str> = buffer.split_whitespace().collect();


            if (words.len() != 2) && (words.len() != 3) {
                println!("invalid input - exactly two or three!");
                continue;
            }
            let mut stop = false;
            if words.len() == 3 {
                words.remove(0);
                stop = true;
            }
            let word1 = words[0];
            let word2 = words[1];
            let (pole1, pole2) = (word1.parse::<usize>(), word2.parse::<usize>());

            //            let ourtimeout = std::time::Duration::from_secs(1000);
            match (stop, pole1, pole2) {
                (false, Ok(p1), Ok(p2)) => {
                    println!("sending touch event {} {}", p1, p2);
                    sender.send(Events::Start(EventTypes::Connect(p1, p2)));
                }
                (true, Ok(p1), Ok(p2)) => {
                    println!("sending stop touch event {} {}", p1, p2);
                    sender.send(Events::Stop(EventTypes::Connect(p1, p2)));
                }
                _ => {
                    println!("invalid input! - two numbers please");
                }
            }

        }
    }

    fn get_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1000)
    }
}

struct SerialEventSource {
    devicefile: String,
}

impl SerialEventSource {
    fn new(devicefile: &str) -> Self {
        SerialEventSource { devicefile: devicefile.to_string() }
    }
}

impl Eventer for SerialEventSource {
    fn get_events(&mut self, mut sender: std::sync::mpsc::Sender<Events>) {
        loop {
            let err = self.eventloop(&mut sender);
            warn!("Event loop returned unxpectedly {:?}", err);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    fn get_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1000)
    }
}

impl SerialEventSource {
    #[cfg(target_os = "linux")]
    fn auto_detect() -> String {
        // /dev/ttyACM* or /dev/ttyUSB*
        let dir = std::path::Path::new("/dev/");
        let res = std::fs::read_dir(dir);
        if let Ok(readir) = res {
            for entry in readir {
                if let Ok(entry) = entry {
                    if let Ok(meta) = entry.metadata() {
                        if !meta.is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                if name.starts_with("ttyACM") || name.starts_with("ttyUSB") {
                                    if let Some(path) = entry.path().to_str() {
                                        return path.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }


        }

        String::new()
    }


    #[cfg(not(target_os = "linux"))]
    fn auto_detect() -> String {
        panic!("Must provide serial device file name; for testing use filename \"stdin\" to get \
                interactive input");
    }

    fn eventloop(&mut self, sender: &mut std::sync::mpsc::Sender<Events>) -> std::io::Result<()> {

        let mut devicefile = self.devicefile.clone();
        if devicefile.is_empty() {
            devicefile = Self::auto_detect();
        }

        if devicefile.is_empty() {
            use std::io::{Error, ErrorKind};
            return Err(Error::new(ErrorKind::NotFound, "no serial device found"));
        }

        info!("Found serial device {}", devicefile);

        let mut port = serial::open(&devicefile)?;
        port.reconfigure(&|settings| {
                settings.set_baud_rate(serial::Baud115200)?;
                settings.set_char_size(serial::Bits8);
                settings.set_parity(serial::ParityNone);
                settings.set_stop_bits(serial::Stop1);
                settings.set_flow_control(serial::FlowNone);
                Ok(())
            })?;

        port.set_timeout(std::time::Duration::from_secs(10))?;

        let mut reader = std::io::BufReader::new(port);

        let mut line = String::new();

        let mut events: [[[bool; NUM_POLES]; NUM_POLES]; 2] = [[[false; NUM_POLES]; NUM_POLES]; 2];
        let mut currentindex: usize = 0;
        let mut pastindex: usize = 1;

        sender.send(Events::Reset);

        loop {
            line.clear();
            reader.read_line(&mut line)?;

            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with("#") {
                continue;
            }

            let indexes: Result<Vec<usize>, _> =
                line.split(':').map(|s| s.parse::<usize>()).collect();

            debug!("serial line: {}", line);
            match indexes {
                Err(e) => {
                    warn!("error parsing serial line {:?}", e);
                    continue;
                }
                Ok(v) => {
                    if v.len() >= 2 {
                        let lastheardof = std::time::Duration::from_millis(v[0] as u64);
                        let senderindex = v[1];
                        let touching = &v[2..];
                        // TODO: add timeout
                        let secs = lastheardof.as_secs();
                        if (secs < 10) && (secs > 0) {
                            warn!("Pole has a long timeout {} {:?}", senderindex, lastheardof);
                        }

                        Self::set_events(&mut events[currentindex], senderindex, touching);

                        if senderindex == (NUM_POLES - 1) {
                            Self::send_events(sender, &events[pastindex], &events[currentindex]);

                            std::mem::swap(&mut currentindex, &mut pastindex);

                            for e in events[currentindex].iter_mut() {
                                for b in e.iter_mut() {
                                    *b = false;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn set_events(events: &mut [[bool; NUM_POLES]; NUM_POLES],
                  senderindex: usize,
                  touches: &[usize]) {
        if senderindex >= NUM_POLES {
            return;
        }
        for ind in touches.iter().filter(|&x| *x < NUM_POLES) {
            events[senderindex][*ind] = true;
            events[*ind][senderindex] = true;
        }
    }

    fn send_events(sender: &mut std::sync::mpsc::Sender<Events>,
                   pastevents: &[[bool; NUM_POLES]; NUM_POLES],
                   events: &[[bool; NUM_POLES]; NUM_POLES]) {
        for i in 0..NUM_POLES {
            for j in i..NUM_POLES {
                if events[i][j] != pastevents[i][j] {
                    let event = match events[i][j] {
                        false => {
                            debug!("Not Connected({},{})", i, j);
                            Events::Stop(EventTypes::Connect(i, j))
                        }
                        true => {
                            debug!("Connect({},{})", i, j);
                            Events::Start(EventTypes::Connect(i, j))
                        }
                    };
                    sender.send(event);
                }
            }
        }

    }
}

pub fn get_eventer(s: &str) -> Box<Eventer> {
    if s == "stdin" {
        Box::new(StdinEventSource)
    } else {
        Box::new(SerialEventSource::new(s))
    }
}



pub struct UDPEventSource {
    events: [[bool; NUM_POLES]; NUM_POLES],
    socket: UdpSocket,
}

impl Eventer for UDPEventSource {
    fn get_events(&mut self, mut sender: std::sync::mpsc::Sender<Events>) {
        
        info!("osc event server up");
        let mut buf = [0; 4096];
        loop {
            let res = self.socket.recv_from(&mut buf);
            if res.is_err() {
                // TODO log
                continue;
            }
            let (amt, src) = res.unwrap();  
            let buf = &mut buf[..amt];

            debug!("Received udp packet {}",amt);
            

            let res = rosc::decoder::decode(&buf);
            let msg = match res {
                Err(e) => {

                warn!("Error parsing packet udp event {:?}", e);
                continue;
                }
                Ok(msg) => msg,
            };

            debug!("got event osc packet {:?}", &msg);
            self.process(&sender, msg);
        }
    }


    fn get_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1000)
    }
}


impl UDPEventSource {

    pub fn new() -> Self {
        UDPEventSource {
            events: [[false; NUM_POLES];NUM_POLES],
            socket : UdpSocket::bind("0.0.0.0:3134").expect("this must work"),
        }
    }

    fn process(&mut self, sender: &std::sync::mpsc::Sender<Events>,
               p: rosc::OscPacket) {
       match p {
            rosc::OscPacket::Message(m) => {
                self.process_message( sender, m);
            }
            rosc::OscPacket::Bundle(b) => {
                // we ignore time tag. sorry.
                for inner in b.content {
                    self.process( sender, inner);
                }
            }
        }
    }


// /pole_touch
// first arg is int: my id 
// second arg is V{id, map}


    fn process_message(&mut self, sender: &std::sync::mpsc::Sender<Events>,
                       m: rosc::OscMessage) {
        match (m.addr.as_ref(), &m.args) {
            ("/pole_touch", &Some(ref args)) if (args.len() == 1) => {
                let packet = match &args[0] {
                    &rosc::OscType::Blob(ref packet) => packet,
                    _ => {warn!("unexpected pole_touch packet {:?}", args[0]); return;}
                };

                let id = packet[0] as usize;
                let bitmap = &packet[1..4];
                let checksum = packet[4] ;

                let sum : usize =  (id as usize) + bitmap.iter().fold(0usize, |acc : usize, &x| acc + (x as usize));
                let sum : u8 = sum as u8;

                if sum != checksum {
                    warn!("Error in checksum {} != {}", sum, checksum);
                    
                }

                if id >= NUM_POLES {
                    warn!("Invalid id {}", id);
                    return;
                }

                let mut currentstate = [false; NUM_POLES];
                
                for i in 0..NUM_POLES {
                    currentstate[i] = if ((bitmap[i>>3] >> (i&0b111)) & 0b1) != 0 {true} else {false};
                }
                
              //  sender.send(Events:: ) 

            for j in 0..NUM_POLES {
                let past_state = self.events[id][j];
                let transpose_past_state =  
                if j == id {
                    false
                } else {
                    self.events[j][id]
                };

                let event = match (currentstate[j], past_state, transpose_past_state) {
                    (false, true, false)  => {
                            debug!("udp Not Connected({},{})", id, j);
                            Events::Stop(EventTypes::Connect(id, j))
                    }
                    (true, false, false) => {
                            debug!("udp Connect({},{})", id, j);
                            Events::Start(EventTypes::Connect(id, j))
                    }
                    _ => {
                        // nothing to do...
                        continue;
                    }
                };
                sender.send(event);
            }
            self.events[id] = currentstate;

            }
            _ => {warn!("got event unexpected msg {:?}", m);}
        }
    }

}