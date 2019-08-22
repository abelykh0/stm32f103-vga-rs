#![no_std]
#![no_main]

extern crate panic_halt;
mod stm32;
mod vga;
use rtfm::app;
use stm32f1::stm32f103 as device;

#[app(device = stm32f1::stm32f103)]
const APP: () = {
    // Late resorce binding
    static mut GPIOC: device::GPIOC = ();
    static mut TIM2: device::TIM2 = ();
    static mut TIM3: device::TIM3 = ();
    static mut TIM4: device::TIM4 = ();

    #[init]
    fn init() -> init::LateResources {
        // Configure PLL and flash
        let p = device::Peripherals::take().unwrap();
        stm32::configure_clocks(&p.RCC, &p.FLASH);

        // Configures the system timer to trigger a SysTick exception every second
        stm32::configure_systick(&mut core.SYST, 72_000_000); // period = 1s

        // Built-in LED is on GPIOC, pin 13
        p.RCC.apb2enr.modify(|_r, w| w.iopcen().set_bit());
        p.GPIOC.crh.modify(|_r, w| { w
            .mode13().output50()
            .cnf13().push_pull()
        });

        // Initialize VGA
        vga::init_vga(&p);

        init::LateResources { 
            GPIOC: device.GPIOC,
            TIM2: device.TIM2,
            TIM3: device.TIM3,
            TIM4: device.TIM4
        }
    }

    #[idle (resources = [GPIOC])]
    fn idle() -> ! {
        loop {
            stm32::delay(1000);
            resources.GPIOC.brr.write(|w| w.br13().reset());
            stm32::delay(1000);
            resources.GPIOC.bsrr.write(|w| w.bs13().set());
        }
    }

    #[exception]
    fn SysTick() {
        unsafe {
            let count = stm32::SYSTICK_COUNT.get();
            *count = *count.wrapping_add(1)
        }
    }

    #[exception]
    fn PendSV() {
    }

    // Enabled manually
    #[interrupt (priority = 15, resources = [TIM2])]
    fn TIM2() 
    {
        // Acknowledge IRQ
        resources.TIM2.sr.modify(|_, w| w.cc2if().clear_bit());

        // Idle the CPU until an interrupt arrives
        cortex_m::asm::wfi()
    }

    // Enabled manually
    #[interrupt (priority = 16, resources = [TIM3])]
    fn TIM3() 
    {
    }

    #[interrupt (priority = 16, resources = [TIM4])]
    fn TIM4() 
    {
    }
};
