use stm32f1::stm32f103 as device;

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
