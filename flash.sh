#!/bin/bash
openocd -f openocd.cfg -c "program target/thumbv7m-none-eabi/release/stm32f103-vga-rs verify reset exit"
