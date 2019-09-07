pub const HSIZE_CHARS : u16 = 40;
pub const VSIZE_CHARS : u16 = 37;

extern crate panic_halt;
use stm32f1::stm32f103 as device;
use embedded_graphics::prelude::*;
use embedded_graphics::Drawing;
use embedded_graphics::pixelcolor::BinaryColor;

/// VGA display
pub struct VgaDisplay {
    pub pixels: [u8; (HSIZE_CHARS * 8 * VSIZE_CHARS) as usize],
    pub attributes : [u8; (HSIZE_CHARS * VSIZE_CHARS) as usize],
    pub default_attribute : [u8; 64]
}

impl VgaDisplay {
    pub fn init_default_attribute(&mut self, back_color : u8, fore_color : u8)
    {
        for i in 0..16 {
            let mut value = i;
            let mut index = i << 2;
            for _bit in 0..4 {
                self.default_attribute[index] = if value & 0x08 == 0 { back_color } else { fore_color };
                value <<= 1;
                index += 1;
            }
        }
    }

    fn write_pixel(&mut self, x: u16, y: u16, val: BinaryColor) {
        if x >= HSIZE_CHARS * 8 || y >= VSIZE_CHARS * 8 {
            return
        }

        let bit = x & 0x07;
        let byte = x >> 3;

        if val == BinaryColor::Off {
            self.pixels[(y * HSIZE_CHARS + byte) as usize] &= !(1 << bit);
        } else {
            self.pixels[(y * HSIZE_CHARS + byte) as usize] |= 1 << bit;
        }
    }
}

impl Drawing<BinaryColor> for VgaDisplay {
    fn draw<T>(&mut self, item_pixels: T)
        where T: IntoIterator<Item = Pixel<BinaryColor>>,
    {
        for pixel in item_pixels {
            let point = pixel.0;
            self.write_pixel(point.x as u16, point.y as u16, pixel.1);
        }
    }
}

pub fn init_vga(
    p: &device::Peripherals) 
{
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

    // HSync on PB0 and VSync on PB6
    p.RCC.apb2enr.modify(|_r, w| w.iopben().set_bit());
    p.GPIOB.crl.modify(|_r, w| { w
        .mode0().output50().cnf0().alt_push_pull()
        .mode6().output50().cnf6().alt_push_pull()
    });

    // CPU is running at 72 MHz
    // VGA is 800x600@56Hz (pixel frequency 36 MHz)
    let real_pixels_per_pixel : u16 = 72 / 18;
	let mut used_horizontal_pixels = HSIZE_CHARS * 8 * real_pixels_per_pixel;
	if used_horizontal_pixels > 800 * real_pixels_per_pixel
	{
		used_horizontal_pixels = 800 * real_pixels_per_pixel;
	}
	let horizontal_offset = ((800 * real_pixels_per_pixel - used_horizontal_pixels) / 2) as u16;
    let factor = 72 / 36;
    let whole_line = factor * 1024;
    let sync_pulse = factor * 72;
    let start_draw = factor * 72 - 24 + 150;
    init_h_sync(p, whole_line, sync_pulse, start_draw + horizontal_offset);
    init_v_sync(p, 625, 2, 25);
}

fn init_v_sync(
    p: &device::Peripherals,
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
    p: &device::Peripherals, 
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
