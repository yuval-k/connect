use std;

use tk_opc::OpcMessage;
use tk_opc::OpcMessageData;
use tk_opc::Pixels;
use std::io::Write;

pub struct OPCLedArray {
    pixels: Pixels,
    sender: std::sync::mpsc::Sender<OpcMessage>,
}

impl OPCLedArray {
    pub fn new(size: usize, address: &str) -> std::io::Result<Self> {
        let mut stream = std::net::TcpStream::connect(address)?;
        let (sender, receiver) = std::sync::mpsc::channel::<OpcMessage>();
        std::thread::spawn(move || for msg in receiver.into_iter() {
            let msg: Vec<u8> = msg.into();
            stream.write_all(&msg);
        });

        Ok(OPCLedArray {
            pixels: Pixels::new(size),
            sender: sender,
        })

    }
}

impl super::anim::LedArray for OPCLedArray {
    fn len(&self) -> usize {
        self.pixels.iter().len()
    }

    fn set_color_rgba(&mut self, lednum: usize, r: u8, g: u8, b: u8, a: u8) {
        if let Some(mut pixel) = self.pixels.iter_mut().nth(lednum) {
            pixel.set_r(r);
            pixel.set_g(g);
            pixel.set_b(b);
        }
    }

    fn show(&mut self) -> std::io::Result<()> {
        let pixels = self.pixels.clone();

        self.sender
            .send(OpcMessage::new(0, OpcMessageData::SetPixelColours(pixels)))
            .expect("Sending msg failed!");
        Ok(())
    }
}
