use core::sync::atomic;

use cortex_m::interrupt::free;
use cortex_m::prelude::*;
use stm32f4::stm32f407::interrupt;
use stm32f4xx_hal::block;

use crate::{global::*, update_global};

pub fn set_led() {
    let state = LED_STATE.fetch_xor(
        true, atomic::Ordering::Relaxed
    );

    update_global!(|mut led: Option<LED>| {
        if state {
            led.set_high()
        } else {
            led.set_low()
        };
    });
}

#[allow(non_snake_case)]
#[interrupt]
fn EXTI15_10() {
    update_global!(|exti: Option<EXTI>| {
        let pr = exti.pr.read();
        if pr.pr13().bit_is_set() {
            // when an interrupt is triggered, the pending bit corresponding to 
            // the interrupt line is set. This request can be reset by writing 
            // a 1 in the pending register
            exti.pr.write(|w| w.pr13().set_bit());
        }
    });

    set_led();
}

#[allow(non_snake_case)]
#[interrupt]
fn USART1() {
    update_global!(|mut rx: Option<SERIAL_RX>, mut buf: Copy<MSG_BUFFER>| {
        while rx.is_rx_not_empty() {
            match block!(rx.read()) {
                Ok(byte) => buf.read(byte),
                Err(_error) => {}
            }
        }
    });
}
