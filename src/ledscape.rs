use std;
use libc;

pub const LEDSCAPE_NUM_STRIPS: usize = 48;


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

#[link(name = "ledscape", kind = "static")]
extern "C" {

    fn ledscape_init(num_pixels: libc::c_uint) -> *const ::libc::c_void;
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
}

impl Drop for LEDScape {
    fn drop(&mut self) {
        unsafe { ledscape_close(self.h) };
    }
}

impl LEDScape {
    pub unsafe fn new(numpixels_per_strip: usize) -> Self {
        LEDScape {
            h: ledscape_init(numpixels_per_strip as libc::c_uint),
            numpixels_per_strip: numpixels_per_strip,
        }
    }

    // unsafe as frame can potentially be used from multipe threads?!
    pub unsafe fn frame(&mut self, frame_num: FramedIndex) -> LEDScapeFrame {
        let f = ledscape_frame(self.h, frame_num as libc::c_uint);
        LEDScapeFrame { frame: std::slice::from_raw_parts_mut(f, self.numpixels_per_strip) }
    }

    pub fn draw(&mut self, frame_num: FramedIndex) {
        unsafe { ledscape_draw(self.h, frame_num as libc::c_uint) };
    }

    pub fn wait(&mut self) {
        unsafe { ledscape_wait(self.h) };
    }
}
