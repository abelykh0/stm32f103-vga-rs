#![no_std]
#![no_main]

extern crate panic_halt;
extern crate stm32f1;
extern crate cortex_m_semihosting; 

use cortex_m_rt::{entry};
use stm32f1::stm32f103 as device;
use cortex_m_semihosting::hio;
use core::fmt::Write;

#[entry]
fn main() -> ! {
    let mut debug_out = hio::hstdout().unwrap();
    write!(debug_out, "Hello, world!").unwrap();

    let p = device::Peripherals::take().unwrap();

    configure_clocks(&p.RCC, &p.FLASH);
    
    // Configure SysTick
    let peripherals = cortex_m::peripheral::Peripherals::take().unwrap();
    let mut syst = peripherals.SYST;
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(1_000);
    syst.enable_counter();

    // LED is on GPIOC, pin 13
    p.RCC.apb2enr.modify(|_r, w| w.iopcen().set_bit());
    p.GPIOC.crh.modify(|_r, w| { w
        .mode13().output50()
        .cnf13().push_pull()
    });

    let mut is_led_on = false;
    loop {
        // Delay
        while !syst.has_wrapped() {}

        // Toggle LED
        if is_led_on {
            p.GPIOC.brr.write(|w| w.br13().reset());
        }
        else {
            p.GPIOC.bsrr.write(|w| w.bs13().set());
        }

        is_led_on = !is_led_on;
    }
}

macro_rules! block_while {
    ($condition:expr) => {
        while $condition {}
    };
}

macro_rules! block_until {
    ($condition:expr) => {
        block_while!(!$condition)
    };
}

pub fn configure_clocks(rcc: &device::RCC, flash: &device::FLASH) {
    // Switch to the internal oscillator while messing with the PLL.
    rcc.cr.modify(|_, w| w.hsion().set_bit());
    block_until! { rcc.cr.read().hsirdy().bit() }

    // Make the switch.
    rcc.cfgr.modify(|_, w| w.sw().hsi());
    block_until! { rcc.cfgr.read().sws().is_hsi() }

    // Turn off the PLL.
    rcc.cr.modify(|_, w| w.pllon().clear_bit());
    block_while! { rcc.cr.read().pllrdy().bit() }

    // Apply divisors before boosting frequency.
    rcc.cfgr.modify(|_, w| { w
        .hpre().div1()  // AHB
        .ppre1().div2() // APB1
        .ppre2().div1() // APB2
        .usbpre().div1_5()
    });

    flash.acr.modify(|_, w| w.latency().ws2());

    // Switch on the crystal oscillator.
    rcc.cr.modify(|_, w| w.hseon().set_bit());
    block_until! { rcc.cr.read().hserdy().bit() }

    // Configure the PLL.
    rcc.cfgr.modify(|_, w| { w
        .pllmul().mul9()
        .pllsrc().hse_div_prediv()
    });

    // Turn it on.
    rcc.cr.modify(|_, w| w.pllon().set_bit());
    block_until! { rcc.cr.read().pllrdy().bit() }

    // Select PLL as clock source.
    rcc.cfgr.modify(|_, w| w.sw().pll());
    block_until! { rcc.cfgr.read().sws() == device::rcc::cfgr::SWSR::PLL }
}
