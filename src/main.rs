//! Demonstrate the use of a blocking `Delay` using the SYST (sysclock) timer.

#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use hal::gpio::Speed;
// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};

macro_rules! set_pins {
    (@set_pin $dp:expr, $part:tt, [$($pin:tt, $res:tt),+]) => {
        let gpio = $dp.$part.split();
        $(
            let mut $res = gpio.$pin.into_push_pull_output();
            $res.set_speed(Speed::Low);
        )+
    };

    ($dp:expr, [$($part:tt, [$($pin:tt, $res:tt),+]),+]) => {
        $(
            set_pins!(@set_pin  $dp, $part,[$($pin, $res),+])
        );+
    }
}

const SPEED: u32 = 3000;

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        
        
        set_pins!(dp, [
            GPIOE, [
                pe4, an_c,
                pe6, an_d
            ],
            GPIOD, [pd12, an_a],
            GPIOH, [ph13, an_b]
        ]);
        
        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Create a delay abstraction based on SysTick
        let mut delay = cp.SYST.delay(&clocks);
        
        loop {
            an_a.set_high();
            an_b.set_low();
            an_c.set_low();
            an_d.set_low();

            delay.delay_ms(SPEED);

            an_a.set_low();
            an_b.set_high();
            an_c.set_low();
            an_d.set_low();

            delay.delay_ms(SPEED);

            an_a.set_low();
            an_b.set_low();
            an_c.set_high();
            an_d.set_low();

            delay.delay_ms(SPEED);

            an_a.set_low();
            an_b.set_low();
            an_c.set_low();
            an_d.set_high();

            delay.delay_ms(SPEED);
        }
        


        // // Set up the system clock. We want to run at 48MHz for this one.
        // let rcc = dp.RCC.constrain();
        // let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();
// 
        // // Create a delay abstraction based on SysTick
        // let mut delay = cp.SYST.delay(&clocks);

    }

    loop {}
}