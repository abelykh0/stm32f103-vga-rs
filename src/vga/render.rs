const SHOCK_ABSORBER_SHIFT_CYCLES : u16 = 60;

use stm32f1::stm32f103 as blue_pill;
use crate::vga::{HSIZE_CHARS, VSIZE_CHARS};
use crate::vga::display::VgaDisplay;

pub struct VgaDraw {
    pixels_ptr: u32,
    attributes_ptr : u32,
    attribute_definitions_ptr : u32,
    vline: u32, /* The current line being drawn */
    vdraw: u32, /* Used to increment vline every 2 drawn lines */
    vflag: bool /* When true, can draw on the screen */
}

impl VgaDraw {
    pub const fn new() -> VgaDraw {
        VgaDraw {
            pixels_ptr : 0,
            attributes_ptr : 0,
            attribute_definitions_ptr : 0,
            vline : 0,
            vdraw : 0,
            vflag : false
        }
    }

    pub fn init(&mut self, vga_display : &VgaDisplay) {
        self.pixels_ptr = vga_display.pixels.as_ptr() as u32;
        self.attributes_ptr = vga_display.attributes.as_ptr() as u32;
        self.attribute_definitions_ptr = vga_display.attribute_definitions.as_ptr() as u32;
    }

    // Call this from TIM3 interrupt handler
    #[inline(always)]
    pub fn on_hsync(&mut self) {
        if self.vflag {
            unsafe {
                vga_draw_impl(
                    self.pixels_ptr + self.vline * HSIZE_CHARS as u32,
                    self.attribute_definitions_ptr,
                    self.attributes_ptr + self.vline / 8 * HSIZE_CHARS as u32,
                    if cfg!(feature = "board2") { 0x40010C0D } else { 0x4001080C } as _)
            }

            self.vdraw += 1;
            if self.vdraw >= 2 {
                self.vdraw = 0;
                self.vline += 1;
                if self.vline == VSIZE_CHARS as u32 * 8 {
                    self.vline = 0;
                    self.vdraw = 0;
                    self.vflag = false;
                }
            }
        }
    }

    // Call this from TIM4 interrupt handler
    #[inline(always)]
    pub fn on_vsync(&mut self) {
        self.vline = 0;
        self.vflag = true;
    }
}

extern "C" {
    fn vga_draw_impl(pix: u32, attr_base: u32, attr: u32, odr: u32);
}

pub fn init_vga(
    p: &blue_pill::Peripherals) 
{
    p.RCC.apb2enr.modify(|_r, w| w.iopben().set_bit());

    if cfg!(feature = "board2") {
        // Set PB8..PB9, PB12..PB15 to OUTPUT with high speed
        p.GPIOB.crh.modify(|_r, w| { w
            .mode8().output50().cnf8().push_pull()
            .mode9().output50().cnf9().push_pull()
            .mode12().output50().cnf12().push_pull()
            .mode13().output50().cnf13().push_pull()
            .mode14().output50().cnf14().push_pull()
            .mode15().output50().cnf15().push_pull()
        });
    }
    else {
        // Set PA0..PA5 to OUTPUT with high speed
        p.RCC.apb2enr.modify(|_r, w| w.iopaen().set_bit());
        p.GPIOA.crl.modify(|_r, w| { w
            .mode0().output50().cnf0().push_pull()
            .mode1().output50().cnf1().push_pull()
            .mode2().output50().cnf2().push_pull()
            .mode3().output50().cnf3().push_pull()
            .mode4().output50().cnf4().push_pull()
            .mode5().output50().cnf5().push_pull()
        });
    }

    // HSync on PB0 and VSync on PB6
    p.GPIOB.crl.modify(|_r, w| { w
        .mode0().output50().cnf0().alt_push_pull()
        .mode6().output50().cnf6().alt_push_pull()
    });

    // CPU is running at 72 MHz
    // VGA is 800x600@56Hz (pixel frequency 36 MHz)
    let real_pixels_per_pixel : u16 = 36 / 18;
	let mut used_horizontal_pixels = HSIZE_CHARS * 8 * real_pixels_per_pixel;
	if used_horizontal_pixels > 800
	{
		used_horizontal_pixels = 800;
	}
	let horizontal_offset = ((800 - used_horizontal_pixels) / 2) as u16;
    let factor = 72 / 36;
    let whole_line = factor * 1024;
    let sync_pulse = factor * 72;
    let start_draw = factor * (72 - 24) + 160;
    init_h_sync(p, whole_line, sync_pulse, start_draw + horizontal_offset);

	let mut used_vertical_pixels = VSIZE_CHARS * 8 * 2;
	if used_vertical_pixels > 600
	{
		used_vertical_pixels = 600;
	}
	let vertical_offset = ((600 - used_vertical_pixels) / 2) as u16;
    init_v_sync(p, 625, 2, 25 + vertical_offset);
}

fn init_v_sync(
    p: &blue_pill::Peripherals,
    whole_frame: u16,
    sync_pulse: u16,
    start_draw: u16)
{
    // TIM4 is used to generate vertical sync signal
    p.RCC.apb1enr.modify(|_, w| w.tim4en().set_bit());
    let tim4 = &p.TIM4;
    tim4.arr.write(|w| w.arr().bits(whole_frame - 1));
    tim4.cnt.write(|w| w.cnt().bits(0));
    tim4.cr1.write(|w| w
        .opm().disabled()
        .dir().up()
        .cms().edge_aligned()
        .arpe().disabled()
        .ckd().div1()
    );
    tim4.cr2.write(|w| w
        .ccds().clear_bit()
        .mms().update() // slave mode
        .ti1s().normal()
    );
    tim4.psc.write(|w| w.psc().bits(0));
    tim4.smcr.write(|w| w
        .sms().gated_mode()
        .ts().itr2() // TIM3
        .msm().no_sync()
        .etf().no_filter()
        .etps().div1()
        .ece().disabled()
        .etp().not_inverted()
    );

    // TIM4CH1: VSync on pin PB6
    tim4.ccr1.write(|w| w.ccr().bits(sync_pulse));
    tim4.ccmr1_output().write(|w| w
        .cc1s().output()
        .oc1fe().set_bit()
        .oc1m().pwm_mode1()
    );
    tim4.ccer.write(|w| w
        .cc1e().set_bit()
    );

    // TIM4CH4 triggers interrupt
    tim4.ccr4.write(|w| w.ccr().bits(start_draw));
    tim4.ccmr2_output().write(|w| w
        .cc4s().output()
        .oc4m().frozen()
    );
    tim4.egr.write(|w| w
        .cc4g().set_bit()
    );

    // Enable TIM4 IRQ
    tim4.dier.write(|w| w
        .cc4ie().set_bit()
    );

    // Start TIM4
    tim4.cr1.modify(|_, w| w.cen().set_bit());
}

fn init_h_sync(
    p: &blue_pill::Peripherals, 
    whole_line: u16, 
    sync_pulse: u16,
    start_draw: u16) 
{
    // TIM2 is used as a "shock absorber"
    p.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());
    let tim2 = &p.TIM2;
    tim2.arr.write(|w| w.arr().bits(whole_line - 1));
    tim2.cnt.write(|w| w.cnt().bits(0));
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
    tim2.ccr2.write(|w| w.ccr().bits(start_draw - 1 - SHOCK_ABSORBER_SHIFT_CYCLES));
    tim2.ccmr1_output().write(|w| w
        .cc2s().output()
        .oc2m().frozen()
    );
    tim2.egr.write(|w| w
        .cc2g().set_bit()
    );

    // Enable TIM2 IRQ
    tim2.dier.write(|w| w
        .cc2ie().set_bit()
    );

    // TIM3 is used to generate horizontal sync signal
    p.RCC.apb1enr.modify(|_, w| w.tim3en().set_bit());
    let tim3 = &p.TIM3;
    tim3.arr.write(|w| w.arr().bits(whole_line - 1));
    tim3.cnt.write(|w| w.cnt().bits(0));
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
    tim3.ccr3.write(|w| w.ccr().bits(sync_pulse));
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
        .cc2ie().set_bit()
    );

    // Start TIM2, which starts TIM3
    tim2.cr1.modify(|_, w| w.cen().set_bit());
}
