//! Demonstrate the use of a blocking `Delay` using the SYST (sysclock) timer.

#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        
        // Set up the LED. On the Nucleo-446RE it's connected to pin PA5.
        let gpiof = dp.GPIOF.split();
        let gpioc = dp.GPIOC.split();
        let gpoib = dp.GPIOB.split();
        let gpioh = dp.GPIOH.split();
        

        let mut led1 = gpiof.pf10.into_push_pull_output();
        let mut led2 = gpioc.pc0.into_push_pull_output();
        let mut led3 = gpoib.pb15.into_push_pull_output();
        let mut led4 = gpioh.ph15.into_push_pull_output();

        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Create a delay abstraction based on SysTick
        let mut delay = cp.SYST.delay(&clocks);

        loop {
            // On for 1s, off for 1s.
            led1.set_high();
            delay.delay_ms(100_u32);
            led2.set_high();
            delay.delay_ms(100_u32);
            led3.set_high();
            delay.delay_ms(100_u32);
            led4.set_high();
            delay.delay_ms(100_u32);
            led1.set_low();
            delay.delay_ms(100_u32);
            led2.set_low();
            delay.delay_ms(100_u32);
            led3.set_low();
            delay.delay_ms(100_u32);
            led4.set_low();
            delay.delay_ms(100_u32);
        }
    }

    loop {}
}