use core::{cell::Cell, sync::atomic::AtomicBool};

use chacha20::ChaCha20;
use cortex_m::interrupt::Mutex;
use rand_chacha::ChaCha20Rng;
use stm32f4::stm32f407::{self, USART1};
use stm32f4xx_hal::{
    gpio::{Output, Input, Pin}, 
    flash::LockedFlash, serial::{Tx, Rx}, i2c::I2c1
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

#[macro_export]
macro_rules! update_global {
    (|$($($t:ident)+: $ty:ident<$global:ident>),*| $b:block) => {{
        use cortex_m::interrupt::free;
        free(|cs| {
            $(update_global!(@get $($t)+: $ty<$global>, cs);)*
            let block_result = $b;
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
global!(@option FLASH: LockedFlash);

global!(@option SERIAL_TX: Tx<USART1>);
global!(@option SERIAL_RX: Rx<USART1>);

global!(@option I2C1: I2c1<(Pin<'B', 6, Input>, Pin<'B', 7, Input>)>);
global!(@copy MSG_BUFFER: MsgBuffer = MsgBuffer::new());
global!(@copy KEY_BUFFER: KeyInputBuffer = KeyInputBuffer::new());
global!(@option FLASH: LockedFlash);

global!(@option CIPHER: ChaCha20);

