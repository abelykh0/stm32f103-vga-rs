#![forbid(unsafe_code)]

use crate::vga::{HSIZE_CHARS, VSIZE_CHARS};

use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;
use embedded_graphics::pixelcolor::BinaryColor;

// VGA display
pub struct VgaDisplay {
    pub pixels: [u8; (HSIZE_CHARS * 8 * VSIZE_CHARS) as usize],
    pub attributes : [u8; (HSIZE_CHARS * VSIZE_CHARS) as usize],
    pub attribute_definitions : [u8; 320]
}

impl VgaDisplay {

    pub const fn new() -> VgaDisplay
    {
        VgaDisplay {
            pixels : [0; (HSIZE_CHARS * 8 * VSIZE_CHARS) as usize],
            attributes : [0; (HSIZE_CHARS * VSIZE_CHARS) as usize],
            attribute_definitions : [0; 320]
        }
    }
    
    pub fn init_default_attribute(&mut self, back_color : u8, fore_color : u8)
    {
        for i in 0..16 {
            let mut value = i;
            let mut index = i << 2;
            for _bit in 0..4 {
                self.attribute_definitions[index] = if value & 0x08 == 0 { back_color } else { fore_color };
                value <<= 1;
                index += 1;
            }
        }
    }

    fn write_pixel(&mut self, x: u16, y: u16, val: BinaryColor) {
        if x >= HSIZE_CHARS * 8 || y >= VSIZE_CHARS * 8 {
            return
        }

        let bit = x & 0x07;
        let byte = x >> 3;

        if val == BinaryColor::Off {
            self.pixels[(y * HSIZE_CHARS + byte) as usize] &= !(0x80 >> bit);
        } else {
            self.pixels[(y * HSIZE_CHARS + byte) as usize] |= 0x80 >> bit;
        }
    }
}

impl Drawing<BinaryColor> for VgaDisplay {
    fn draw<T>(&mut self, item_pixels: T)
        where T: IntoIterator<Item = Pixel<BinaryColor>>,
    {
        for pixel in item_pixels {
            let point = pixel.0;
            self.write_pixel(point.x as u16, point.y as u16, pixel.1);
        }
    }
}
