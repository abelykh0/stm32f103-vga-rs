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
use stm32f1::stm32f103 as blue_pill;
//use pc_keyboard::KeyEvent;

#[rtfm::app(device = stm32f1::stm32f103, peripherals = true)]
const APP: () = {
    struct Resources {
        gpioa: blue_pill::GPIOA,
        gpioc: blue_pill::GPIOC,
        tim2: blue_pill::TIM2,
        tim3: blue_pill::TIM3,
        tim4: blue_pill::TIM4,
        keyboard : Ps2Keyboard,
        #[init(VgaDraw::new())]
        vga_draw : VgaDraw,
        #[init(VgaDisplay::new())]
        display : VgaDisplay
    }

    #[init(resources = [vga_draw, display])]
    fn init(cx: init::Context) -> init::LateResources {
        // Configure PLL and flash
        stm32::configure_clocks(&cx.device.RCC, &cx.device.FLASH);

        // Configures the system timer to trigger a SysTick exception every second
        //stm32::configure_systick(&mut core.SYST, 72_000); // period = 1ms

        // Built-in LED is on GPIOC, pin 13
        cx.device.RCC.apb2enr.modify(|_r, w| w.iopcen().set_bit());
        cx.device.GPIOC.crh.modify(|_r, w| { w
            .mode13().output50()
            .cnf13().push_pull()
        });

        // This is used to display 64 colors
        for i in 0..64 {
            for j in 0..4 {
                cx.resources.display.attribute_definitions[(i << 2) + 64 + j] = convert_color(i as u8);
            }
        }
        cx.resources.display.init_default_attribute(convert_color(0x10), convert_color(0x3F));
        cx.resources.vga_draw.init(&cx.resources.display);
        vga::render::init_vga(&cx.device);

        // Initialize keyboard
        Ps2Keyboard::init(&cx.device);

        init::LateResources { 
            gpioa: cx.device.GPIOA,
            gpioc: cx.device.GPIOC,
            tim2: cx.device.TIM2,
            tim3: cx.device.TIM3,
            tim4: cx.device.TIM4,
            keyboard : Ps2Keyboard::new(),
        }
    }

    #[idle (resources = [tim2, tim4, display, gpioc])]
    fn idle(mut cx: idle::Context) -> ! {
        cx.resources.display.draw(
            Rectangle::new(Point::new(2, 2), Point::new(vga::HSIZE_CHARS as i32 * 8 - 3, vga::VSIZE_CHARS as i32 * 8 - 3)).stroke(Some(BinaryColor::On))
        );
        cx.resources.display.draw(
            Rectangle::new(Point::new(4, 4), Point::new(vga::HSIZE_CHARS as i32 * 8 - 5, vga::VSIZE_CHARS as i32 * 8 - 5)).stroke(Some(BinaryColor::On))
        );

        for i in 0..64 {
            let mut buffer = [0u8; 6];
            let color = format_color(i as u8, &mut buffer);
            cx.resources.display.draw(
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
                    cx.resources.display.pixels[(y_position * 8 + y) * vga::HSIZE_CHARS as usize + x_position + j as usize] = offset2;
                }
                cx.resources.display.attributes[y_position * vga::HSIZE_CHARS as usize + x_position + j as usize] = offset1;
            }
        }

        loop {
            cx.resources.gpioc.bsrr.write(|w| w.bs13().set_bit());
            cortex_m::asm::delay(2000000);
            cx.resources.gpioc.brr.write(|w| w.br13().set_bit());
            cortex_m::asm::delay(2000000);
        }
    }

    #[task(binds = PendSV, resources = [gpioa, keyboard])]
    fn pendSV(cx: pendSV::Context) {
        let gpio_bits = cx.resources.gpioa.idr.read().bits() as u16;
        cx.resources.keyboard.update(gpio_bits);
    }

    #[task(binds = TIM3, priority = 16, resources = [tim3, vga_draw])]
    fn tim3(cx: tim3::Context) 
    {
        // Acknowledge IRQ
        cx.resources.tim3.sr.modify(|_, w| w.cc2if().clear_bit());

        cx.resources.vga_draw.on_hsync();
    }

    #[task(binds = TIM4, priority = 16, resources = [tim4, vga_draw])]
    fn tim4(cx: tim4::Context) 
    {
        // Acknowledge IRQ
        cx.resources.tim4.sr.modify(|_, w| w.cc4if().clear_bit());

        cx.resources.vga_draw.on_vsync();
    }

    #[task(binds = TIM2, priority = 15, resources = [tim2])]
    fn tim2(cx: tim2::Context) 
    {
        // Acknowledge IRQ
        cx.resources.tim2.sr.modify(|_, w| w.cc2if().clear_bit());

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