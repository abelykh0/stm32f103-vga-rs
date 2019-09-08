use crate::vga::HSIZE_CHARS;
use crate::vga::display::VgaDisplay;

pub struct VgaDraw {
    pub pixels_ptr: u32,
    pub attributes_ptr : u32,
    pub attribute_definitions_ptr : u32
}

impl VgaDraw {
    pub fn init(&mut self, vga_display : &VgaDisplay) {
        self.pixels_ptr = vga_display.pixels.as_ptr() as u32;
        self.attributes_ptr = vga_display.attributes.as_ptr() as u32;
        self.attribute_definitions_ptr = vga_display.default_attribute.as_ptr() as u32;
    }

    pub fn draw(&self, vline : u32) {
        unsafe {
            vga_draw_impl(
                self.pixels_ptr + vline * HSIZE_CHARS as u32,
                self.attribute_definitions_ptr,
                self.attributes_ptr + vline / 8 * HSIZE_CHARS as u32,
                0x4001080C as _)
        }
    }
}

extern "C" {
    fn vga_draw_impl(pix: u32, attr_base: u32, attr: u32, odr: u32);
}
