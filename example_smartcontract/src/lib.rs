#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use alloc::vec::Vec;
use data_model::Account;
use parity_scale_codec::Decode;

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
pub extern "C" fn remove_first() -> u32 {
    unsafe { STACK.remove(0) as u32 }
}

fn pop_argument<T: Decode>(size: u32) -> T {
    let mut bytes = Vec::new();
    for _ in 0..size {
        bytes.push(unsafe { STACK.remove(0) });
    }
    T::decode(&mut &bytes[..]).expect("Failed to decode")
}

#[no_mangle]
pub extern "C" fn execute(account_size: u32) -> u32 {
    pop_argument::<Account>(account_size).name.len() as u32
}
