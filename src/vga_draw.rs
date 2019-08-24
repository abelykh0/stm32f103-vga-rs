extern "C" {
    fn vga_draw(font: *const u8, characters: *const u8, attributes: *const u32, dest: *const u8);
}
