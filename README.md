# stm32f103-vga-rs
VGA demo on STM32F103 "bluepill"

## Overview
The STM32F103 is a Cortex-M3 microcontroller that has neither a video controller, nor enough RAM for a framebuffer at any reasonable resolution.

This demo works around this to produce an acceptable quality 800x600 video with 64 colors to display 364x296 pixels. It uses three timers and a GPIO port. 

## How to connect wires

| PIN | Description | Connect To | Output |
| --- | ----------- | ---------- | ------ |
| PA0 | Red 1 | Resistor 470 Ohm | VGA red (1)
| PA1 | Red 2 | Resistor 680 Ohm | VGA red (1)
| PA2 | Green 1 | Resistor 470 Ohm | VGA green (2)
| PA3 | Green 2 | Resistor 680 Ohm | VGA green (2)
| PA4 | Blue 1 | Resistor 470 Ohm | VGA blue (3)
| PA5 | Blue 2 | Resistor 680 Ohm | VGA blue (3)
| PB0 | HSync | | VGA HSync (13)
| PB6 | VSync | | VGA VSync (14)
| G | Ground | | VGA Ground (5,6,7,8,10)

## How to build

I recommend following the setup chapters from the [Rust Embedded][2] book. In
particular, you need to have [Rust][1] and you need to make Rust aware of the
cross compilation target we're using here:

```shell
$ rustup target add thumbv7em-none-eabi
```

You will also need a GNU ARM toolchainhttps://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads to compile the assembly language
routines.

Now you should be able to compile everything by entering:

```shell
$ cargo build --release
```
[1]: https://rust-lang.org
[2]: https://rust-embedded.github.io/book

