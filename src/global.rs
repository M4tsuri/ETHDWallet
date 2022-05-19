use core::cell::Cell;

use cortex_m::interrupt::Mutex;
use rand_chacha::ChaCha20Rng;
use stm32f4::stm32f407;
use stm32f4xx_hal::{
    gpio::{Output, Input, Pin}, 
    flash::LockedFlash
};

pub static LED: Mutex<Cell<Option<Pin<'F', 10, Output>>>> = Mutex::new(Cell::new(None));
pub static LED_STATE: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));
pub static EXTI: Mutex<Cell<Option<stm32f407::EXTI>>> = Mutex::new(Cell::new(None));
pub static RNG: Mutex<Cell<Option<ChaCha20Rng>>> = Mutex::new(Cell::new(None));
pub static KEY_TRIGGER: Mutex<Cell<Option<Pin<'D', 13, Input>>>> = Mutex::new(Cell::new(None));
pub static FLASH: Mutex<Cell<Option<LockedFlash>>> = Mutex::new(Cell::new(None));
