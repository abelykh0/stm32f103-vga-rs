# stm32f103-vga-rs
VGA demo on STM32F103 "bluepill"

![Board](https://raw.githubusercontent.com/abelykh0/stm32f103-vga-rs/master/doc/Board.jpg)

## Overview
The STM32F103 is a Cortex-M3 microcontroller that has neither a video controller, nor enough RAM for a framebuffer at any reasonable resolution.

This demo works around this to produce an acceptable quality 800x600 video with 64 colors to display 364x296 pixels. It uses three timers and a GPIO port. 

![Screenshot](https://raw.githubusercontent.com/abelykh0/stm32f103-vga-rs/master/doc/Screenshot.jpg)

## How to connect wires

| PIN | Description | Connect To | Output |
| --- | ----------- | ---------- | ------ |
| PB08 | Red 1 | Resistor 680 Ohm | VGA red (1)
| PB09 | Red 2 | Resistor 470 Ohm | VGA red (1)
| PB12 | Green 1 | Resistor 680 Ohm | VGA green (2)
| PB13 | Green 2 | Resistor 470 Ohm | VGA green (2)
| PB14 | Blue 1 | Resistor 680 Ohm | VGA blue (3)
| PB15 | Blue 2 | Resistor 470 Ohm | VGA blue (3)
| PB0 | HSync | | VGA HSync (13)
| PB6 | VSync | | VGA VSync (14)
| PA0 | CLK | Resistor 2K2 to keyboard CLK and resistor 3K3 to GND
| PA1 | DATA | Resistor 2K2 to keyboard DATA and resistor 3K3 to GND
| G | Ground | | VGA Ground (5,6,7,8,10)

## How to build

I recommend following the setup chapters from the [Rust Embedded][2] book. In
particular, you need to have [Rust][1] and you need to make Rust aware of the
cross compilation target we're using here:

```shell
$ rustup target add thumbv7m-none-eabi
```

You will also need a [GNU ARM toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads/) to compile the assembly language
routine. 

Now you should be able to compile everything by entering:

```shell
$ cargo build --release
```

Note: on Windows there's currently an issue https://github.com/rust-embedded/cortex-m-rt/issues/80.

[1]: https://rust-lang.org
[2]: https://rust-embedded.github.io/book

## Implementation Details
* Timer TIM4 is used to generate vertical sync signal
* Timer TIM2 is used as a "shock absorber" to make the VGA stable
* Timer TIM3 is used to generate horizontal sync signal


