use stm32f1::stm32f103 as device;

pub static mut SYSTICK_COUNT : core::cell::UnsafeCell<u32> = core::cell::UnsafeCell::new(0);

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

pub fn configure_systick(syst : &mut cortex_m::peripheral::SYST, period : u32) {
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(period);
    syst.enable_counter();
    syst.enable_interrupt();
}

pub fn delay(milliseconds : u32) {
    let start_count = get_count();
    let end_count = start_count.wrapping_add(milliseconds);

    let mut count = start_count;
    while count < end_count || (end_count < start_count && count > start_count) {
        count = get_count();
    }
}

pub fn get_count() -> u32 {
    unsafe {
        *SYSTICK_COUNT.get()
    }
}

