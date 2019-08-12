#![no_std]
#![no_main]

extern crate panic_halt;
extern crate stm32f1;
extern crate cortex_m_semihosting; 

use cortex_m_rt::{entry};
use stm32f1::stm32f103 as device;
//use cortex_m_semihosting::hio;
//use core::fmt::Write;
use cortex_m::asm::delay;

#[entry]
fn main() -> ! {
    let mut cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let p = device::Peripherals::take().unwrap();

    configure_clocks(&p.RCC, &p.FLASH);
    init_vga(&mut cp, &p);
    
    // Configure SysTick
    let mut syst = cp.SYST;
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(1_000);
    syst.enable_counter();

    // LED is on GPIOC, pin 13
    p.RCC.apb2enr.modify(|_r, w| w.iopcen().set_bit());
    p.GPIOC.crh.modify(|_r, w| { w
        .mode13().output50()
        .cnf13().push_pull()
    });

    //let mut is_led_on = false;
    loop {
        // Delay
        //while !syst.has_wrapped() {}
        delay(72_000_000);
        p.GPIOC.odr.write(|w| w.odr13().set_bit());
        delay(72_000_000);
        p.GPIOC.odr.write(|w| w.odr13().clear_bit());

        // Toggle LED
        //if is_led_on {
        //    p.GPIOC.brr.write(|w| w.br13().reset());
        //}
        //else {
        // /   p.GPIOC.bsrr.write(|w| w.bs13().set());
        //}

        //is_led_on = !is_led_on;
    }
}

pub fn init_vga(
    cp: &mut cortex_m::peripheral::Peripherals, 
    p: &device::Peripherals) 
{
    // Set PA0..PA5 to OUTPUT with high speed
    p.RCC.apb2enr.modify(|_r, w| w.iopaen().set_bit());
    p.GPIOC.crl.modify(|_r, w| { w
        .mode0().output50().cnf0().push_pull()
        .mode1().output50().cnf1().push_pull()
        .mode2().output50().cnf2().push_pull()
        .mode3().output50().cnf3().push_pull()
        .mode4().output50().cnf4().push_pull()
        .mode5().output50().cnf5().push_pull()
    });

    // HSync on PB0 and VSync on PB6
    p.RCC.apb2enr.modify(|_r, w| w.iopben().set_bit());
    p.GPIOC.crl.modify(|_r, w| { w
        .mode0().output50().cnf0().alt_push_pull()
        .mode6().output50().cnf6().alt_push_pull()
    });

    unsafe {
        cp.NVIC.set_priority(device::Interrupt::TIM4, 0x00);
        cp.NVIC.set_priority(device::Interrupt::TIM3, 0x10);
        //scb.set_priority(SystemHandler::PendSV, 0xFF);
    }

    init_h_sync(cp, p, 0, 0, 0);
}

pub fn init_h_sync(
    cp: &mut cortex_m::peripheral::Peripherals, 
    p: &device::Peripherals, 
    whole_line: u16, 
    start_draw: u16,
    sync_pulse: u16) 
{
    // TIM2 is used as a "shock absorber"
    p.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());
    let tim2 = &p.TIM2;
    tim2.arr.write(|w| w.arr().bits(whole_line - 1));
    tim2.cr1.write(|w| w
        .opm().disabled()
        .dir().up()
        .cms().edge_aligned()
        .arpe().disabled()
        .ckd().div1()
    );
    tim2.cr2.write(|w| w
        .ccds().clear_bit()
        .mms().enable() // master mode
        .ti1s().normal()
    );
    tim2.psc.write(|w| w.psc().bits(0));

    // TIM2CH2 triggers interrupt
    tim2.ccr2.write(|w| w.ccr().bits(start_draw - 12 - 1));
    tim2.ccmr1_output().write(|w| w
        .cc2s().output()
        .oc2m().frozen()
    );
    tim2.egr.write(|w| w
        .cc2g().set_bit()
    );

    // Enable TIM2 IRQ
    tim2.dier.write(|w| w
        .uie().set_bit()
        .cc2ie().set_bit()
    );

    // TIM3 is used to generate horizontal sync signal
    p.RCC.apb1enr.modify(|_, w| w.tim3en().set_bit());
    let tim3 = &p.TIM3;
    tim3.arr.write(|w| w.arr().bits(whole_line - 1));
    tim3.cr1.write(|w| w
        .opm().disabled()
        .dir().up()
        .cms().edge_aligned()
        .arpe().disabled()
        .ckd().div1()
    );
    tim3.cr2.write(|w| w
        .ccds().clear_bit()
        .mms().update() // slave mode
        .ti1s().normal()
    );
    tim3.psc.write(|w| w.psc().bits(0));
    tim3.smcr.write(|w| w
        .sms().trigger_mode()
        .ts().itr1() // TIM2
        .msm().no_sync()
        .etf().no_filter()
        .etps().div1()
        .ece().disabled()
        .etp().not_inverted()
    );

    // TIM3CH2 triggers interrupt
    tim3.ccr2.write(|w| w.ccr().bits(start_draw - 1));
    tim3.ccmr1_output().write(|w| w
        .cc2s().output()
        .oc2m().frozen()
    );
    tim3.egr.write(|w| w
        .cc2g().set_bit()
    );

    // TIM3CH3: HSync on pin PB0
    tim3.ccr2.write(|w| w.ccr().bits(sync_pulse));
    tim3.ccmr2_output().write(|w| w
        .cc3s().output()
        .oc3fe().set_bit()
        .oc3m().pwm_mode1()
    );
    tim3.ccer.write(|w| w
        .cc3e().set_bit()
    );


    // Enable TIM3 IRQ
    tim3.dier.write(|w| w
        .uie().set_bit()
        .cc2ie().set_bit()
    );

    // Start TIM3, which starts TIM4.
    tim3.cr1.modify(|_, w| w.cen().set_bit());
    //*isr::shock::SHOCK_TIMER.try_lock().unwrap() = Some(tim3);

    // Turn on both our device interrupts. We need to turn on TIM4 before
    // TIM3 or TIM3 may just wake up and idle forever.
    cp.NVIC.enable(device::Interrupt::TIM4);
    cp.NVIC.enable(device::Interrupt::TIM3);
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
