#![no_std]
#![no_main]

//extern crate panic_semihosting;
extern crate panic_halt;

mod stm32;
mod vga;
mod keyboard;
use crate::vga::display::VgaDisplay;
use crate::vga::render::VgaDraw;
use crate::keyboard::Ps2Keyboard;

use core::str;
use embedded_graphics::prelude::*;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::pixelcolor::BinaryColor;
use rtfm::app;
use stm32f1::stm32f103 as blue_pill;
use arraydeque::ArrayDeque;
use pc_keyboard::{Keyboard, layouts, ScancodeSet2, HandleControl};

#[app(device = stm32f1::stm32f103)]
const APP: () = {
    // Late resorce binding
    static mut GPIOA: blue_pill::GPIOA = ();
    static mut GPIOC: blue_pill::GPIOC = ();
    static mut TIM2: blue_pill::TIM2 = ();
    static mut TIM3: blue_pill::TIM3 = ();
    static mut TIM4: blue_pill::TIM4 = ();

    // VGA
    static mut DISPLAY : VgaDisplay = VgaDisplay {
        pixels : [0; (vga::HSIZE_CHARS * 8 * vga::VSIZE_CHARS) as usize],
        attributes : [0; (vga::HSIZE_CHARS * vga::VSIZE_CHARS) as usize],
        attribute_definitions : [0; 320]
    };
    static mut VGA_DRAW : VgaDraw = VgaDraw::new();

    // PS/2 Keyboard
    static mut KEYBOARD : Ps2Keyboard = ();

    #[init (resources = [DISPLAY, VGA_DRAW])]
    fn init() -> init::LateResources {
        // Configure PLL and flash
        stm32::configure_clocks(&device.RCC, &device.FLASH);

        // Configures the system timer to trigger a SysTick exception every second
        //stm32::configure_systick(&mut core.SYST, 72_000); // period = 1ms

        // Built-in LED is on GPIOC, pin 13
        device.RCC.apb2enr.modify(|_r, w| w.iopcen().set_bit());
        device.GPIOC.crh.modify(|_r, w| { w
            .mode13().output50()
            .cnf13().push_pull()
        });

        // This is used to display 64 colors
        for i in 0..64 {
            for j in 0..4 {
                resources.DISPLAY.attribute_definitions[(i << 2) + 64 + j] = convert_color(i as u8);
            }
        }

        // Initialize VGA
        resources.DISPLAY.init_default_attribute(convert_color(0x10), convert_color(0x3F));
        resources.VGA_DRAW.init(&resources.DISPLAY);
        vga::render::init_vga(&device);

        init::LateResources { 
            GPIOA: device.GPIOA,
            GPIOC: device.GPIOC,
            TIM2: device.TIM2,
            TIM3: device.TIM3,
            TIM4: device.TIM4,
            KEYBOARD : Ps2Keyboard {
                queue : ArrayDeque::new(),
                pc_keyboard : Keyboard::new(layouts::Us104Key, ScancodeSet2, HandleControl::MapLettersToUnicode)
            }
        }
    }

    #[idle (resources = [TIM2, TIM4, DISPLAY, GPIOC])]
    fn idle() -> ! {
        resources.DISPLAY.draw(
            Rectangle::new(Point::new(2, 2), Point::new(vga::HSIZE_CHARS as i32 * 8 - 3, vga::VSIZE_CHARS as i32 * 8 - 3)).stroke(Some(BinaryColor::On))
        );
        resources.DISPLAY.draw(
            Rectangle::new(Point::new(4, 4), Point::new(vga::HSIZE_CHARS as i32 * 8 - 5, vga::VSIZE_CHARS as i32 * 8 - 5)).stroke(Some(BinaryColor::On))
        );

        for i in 0..64 {
            let mut buffer = [0u8; 6];
            let color = format_color(i as u8, &mut buffer);
            resources.DISPLAY.draw(
                Font6x8::render_str(color)
                .stroke(Some(BinaryColor::On))
                .translate(Point::new(16 + (i % 6) * 56, 41 + (i / 6) * 16))
            );

            let x_position = (2 + (i % 6) * 7) as usize;
            let y_position = (6 + (i / 6) * 2) as usize;
            let offset1 = (1 + (i >> 4)) as u8;
            let mut offset2 = (i & 0x0F) as u8;
            offset2 |= offset2 << 4;
            for j in 0..5 {
                for y in 0..8 {
                    resources.DISPLAY.pixels[(y_position * 8 + y) * vga::HSIZE_CHARS as usize + x_position + j as usize] = offset2;
                }
                resources.DISPLAY.attributes[y_position * vga::HSIZE_CHARS as usize + x_position + j as usize] = offset1;
            }
        }

        loop {
        }
    }

    #[interrupt (priority = 16, resources = [TIM4, VGA_DRAW])]
    fn TIM4() 
    {
        // Acknowledge IRQ
        resources.TIM4.sr.modify(|_, w| w.cc4if().clear_bit());

        resources.VGA_DRAW.on_vsync();
    }

    #[interrupt (priority = 16, resources = [TIM3, VGA_DRAW])]
    fn TIM3() 
    {
        // Acknowledge IRQ
        resources.TIM3.sr.modify(|_, w| w.cc2if().clear_bit());

        resources.VGA_DRAW.on_hsync();
    }

    #[interrupt (priority = 15, resources = [TIM2])]
    fn TIM2() 
    {
        // Acknowledge IRQ
        resources.TIM2.sr.modify(|_, w| w.cc2if().clear_bit());

        // Idle the CPU until an interrupt arrives
        cortex_m::asm::wfi();
    }
};

fn format_color(color : u8, buffer: &mut [u8]) -> &str {
    let mut c = color;
    for i in 0..6 {
        if c & 0x20 == 0 {
            buffer[i] = b'0';
        }
        else {
            buffer[i] = b'1';
        }
        c <<= 1;
    }

    unsafe {
        str::from_utf8_unchecked(buffer)
    }
}

#[cfg(feature = "board2")]
fn convert_color(color : u8) -> u8 {
    (color & 0x03) | ((color << 2) & 0xF0)
}
#[cfg(not(feature = "board2"))]
fn convert_color(color : u8) -> u8 {
    color
}