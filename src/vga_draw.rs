extern "C" {
    pub fn vga_draw_impl(pix: *const u8, attr_base: *const u8, attr: *const u8, odr: *const u8);
}
