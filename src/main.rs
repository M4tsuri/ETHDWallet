#![feature(let_else)]
#![allow(clippy::empty_loop)]
#![no_main]
#![cfg_attr(not(test), no_std)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]

use core::alloc::Layout;

use alloc_cortex_m::CortexMHeap;
// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;


extern crate alloc;

mod error;
mod wallet;
mod main_loop;
mod init;
mod global;
mod interrupts;
mod input;
mod i2c;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop { }
}

#[entry]
fn main() -> ! {
    match init::init() {
        Err(_) => loop { },
        Ok(_) => main_loop::main_loop()
    }
}

