use std;
use libc;

use std::io::Write;
use super::pixels;

pub const LEDSCAPE_NUM_STRIPS: usize = 48;

static pru0_program: &'static [u8] = include_bytes!("../lib/bin/ws281x-rgb-123-v3-pru0.bin");
static pru1_program: &'static [u8] = include_bytes!("../lib/bin/ws281x-rgb-123-v3-pru1.bin");

#[repr(C, packed)]
struct ledscape_pixel_t {
    b: u8,
    r: u8,
    g: u8,
    a: u8,
}

#[repr(C, packed)]
struct ledscape_frame_t {
    strip: [ledscape_pixel_t; LEDSCAPE_NUM_STRIPS],
}

type LEDScapeHandle = *const ::libc::c_void;

#[link(name = "prussdrv", kind = "static")]
#[link(name = "ledscape", kind = "static")]
extern "C" {

    fn ledscape_init(num_pixels: libc::c_uint) -> *const ::libc::c_void;
    fn ledscape_init_with_programs(num_pixels: libc::c_uint,
                                   pru0_program_filename: *const libc::c_char,
                                   pru1_program_filename: *const libc::c_char)
                                   -> *const ::libc::c_void;
    fn ledscape_frame(handle: *const ::libc::c_void,
                      frame_num: libc::c_uint)
                      -> *mut ledscape_frame_t;
    fn ledscape_draw(handle: *const ::libc::c_void, frame_num: libc::c_uint);
    fn ledscape_wait(handle: *const ::libc::c_void);
    fn ledscape_close(handle: *const ::libc::c_void);

}

#[derive(Clone,Copy,Debug)]
pub enum FramedIndex {
    FirstFrame = 0,
    SecondFrame = 1,
}

impl FramedIndex {
    pub fn other_frame(&self) -> FramedIndex {
        match *self {
            FramedIndex::FirstFrame => FramedIndex::SecondFrame,
            FramedIndex::SecondFrame => FramedIndex::FirstFrame,
        }
    }
}
pub struct LEDScapeFrame {
    frame: &'static mut [ledscape_frame_t],
}

impl LEDScapeFrame {
    pub fn set_pixel(&mut self, strip_num: usize, pixel_index: usize, r: u8, g: u8, b: u8, a: u8) {
        self.frame[pixel_index].strip[strip_num] = ledscape_pixel_t {
            r: r,
            g: g,
            b: b,
            a: a,
        };
    }
}

pub struct LEDScape {
    h: LEDScapeHandle,
    numpixels_per_strip: usize,
    pru0_program_filename: std::path::PathBuf,
    pru1_program_filename: std::path::PathBuf,
}

impl Drop for LEDScape {
    fn drop(&mut self) {
        unsafe { ledscape_close(self.h) };
        std::fs::remove_file(&self.pru0_program_filename);
        std::fs::remove_file(&self.pru1_program_filename);
    }
}

impl LEDScape {
    pub unsafe fn new(numpixels_per_strip: usize) -> std::io::Result<Self> {
        let tmpdir = std::env::temp_dir();
        let mut pru0_program_file = tmpdir.clone();
        pru0_program_file.push("pru0_program_file");
        {
            let mut f = std::fs::File::create(&pru0_program_file)?;
            f.write_all(pru0_program)?;
        }
        let mut pru1_program_file = tmpdir.clone();
        pru1_program_file.push("pru1_program_file");
        {
            let mut f = std::fs::File::create(&pru1_program_file)?;
            f.write_all(pru1_program)?;
        }

        Ok(Self::new_with_programs(numpixels_per_strip, pru0_program_file, pru1_program_file))

    }

    unsafe fn new_with_programs(numpixels_per_strip: usize,
                                pru0_program_filename: std::path::PathBuf,
                                pru1_program_filename: std::path::PathBuf)
                                -> Self {
        use std::ffi::CString;

        let pru0 = CString::new(&*pru0_program_filename.to_string_lossy()).unwrap();
        let pru1 = CString::new(&*pru1_program_filename.to_string_lossy()).unwrap();

        LEDScape {
            h: ledscape_init_with_programs(numpixels_per_strip as libc::c_uint,
                                           pru0.as_ptr(),
                                           pru1.as_ptr()),
            numpixels_per_strip: numpixels_per_strip,
            pru0_program_filename: pru0_program_filename,
            pru1_program_filename: pru1_program_filename,
        }
    }

    // unsafe as frame can potentially be used from multipe threads?!
    pub unsafe fn frame(&mut self, frame_num: FramedIndex) -> LEDScapeFrame {
        let f = ledscape_frame(self.h, frame_num as libc::c_uint);
        LEDScapeFrame { frame: std::slice::from_raw_parts_mut(f, self.numpixels_per_strip) }
    }

    pub fn draw(&mut self, frame_num: FramedIndex) {
        trace!("drawing frame!");
        unsafe { ledscape_draw(self.h, frame_num as libc::c_uint) };
    }

    pub fn wait(&mut self) {
        unsafe { ledscape_wait(self.h) };
    }
}

pub struct LedscapeLedArray {
    ls: LEDScape,
    strip_size: usize,
    frame_index: FramedIndex,
    frame: LEDScapeFrame,
}

impl LedscapeLedArray {
    pub fn new(strip_size: usize) -> Self {
        let mut ls = unsafe { LEDScape::new(strip_size).expect("can't init ledscape") };
        let frame = unsafe { ls.frame(FramedIndex::FirstFrame) };
        LedscapeLedArray {
            ls: ls,
            strip_size: strip_size,
            frame_index: FramedIndex::FirstFrame,
            frame: frame,
        }
    }
}

impl anim::LedArray for LedscapeLedArray {
    fn len(&self) -> usize {
        self.strip_size * LEDSCAPE_NUM_STRIPS
    }

    fn set_color_rgba(&mut self, lednum: usize, r: u8, g: u8, b: u8, a: u8) {
        let (stripindex, ledindex) = (lednum / self.strip_size, lednum % self.strip_size);
        //    debug!("drawing frame! {} {} : {} {} {} {} ",stripindex, ledindex,r, g, b, a );

        self.frame.set_pixel(stripindex, ledindex, r, g, b, a);
    }

    fn show(&mut self) -> std::io::Result<()> {
        self.ls.wait();
        self.ls.draw(self.frame_index);
        self.frame_index = self.frame_index.other_frame();
        self.frame = unsafe { self.ls.frame(self.frame_index) };
        Ok(())
    }
}
