use core::intrinsics::transmute;

use cortex_m::interrupt::free;
use stm32f4::stm32f407::{
    GPIOH, GPIOD, GPIOA, GPIOB,
    self, USART1, I2C1
};
use stm32f4xx_hal::{
    pac,
    i2c::{I2c1, self},
    rcc::{RccExt, Enable, Clocks, Rcc},
    prelude::*, 
    gpio::{Edge, gpioa, gpiod, gpiob}, flash::LockedFlash, serial::{Serial, self}, syscfg::SysCfg, rng::Rng,
};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

use crate::{
    ALLOCATOR,
    set_global,
    wallet::initializer::try_initialize_wallet,
    global::*,
    error::{Result, Error}
};

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

/// initialize serial port
fn serial_init(gpioa: gpioa::Parts, usart1: USART1, clk: &Clocks) -> Result<()> {
    let pins = (
        gpioa
            .pa9
            .into_alternate(),
        gpioa
            .pa10
            .into_alternate()
    );
    let config = serial::Config::default().baudrate(14330.bps());
    let serial = Serial::new(
        usart1, pins, config, clk
    )?.with_u8_data();

    let (tx, mut rx) = serial.split();
    rx.listen();
    rx.unlisten_idle();

    free(|cs| {
        set_global!(SERIAL_RX, rx, cs);
        set_global!(SERIAL_TX, tx, cs);
    });

    Ok(())
}

fn keyboard_init(gpiod: gpiod::Parts, exti: &mut pac::EXTI, syscfg: &mut SysCfg) {
    // initialize triggering keyboard interrupt
    let mut key_trigger = gpiod.pd13.into_pull_down_input();
    key_trigger.make_interrupt_source(syscfg);
    key_trigger.trigger_on_edge(exti, Edge::Falling);

    free(|cs| {
        set_global!(KEY_TRIGGER, key_trigger, cs);
    })
}

fn heap_init() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024;
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
}

fn rng_init(mut rand_source: Rng) {
    let seed: [u64; 4] = array_init::array_init(|_| {
        rand_source.next_u64()
    });
    let rng = ChaCha20Rng::from_seed(unsafe {
        // this is totally safe since their size are the same
        transmute(seed)
    });

    free(|cs| {
        set_global!(RNG, rng, cs);
    })
}

fn i2c1_init(gpiob: gpiob::Parts, i2c1: I2C1, clk: &Clocks) {
    let i2c_pins = (gpiob.pb6, gpiob.pb7);
    let i2c1 = I2c1::new(
        i2c1,
        i2c_pins,
        i2c::Mode::Fast { 
            frequency: 100000.Hz(), 
            duty_cycle: i2c::DutyCycle::Ratio2to1 
        },
        clk
    );

    free(|cs| {
        set_global!(I2C1, i2c1, cs);
    })
}

pub fn init() -> Result<()> {
    let (Some(mut dp), Some(_cp)) = (
        stm32f407::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) else {
        return Err(Error::HalInitError)
    };

    heap_init();
    gpio_init(&dp);

    // see https://github.com/probe-rs/probe-rs/issues/350
    dp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    dp.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());

    let clocks = clock_init(dp.RCC.constrain());
    let mut syscfg = dp.SYSCFG.constrain();
    
    rng_init(dp.RNG.constrain(&clocks));
    i2c1_init(dp.GPIOB.split(), dp.I2C1, &clocks);
    serial_init(dp.GPIOA.split(), dp.USART1, &clocks)?;
    keyboard_init(dp.GPIOD.split(), &mut dp.EXTI, &mut syscfg);

    let gpiof = dp.GPIOF.split();

    free(|cs| {
        LED.borrow(cs).set(Some(gpiof.pf10.into_push_pull_output()));
        EXTI.borrow(cs).set(Some(dp.EXTI));
        FLASH.borrow(cs).set(Some(LockedFlash::new(dp.FLASH)));
    });
    
    // enable interrupts
    pac::NVIC::unpend(pac::interrupt::EXTI15_10);
    pac::NVIC::unpend(pac::interrupt::USART1);
    unsafe {
        pac::NVIC::unmask(pac::interrupt::EXTI15_10);
        pac::NVIC::unmask(pac::interrupt::USART1);
    }

    try_initialize_wallet()

    // // Create a delay abstraction based on SysTick
    // let mut delay = cp.SYST.delay(&clocks);
}
