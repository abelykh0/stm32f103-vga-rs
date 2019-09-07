#![no_std]
#![no_main]

extern crate panic_halt;

mod stm32;
mod vga;
mod vga_draw;

use rtfm::app;
use stm32f1::stm32f103 as device;

#[app(device = stm32f1::stm32f103)]
const APP: () = {
    // Late resorce binding
    static mut GPIOA: device::GPIOA = ();
    static mut GPIOC: device::GPIOC = ();
    static mut TIM2: device::TIM2 = ();
    static mut TIM3: device::TIM3 = ();
    static mut TIM4: device::TIM4 = ();

    // VGA
    static mut DISPLAY : vga::VgaDisplay = vga::VgaDisplay {
        pixels : [0; (vga::HSIZE_CHARS * 8 * vga::VSIZE_CHARS) as usize],
        attributes : [0; (vga::HSIZE_CHARS * vga::VSIZE_CHARS) as usize],
        default_attribute : [0; 64]
    };
    static mut VLINE: i32 = 0; /* The current line being drawn */
    static mut VDRAW: i32 = 0; /* Used to increment vline every 2 drawn lines */
    static mut VFLAG: bool = true; /* When true, can draw on the screen */
    //const GPIO_ODR: *device::gpioa::RegisterBlock = device::GPIOA::ptr();

    #[init (resources = [DISPLAY])]
    fn init() -> init::LateResources {
        // Configure PLL and flash
        stm32::configure_clocks(&device.RCC, &device.FLASH);

        // Configures the system timer to trigger a SysTick exception every second
        stm32::configure_systick(&mut core.SYST, 72_000); // period = 1ms

        // Built-in LED is on GPIOC, pin 13
        device.RCC.apb2enr.modify(|_r, w| w.iopcen().set_bit());
        device.GPIOC.crh.modify(|_r, w| { w
            .mode13().output50()
            .cnf13().push_pull()
        });

        // Initialize VGA
        resources.DISPLAY.init_attribute(0x10, 0x3F);
        vga::init_vga(&device);

        init::LateResources { 
            GPIOA: device.GPIOA,
            GPIOC: device.GPIOC,
            TIM2: device.TIM2,
            TIM3: device.TIM3,
            TIM4: device.TIM4
        }
    }

    #[idle (resources = [GPIOC])]
    fn idle() -> ! {
        loop {
            resources.GPIOC.brr.write(|w| w.br13().reset());
            stm32::delay(1000);
            //cortex_m::asm::delay(72_000_000);

            resources.GPIOC.bsrr.write(|w| w.bs13().set());
            stm32::delay(1000);
            //cortex_m::asm::delay(72_000_000);
        }
    }

    #[exception (priority = 14)]
    fn SysTick() {
        unsafe {
            let count = stm32::SYSTICK_COUNT.get();
            *count = (core::num::Wrapping(*count) + core::num::Wrapping(1)).0;
        }
    }

    // #[exception (priority = 1)]
    // fn PendSV() {
    // }

    #[interrupt (priority = 15, resources = [TIM2])]
    fn TIM2() 
    {
        // Acknowledge IRQ
        resources.TIM2.sr.modify(|_, w| w.cc2if().clear_bit());

        // Idle the CPU until an interrupt arrives
        cortex_m::asm::wfi()
    }

    #[interrupt (priority = 16, resources = [TIM3, GPIOA, VLINE, VDRAW, VFLAG, DISPLAY])]
    fn TIM3() 
    {
        // Acknowledge IRQ
        resources.TIM3.sr.modify(|_, w| w.cc2if().clear_bit());

        // Draw
        unsafe {
            if *resources.VFLAG {
                resources.GPIOA.odr.write(|w| w.bits(0x02));
                cortex_m::asm::delay(5);
                resources.GPIOA.odr.write(|w| w.bits(0x0));
                vga_draw::vga_draw_impl(
                    &*resources.DISPLAY.pixels.as_ptr().offset(*resources.VLINE as isize * vga::HSIZE_CHARS as isize),
                    &*resources.DISPLAY.default_attribute.as_ptr(),
                    &*resources.DISPLAY.attributes.as_ptr().offset(*resources.VLINE as isize / 8 * vga::HSIZE_CHARS as isize),
                    0x4001080C as _);

                *resources.VDRAW += 1;
                if *resources.VDRAW == 2 {
                    *resources.VDRAW = 0;
                    *resources.VLINE += 1;
                    if *resources.VLINE == vga::VSIZE_CHARS as i32 * 8 {
                        *resources.VLINE = 0;
                        *resources.VDRAW = 0;
                        *resources.VFLAG = false;
                    }
                }
            }
        }
    }

    #[interrupt (priority = 16, resources = [TIM4, VLINE, VFLAG])]
    fn TIM4() 
    {
        // Acknowledge IRQ
        resources.TIM4.sr.modify(|_, w| w.cc4if().clear_bit());

        *resources.VLINE = 0;
        *resources.VFLAG = true;
    }
};
