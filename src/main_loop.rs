use cortex_m::interrupt::free;
use stm32f4xx_hal::gpio::ExtiPin;

use crate::{global::*, error::Result, update_global};

pub fn main_loop() -> Result<()> {

    update_global!(|mut keyboard: Option<KEY_TRIGGER>, mut exti: Option<EXTI>| {
        keyboard.enable_interrupt(&mut exti);
    });


    Ok(())
}