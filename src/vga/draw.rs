use crate::vga::{HSIZE_CHARS, VSIZE_CHARS};
use crate::vga::display::VgaDisplay;

pub struct VgaDraw {
    pixels_ptr: u32,
    attributes_ptr : u32,
    attribute_definitions_ptr : u32,
    vline: u32, /* The current line being drawn */
    vdraw: u32, /* Used to increment vline every 2 drawn lines */
    vflag: bool /* When true, can draw on the screen */
}

impl VgaDraw {
    pub const fn new() -> VgaDraw {
        VgaDraw {
            pixels_ptr : 0,
            attributes_ptr : 0,
            attribute_definitions_ptr : 0,
            vline : 0,
            vdraw : 0,
            vflag : false
        }
    }

    pub fn init(&mut self, vga_display : &VgaDisplay) {
        self.pixels_ptr = vga_display.pixels.as_ptr() as u32;
        self.attributes_ptr = vga_display.attributes.as_ptr() as u32;
        self.attribute_definitions_ptr = vga_display.default_attribute.as_ptr() as u32;
    }

    // Call this from TIM3 interrupt handler
    #[inline(always)]
    pub fn on_hsync(&mut self) {
        if self.vflag {
            unsafe {
                vga_draw_impl(
                    self.pixels_ptr + self.vline * HSIZE_CHARS as u32,
                    self.attribute_definitions_ptr,
                    self.attributes_ptr + self.vline / 8 * HSIZE_CHARS as u32,
                    0x4001080C as _)
            }

            self.vdraw += 1;
            if self.vdraw >= 2 {
                self.vdraw = 0;
                self.vline += 1;
                if self.vline == VSIZE_CHARS as u32 * 8 {
                    self.vline = 0;
                    self.vdraw = 0;
                    self.vflag = false;
                }
            }
        }
    }

    // Call this from TIM4 interrupt handler
    #[inline(always)]
    pub fn on_vsync(&mut self) {
        self.vline = 0;
        self.vflag = true;
    }
}

extern "C" {
    fn vga_draw_impl(pix: u32, attr_base: u32, attr: u32, odr: u32);
}
