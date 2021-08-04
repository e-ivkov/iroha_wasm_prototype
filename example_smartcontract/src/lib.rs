#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use alloc::vec::Vec;
use data_model::Account;
use parity_scale_codec::{Decode, Encode};

extern crate alloc;
extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut STACK: Vec<u8> = Vec::new();

// Need to provide a tiny `panic` implementation for `#![no_std]`.
// This translates into an `unreachable` instruction that will
// raise a `trap` the WebAssembly execution if we panic at runtime.
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &::core::panic::PanicInfo) -> ! {
    ::core::intrinsics::abort();
}

// Need to provide an allocation error handler which just aborts
// the execution with trap.
#[alloc_error_handler]
#[no_mangle]
pub extern "C" fn oom(_: ::core::alloc::Layout) -> ! {
    ::core::intrinsics::abort();
}

#[no_mangle]
pub extern "C" fn push(byte: u32) {
    unsafe { STACK.push(byte as u8) }
}

#[no_mangle]
pub extern "C" fn pop() -> u32 {
    unsafe { STACK.pop().expect("Failed to pop.") as u32 }
}

fn pop_argument<T: Decode>(size: u32) -> T {
    let mut bytes = Vec::new();
    for _ in 0..size {
        bytes.push(unsafe { STACK.pop().expect("Failed to pop.") });
    }
    T::decode(&mut &bytes[..]).expect("Failed to decode")
}

fn push_argument<T: Encode>(argument: T) -> u32 {
    let mut bytes = argument.encode();
    let size = bytes.len();
    bytes.reverse();
    for byte in bytes {
        unsafe { STACK.push(byte as u8) }
    }
    size as u32
}

#[no_mangle]
pub extern "C" fn execute(account_size: u32) -> u32 {
    let account = pop_argument::<Account>(account_size);
    0
}
