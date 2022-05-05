#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f4 as _;

extern crate panic_halt;

#[entry]
fn main() -> ! {
    loop {}
}
