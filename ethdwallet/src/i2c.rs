use cortex_m::{interrupt::free, prelude::*};
use stm32f4xx_hal::gpio::{Output, Input};

use crate::{global::*, init::i2c1_init};

pub fn set_i2c_bus(pins: I2cPins, delay: &mut TIM1Delay) -> I2cPins {
    let (scl, sda) = pins;
    let mut scl = scl.into_mode::<Output>();
    let mut sda = sda.into_mode::<Output>();
    sda.set_high();
    // Check that the SDA (data) is not being held low. 
    for _ in 0..20 {
        // If SDA is Low, then clock SCL Low for >5us then High>5us
        scl.set_low();
        delay.delay_us(5_u32);
        scl.set_high();
        delay.delay_us(5_u32);
        // check SDA input again. 
    }
    // When SDA becomes High make SDA LOW while the SCL is High 
    sda.set_low();
    // wait >5us and make SDA high i.e. send I2C STOP control.
    delay.delay_us(5_u32);
    sda.set_high();
    // Wait >5us and restore SDA and SCL to tri-state inputs which is the   defaultstate on reset.
    (
        scl.into_mode::<Input>(),
        sda.into_mode::<Input>()
    )
}

pub fn reset_i2c1() {
    free(|cs| {
        let clk = CLOCK.borrow(cs).get().unwrap();
        let i2c = I2C1.borrow(cs).take().unwrap();
        let mut delay = DELAY.borrow(cs).take().unwrap();
        let (i2c1, (scl, sda)) = i2c.release();
        

        I2C1.borrow(cs).set(Some(
            i2c1_init(
                set_i2c_bus((scl, sda), &mut delay), 
                i2c1, &clk
            )
        ));

        DELAY.borrow(cs).set(Some(delay));
    })
}
