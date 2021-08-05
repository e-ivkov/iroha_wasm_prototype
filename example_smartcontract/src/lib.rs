#![no_std]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use data_model::{AccountName, Instruction, Query, QueryResult, Stack};

extern crate alloc;
extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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

mod host {
    use data_model::Stack;

    #[link(wasm_import_module = "stack")]
    extern "C" {
        fn pop() -> u32;

        fn push(byte: u32);
    }

    pub struct HostStack;

    impl Stack for HostStack {
        fn push(&mut self, byte: u8) {
            unsafe { push(byte as u32) }
        }

        fn pop(&mut self) -> u8 {
            unsafe { pop() as u8 }
        }
    }
}

mod iroha {
    use super::host;
    use data_model::{Instruction, Query, QueryResult, Stack};

    mod inner {
        #[link(wasm_import_module = "iroha")]
        extern "C" {
            pub fn execute_query(size: u32) -> u32;

            pub fn execute_instruction(size: u32);
        }
    }

    pub fn execute_query(query: Query) -> QueryResult {
        let arg_size = host::HostStack.push_argument(query);
        let result_size = unsafe { inner::execute_query(arg_size) };
        host::HostStack.pop_argument(result_size)
    }

    pub fn execute_instruction(instruction: Instruction) {
        let arg_size = host::HostStack.push_argument(instruction);
        unsafe { inner::execute_instruction(arg_size) };
    }
}

mod local {
    use alloc::vec::Vec;
    use data_model::Stack;

    static mut STACK: Vec<u8> = Vec::new();

    #[no_mangle]
    extern "C" fn push(byte: u32) {
        unsafe { STACK.push(byte as u8) }
    }

    #[no_mangle]
    extern "C" fn pop() -> u32 {
        unsafe { STACK.pop().expect("Failed to pop.") as u32 }
    }

    pub struct LocalStack;

    impl Stack for LocalStack {
        fn push(&mut self, byte: u8) {
            unsafe { STACK.push(byte) }
        }

        fn pop(&mut self) -> u8 {
            unsafe { STACK.pop().expect("Failed to pop.") }
        }
    }
}

#[no_mangle]
pub extern "C" fn execute(account_size: u32) {
    let account_name = local::LocalStack.pop_argument::<AccountName>(account_size);
    let QueryResult::Balance(balance) =
        iroha::execute_query(Query::GetBalance(account_name.clone()));
    if balance < 10 {
        iroha::execute_instruction(Instruction::Mint(1, account_name))
    } else {
        iroha::execute_instruction(Instruction::Burn(1, account_name))
    }
}
