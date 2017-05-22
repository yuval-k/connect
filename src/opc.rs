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
    pub fn new(size: usize, address: &str) -> Self {
        let address = address.to_string();
        let (sender, receiver) = std::sync::mpsc::channel::<OpcMessage>();
        std::thread::spawn(move || loop {
            Self::work(&receiver, &address);
            std::thread::sleep(std::time::Duration::from_secs(1));
        });

        OPCLedArray {
            pixels: Pixels::new(size),
            sender: sender,
        }

    }

    fn work(receiver: &std::sync::mpsc::Receiver<OpcMessage>,
            address: &str)
            -> std::io::Result<()> {

        let mut stream = std::net::TcpStream::connect(address)?;
        for msg in receiver.iter() {
            // TODO: write first to vector and use tcp no delay?!
            let header = msg.header().to_bytes();
            // TODO implement reconnection logic / change to tokio
            stream.write_all(&header)?;
            let msg: Vec<u8> = msg.message.into();
            stream.write_all(&msg)?;
        }
        Ok(())
    }
}

impl super::pixels::LedArray for OPCLedArray {
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
