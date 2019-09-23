use arraydeque::ArrayDeque;
use pc_keyboard::{Keyboard, layouts, ScancodeSet2, HandleControl, KeyEvent};
use stm32f1::stm32f103 as blue_pill;

const CLK_PIN : u16 = 0;
const DATA_PIN : u16 = 1;

// PS/2 Keyboard
pub struct Ps2Keyboard {
    queue: ArrayDeque<[KeyEvent; 6]>,
    pc_keyboard : Keyboard<layouts::Us104Key, ScancodeSet2>,
    last_clk : u8,
    last_bit : bool
}

impl Ps2Keyboard {
    pub fn new() -> Ps2Keyboard {
        Ps2Keyboard {
            queue : ArrayDeque::new(),
            pc_keyboard : Keyboard::new(layouts::Us104Key, ScancodeSet2, HandleControl::Ignore),
            last_clk : 1,
            last_bit : false
        }
    }

    pub fn init(p: &blue_pill::Peripherals) {
        if cfg!(feature = "board2") {
            p.GPIOA.crl.modify(|_r, w| { w
                .mode0().input().cnf0().push_pull() // CLK_PIN
                .mode1().input().cnf1().push_pull() // DATA_PIN
            });
        }
        else {
            // TODO
        }
    }

    pub fn update(&mut self, gpio_bits : u16) {
        if gpio_bits & CLK_PIN == 0 {
            // CLK = 0

            self.last_bit = gpio_bits & DATA_PIN != 0;
            self.last_clk = 0;
        }
        else {
            // CLK = 1

            if self.last_clk == 0 {
                match self.pc_keyboard.add_bit(self.last_bit) {
                    Ok(Some(event)) => {
                        match self.queue.push_back(event) {
                            Ok(()) => {}
                            Err(_e) => {}
                        }
                    }
                    Ok(None) => {
                    }
                    Err(_e) => {
                        //println!("Error decoding: {:?}", e);
                    }
                }
            }

            self.last_clk = 1;
        }    
    }

    pub fn get_event(&mut self) -> Option<KeyEvent> {
        self.queue.pop_front()
    }
}
