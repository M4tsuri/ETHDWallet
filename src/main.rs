#![feature(let_else)]
#![allow(clippy::empty_loop)]
#![no_main]
#![cfg_attr(not(test), no_std)]
#![feature(alloc_error_handler)]

use core::{cell::Cell, alloc::Layout, intrinsics::transmute};

use alloc_cortex_m::CortexMHeap;
use cortex_m_semihosting::hprintln;
use error::{Result, Error};
// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use cortex_m::interrupt::{free, Mutex, CriticalSection};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use stm32f4::stm32f407::{
    GPIOH, GPIOD, GPIOA, GPIOB,
    self
};
use stm32f4xx_hal::{
    interrupt, pac,
    i2c::{I2c1, self},
    rcc::{RccExt, Enable, Clocks, Rcc},
    prelude::*, 
    gpio::{Edge, Output, Pin, Input}
};

extern crate alloc;

mod error;
mod task;
mod wallet;

const DEBUG: bool = true;

#[macro_export]
macro_rules! debug {
    ($($t:tt),*) => {
        if DEBUG { hprintln!($($t),*) }
    }
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

static LED: Mutex<Cell<Option<stm32f4xx_hal::gpio::Pin<'F', 10, Output>>>> = Mutex::new(Cell::new(None));
static LED_STATE: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
static EXTI: Mutex<Cell<Option<stm32f407::EXTI>>> = Mutex::new(Cell::new(None));
static RNG: Mutex<Cell<Option<ChaCha20Rng>>> = Mutex::new(Cell::new(None));
static KEY_TRIGGER: Mutex<Cell<Option<Pin<'D', 13, Input>>>> = Mutex::new(Cell::new(None));

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop { }
}


/// initialize GPIO
/// 1. enable clocks on nessessary pins
fn gpio_init(dp: &stm32f407::Peripherals) {
    GPIOH::enable(&dp.RCC);
    GPIOD::enable(&dp.RCC);
    GPIOA::enable(&dp.RCC);
    GPIOB::enable(&dp.RCC);
}

fn clock_init(rcc: Rcc) -> Clocks {
    rcc.cfgr
        .use_hse(25.MHz())
        .sysclk(168.MHz())
        .hclk(168.MHz())
        .pclk1(42.MHz())
        .pclk2(84.MHz())
        // enable for the rand source
        .require_pll48clk()
        .freeze()
}

fn init() -> Result<()> {
    let (Some(mut dp), Some(_cp)) = (
        stm32f407::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) else {
        return Err(Error::HalInitError);
    };

    gpio_init(&dp);
    let clocks = clock_init(dp.RCC.constrain());

    // initialize random source
    let mut rand_source = dp.RNG.constrain(&clocks);
    let seed: [u64; 4] = array_init::array_init(|_| {
        rand_source.next_u64()
    });
    let rng = ChaCha20Rng::from_seed(unsafe {
        // this is totally safe since their size are the same
        transmute(seed)
    });

    // initialize i2c
    let i2c_parts = dp.GPIOB.split();
    let i2c_pins = (i2c_parts.pb6, i2c_parts.pb7);
    let _i2c1 = I2c1::new(
        dp.I2C1, 
        i2c_pins,
        i2c::Mode::Fast { 
            frequency: 100000.Hz(), 
            duty_cycle: i2c::DutyCycle::Ratio2to1 
        },
        &clocks
    );

    // initialize triggering keyboard interrupt
    let mut syscfg = dp.SYSCFG.constrain();
    let gpiod = dp.GPIOD.split();
    let mut key_trigger = gpiod.pd13.into_pull_down_input();
    key_trigger.make_interrupt_source(&mut syscfg);
    key_trigger.trigger_on_edge(&mut dp.EXTI, Edge::Falling);

    let gpiof = dp.GPIOF.split();

    free(|cs| {
        LED.borrow(cs).set(Some(gpiof.pf10.into_push_pull_output()));
        EXTI.borrow(cs).set(Some(dp.EXTI));
        RNG.borrow(cs).set(Some(rng));
        KEY_TRIGGER.borrow(cs).set(Some(key_trigger));
    });

    // initialize heap
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }
    
    // enable interrupt 
    pac::NVIC::unpend(pac::interrupt::EXTI15_10);
    unsafe {
        pac::NVIC::unmask(pac::interrupt::EXTI15_10);
    }

    debug!("initialization finished.");

    Ok(())

    // // Create a delay abstraction based on SysTick
    // let mut delay = cp.SYST.delay(&clocks);
}

#[entry]
fn main() -> ! {
    if let Err(_) = init() {
        loop { }
    };

    loop { }
}

fn set_led(cs: &CriticalSection) {
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
