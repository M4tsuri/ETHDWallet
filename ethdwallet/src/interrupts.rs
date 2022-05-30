use core::sync::atomic;

use cortex_m::{prelude::*, peripheral::SCB};
use stm32f4::stm32f407::interrupt;

use crate::{
    global::*, 
    update_global, 
    error::Error, 
    i2c::reset_i2c1, 
    input::{
        KeyInputState, FIXED_KEY_LEN, MsgBufferState
    }
};

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

pub const ZLG7290_ADDR: u8 = 0x38;

#[allow(non_snake_case)]
#[interrupt]
fn EXTI15_10() {
    let result: Result<(), Error> = update_global!(|
        exti: Option<EXTI>, 
        mut i2c: Option<I2C1>, 
        mut key: Copy<KEY_BUFFER>
    | {
        let pr = exti.pr.read();
        if pr.pr13().bit_is_set() {
            // clear interrupt bit
            exti.pr.write(|w| w.pr13().set_bit());

            let mut one_byte = [0x00];
            // check if any has been pressed
            set_led();
        
            // read key value
            i2c.write_read(ZLG7290_ADDR, &[0x01], &mut one_byte)?;
            if let Some(num) = to_segled_value(one_byte[0]) {
                let idx = match key.state {
                    KeyInputState::Reading(p) => p,
                    KeyInputState::Finished => return Ok(())
                };
                key.read(num);
                
                i2c.write(ZLG7290_ADDR, &[idx as u8 + 0x10, num])?;
                if idx > 0 {
                    i2c.write(ZLG7290_ADDR, &[idx as u8 + 0xf, SEG7_PLACEHOLDER])?
                } 
                
                if idx == FIXED_KEY_LEN - 1 {
                    i2c.write(ZLG7290_ADDR, &[0x17, SEG7_PLACEHOLDER])?;
                }
            }
        }

        Ok(())
    });

    if let Err(Error::I2cError) = result {
        reset_i2c1();
    }
}

macro_rules! conv_seg7 {
    ($($e:expr),*) => {
        $(
            1 << $e
        )|*
    }
}

const SEG7_PLACEHOLDER: u8 = 1  << 1;

/// ```
/// ----7----
/// |       |
/// 2       6
/// |       |
/// |---1---|
/// |       |
/// 3       5
/// |       |
/// ----4----
/// ```
fn to_segled_value(val: u8) -> Option<u8> {
    Some(match val {
        28 => conv_seg7!(5, 6),
        27 => conv_seg7!(1, 7, 3, 4, 6),
        26 => conv_seg7!(5, 6, 1, 7, 4),
        20 => conv_seg7!(1, 2, 5, 6),
        19 => conv_seg7!(5, 4, 1, 7, 2),
        18 => conv_seg7!(5, 4, 3, 2, 1, 7),
        12 => conv_seg7!(5, 6, 7),
        11 => conv_seg7!(1, 2, 3, 4, 5, 6, 7),
        10 => conv_seg7!(1, 2, 4, 5, 6, 7),
        3  => conv_seg7!(2, 3, 4, 5, 6, 7),
        _ => return None,
    })
}

#[allow(non_snake_case)]
#[interrupt]
fn USART1() {
    update_global!(|mut rx: Option<SERIAL_RX>, mut buf: Copy<MSG_BUFFER>| {
        while rx.is_rx_not_empty() {
            match rx.read() {
                Ok(byte) => buf.read(byte),
                Err(_err) => buf.state = MsgBufferState::Error(
                    Error::SerialTxError
                ),
            };
        }
    });
}

#[allow(non_snake_case)]
#[interrupt]
fn TIM2() {
    // check watchdog
    match WATCHDOG.compare_exchange(
        true, false, 
        atomic::Ordering::Acquire, 
        atomic::Ordering::Relaxed) 
    {
        Ok(true) | Err(_) => SCB::sys_reset(),
        _ => {},
    }
}
