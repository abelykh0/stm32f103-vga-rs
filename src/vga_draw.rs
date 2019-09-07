pub fn vga_draw(pix: &[u8], attr_base: &[u8], attr: &[u8]) {
    unsafe {
        vga_draw_impl(pix.as_ptr(), attr_base.as_ptr(), attr.as_ptr(), 0x4001080C as _)
    }
}

extern "C" {
    fn vga_draw_impl(pix: *const u8, attr_base: *const u8, attr: *const u8, odr: *const u8);
}
