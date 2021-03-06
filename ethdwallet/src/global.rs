use core::{cell::Cell, sync::atomic::{AtomicBool, Ordering}};

use chacha20::ChaCha20;
use cortex_m::{interrupt::Mutex, peripheral::SCB};
use fugit::TimerDurationU32;
use rand_chacha::ChaCha20Rng;
use stm32f4::stm32f407::{self, USART1, TIM1, TIM2};
use stm32f4xx_hal::{
    gpio::{Output, Input, Pin}, 
    serial::{Tx, Rx}, i2c::I2c1,
    rcc::Clocks, timer::{Delay, Counter}, flash::LockedFlash, watchdog::IndependentWatchdog
};

use crate::input::{MsgBuffer, KeyInputBuffer};

macro_rules! global {
    (@option $name:ident: $ty:path) => {
        pub static $name: Mutex<Cell<Option<$ty>>> = Mutex::new(Cell::new(None));
    };

    (@copy $name:ident: $ty:path = $val:expr) => {
        pub static $name: Mutex<Cell<$ty>> = Mutex::new(Cell::new($val));
    }
}

#[macro_export]
macro_rules! set_global {
    ($name:ident, $val:expr, $cs:expr) => {
        $name.borrow($cs).set(Some($val))
    }
}

/// update the value of a global variable through a block
/// 
/// **DO NOT EARLY RETURN IN YOUR BLOCK**
/// 
/// use break label instead
#[macro_export]
macro_rules! update_global {
    (|$($($t:ident)+: $ty:ident<$global:ident>),*| $b:block) => {{
        use cortex_m::interrupt::free;
        free(|cs| {
            $(update_global!(@get $($t)+: $ty<$global>, cs);)*
            let block_result = (|| { $b })();
            $(update_global!(@set $($t)+: $ty<$global>, cs);)*
            block_result
        })
    }};

    (@get $name:ident: Option<$global:ident>, $cs:expr) => {
        let $name = $global.borrow($cs).take().unwrap();
    };

    (@get mut $name:ident: Option<$global:ident>, $cs:expr) => {
        let mut $name = $global.borrow($cs).take().unwrap();
    };

    (@set $name:ident: Option<$global:ident>, $cs:expr) => {
        $global.borrow($cs).set(Some($name));
    };

    (@set mut $name:ident: Option<$global:ident>, $cs:expr) => {
        $global.borrow($cs).set(Some($name));
    };

    (@get $name:ident: Copy<$global:ident>, $cs:expr) => {
        let $name = $global.borrow($cs).get();
    };

    (@get mut $name:ident: Copy<$global:ident>, $cs:expr) => {
        let mut $name = $global.borrow($cs).get();
    };

    (@set $name:ident: Copy<$global:ident>, $cs:expr) => {
        $global.borrow($cs).set($name);
    };

    (@set mut $name:ident: Copy<$global:ident>, $cs:expr) => {
        $global.borrow($cs).set($name);
    }
}

global!(@option LED: Pin<'F', 10, Output>);
pub static LED_STATE: AtomicBool = AtomicBool::new(true);

global!(@option EXTI: stm32f407::EXTI);
global!(@option RNG: ChaCha20Rng);
global!(@option KEY_TRIGGER: Pin<'D', 13, Input>);

global!(@option SERIAL_TX: Tx<USART1>);
global!(@option SERIAL_RX: Rx<USART1>);


pub type I2cPins = (Pin<'B', 6, Input>, Pin<'B', 7, Input>);
pub type I2cType = I2c1<I2cPins>;
// I2C1 open : 
// [
//  false <repeats 56 times>, true, 
//  false <repeats 15 times>, true, 
//  false, false, false, false, true, 
//  false, true, false <repeats 48 times>
// ]
global!(@option I2C1: I2cType);
global!(@copy MSG_BUFFER: MsgBuffer = MsgBuffer::new());
global!(@copy KEY_BUFFER: KeyInputBuffer = KeyInputBuffer::new());
global!(@option FLASH: LockedFlash);

global!(@option CIPHER: ChaCha20);
global!(@option CLOCK: Clocks);

pub type TIM1Delay = Delay<TIM1, 15000>;
global!(@option DELAY: TIM1Delay);
global!(@option IWDG: IndependentWatchdog);

pub static WATCHDOG: AtomicBool = AtomicBool::new(true);
/// true for rapid and false for slow
pub static DOG_MODE: AtomicBool = AtomicBool::new(false);
global!(@option DOG_TIMER: Counter<TIM2, 1000000>);

pub fn watchdog_set_rapid() {
    let result = update_global!(|mut dog: Option<DOG_TIMER>| {
        dog.cancel()?;
        dog.start(TimerDurationU32::millis(100))
    });

    if let Err(_) = result {
        SCB::sys_reset()
    }

    DOG_MODE.store(true, Ordering::SeqCst);
}

pub fn watchdog_set_slow() {
    let result = update_global!(|mut dog: Option<DOG_TIMER>| {
        dog.cancel()?;
        dog.start(TimerDurationU32::minutes(1))
    });

    if let Err(_) = result {
        SCB::sys_reset()
    }

    DOG_MODE.store(false, Ordering::SeqCst);
}
