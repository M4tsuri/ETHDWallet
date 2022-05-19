use cortex_m::interrupt::{CriticalSection, free};
use stm32f4::stm32f407::interrupt;

use crate::global::*;

pub fn set_led(cs: &CriticalSection) {
    let state = LED_STATE.borrow(cs).get();
    let mut led = LED.borrow(cs).take().unwrap();

    if state {
        led.set_high()
    } else {
        led.set_low()
    };

    LED.borrow(cs).set(Some(led));
    LED_STATE.borrow(cs).set(state ^ true);
}

#[allow(non_snake_case)]
#[interrupt]
fn EXTI15_10() {
    free(|cs| {
        let exti = EXTI.borrow(cs).take().unwrap();
        
        let pr = exti.pr.read();
        if pr.pr13().bit_is_set() {
            // when an interrupt is triggered, the pending bit corresponding to 
            // the interrupt line is set. This request can be reset by writing 
            // a 1 in the pending register
            exti.pr.write(|w| w.pr13().set_bit());
            set_led(cs);
        }

        EXTI.borrow(cs).set(Some(exti));
    })
}
