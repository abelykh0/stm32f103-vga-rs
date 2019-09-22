use arraydeque::ArrayDeque;
use pc_keyboard::{Keyboard, layouts, ScancodeSet2};

/// VGA display
pub struct Ps2Keyboard {
    pub queue: ArrayDeque<[u8; 6]>,
    pub pc_keyboard : Keyboard<layouts::Us104Key, ScancodeSet2>
}