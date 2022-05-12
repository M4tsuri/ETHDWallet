#![feature(let_else)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use core::cell::Cell;

use error::{Result, Error};
// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use cortex_m::interrupt::{free, Mutex};
use stm32f4::stm32f407::{
    GPIOH, GPIOD, GPIOA, GPIOB,
    self
};
use stm32f4xx_hal::{
    interrupt, pac,
    i2c::{I2c1, self},
    rcc::{RccExt, Enable},
    prelude::*, 
    gpio::{Edge, Output, Input}
};

mod error;

static LED: Mutex<Cell<Option<stm32f4xx_hal::gpio::Pin<'F', 10, Output>>>> = Mutex::new(Cell::new(None));
static KEY_TRIGGER: Mutex<Cell<Option<stm32f4xx_hal::gpio::Pin<'D', 13, Input>>>> = Mutex::new(Cell::new(None));
static LED_STATE: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));

fn gpio_init(dp: &stm32f407::Peripherals) {
    GPIOH::enable(&dp.RCC);
    GPIOD::enable(&dp.RCC);
    GPIOA::enable(&dp.RCC);
    GPIOB::enable(&dp.RCC);
}

fn main_loop() -> Result<()> {
    let (Some(mut dp), Some(_cp)) = (
        stm32f407::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) else {
        return Err(Error::HalInitError);
    };

    gpio_init(&dp);

    let gpiof = dp.GPIOF.split();

    free(|cs| {
        LED.borrow(cs).replace(Some(gpiof.pf10.into_push_pull_output()));
    });

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr
        .use_hse(25.MHz())
        .sysclk(168.MHz())
        .hclk(168.MHz())
        .pclk1(42.MHz())
        .pclk2(84.MHz())
        .freeze();

    let i2c_parts = dp.GPIOB.split();
    let i2c_pins = (i2c_parts.pb6, i2c_parts.pb7);

    // initialize i2c
    let _i2c1 = I2c1::new(
        dp.I2C1, 
        i2c_pins,
        i2c::Mode::Fast { 
            frequency: 100000.Hz(), 
            duty_cycle: i2c::DutyCycle::Ratio2to1 
        },
        &clocks
    );

    let mut syscfg = dp.SYSCFG.constrain();

    let gpiod = dp.GPIOD.split();
    let mut kb_trigger = gpiod.pd13.into_pull_down_input();
    kb_trigger.make_interrupt_source(&mut syscfg);
    kb_trigger.enable_interrupt(&mut dp.EXTI);
    kb_trigger.trigger_on_edge(&mut dp.EXTI, Edge::Falling);

    free(|cs| {
        KEY_TRIGGER.borrow(cs).replace(Some(kb_trigger));
    });
    

    pac::NVIC::unpend(pac::interrupt::EXTI15_10);
    unsafe {
        pac::NVIC::unmask(pac::interrupt::EXTI15_10);
    }

    Ok(())

    // // Set up the system clock. We want to run at 48MHz for this one.
    // let rcc = dp.RCC.constrain();
    // let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    // // Create a delay abstraction based on SysTick
    // let mut delay = cp.SYST.delay(&clocks);
}

#[entry]
fn main() -> ! {
    if let Err(_) = main_loop() {
        loop { }
    };

    loop { }
}

#[allow(non_snake_case)]
#[interrupt]
fn EXTI15_10() {
    free(|cs| {
        let state = LED_STATE.borrow(cs).get();
        let mut led = LED.borrow(cs).replace(None).unwrap();
        let mut key_trigger = KEY_TRIGGER.borrow(cs).replace(None).unwrap();
        key_trigger.clear_interrupt_pending_bit();

        if state { 
            led.set_high()
        } else {
            led.set_low()
        };

        KEY_TRIGGER.borrow(cs).replace(Some(key_trigger));
        LED.borrow(cs).replace(Some(led));
        LED_STATE.borrow(cs).replace(state ^ true);
    })
}
