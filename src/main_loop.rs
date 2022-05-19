use cortex_m::interrupt::{CriticalSection, free};
use stm32f4xx_hal::gpio::ExtiPin;
use stm32f4::stm32f407;

use crate::{global::*, error::{Result, Error}};

pub fn main_loop() -> Result<()> {
    free(|cs| {
        let mut keyboard = KEY_TRIGGER.borrow(cs).take().unwrap();
        let mut exti = EXTI.borrow(cs).take().unwrap();
        keyboard.enable_interrupt(&mut exti);
        KEY_TRIGGER.borrow(cs).set(Some(keyboard));
        EXTI.borrow(cs).set(Some(exti));
    });


    Ok(())
}