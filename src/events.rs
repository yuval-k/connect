use std;
use super::{EventTypes, Events, Modes, NUM_POLES};
use serial;
use std::io::BufRead;
use serial::SerialPort;


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

            if buffer == "disco" {
                disco = !disco;
                if disco {
                    sender.send(Events::ModeChanged(Modes::Disco));
                } else {
                    sender.send(Events::ModeChanged(Modes::Regular));
                }
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
                            let tmp = currentindex;
                            currentindex = pastindex;
                            pastindex = tmp;
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
