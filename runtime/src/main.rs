use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use data_model::{Account, Instruction, Query};
use parity_scale_codec::{Decode, Encode};
use wasmtime::*;
use wsv::WSV;

mod wsv;

struct State {
    wsv: Arc<RwLock<WSV>>,
    stack: Vec<u8>,
}

fn main() -> Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    let module = Module::from_file(&engine, "example_smartcontract.wasm")?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])?;
    linker.func_wrap(
        "stack",
        "push",
        |mut caller: Caller<'_, State>, value: u32| caller.data_mut().stack.push(value as u8),
    );
    linker.func_wrap("stack", "pop", |mut caller: Caller<'_, State>| {
        caller.data_mut().stack.pop().unwrap() as u32
    });
    linker.func_wrap(
        "iroha",
        "execute_instruction",
        |mut caller: Caller<'_, State>, size: u32| {
            let instruction: Instruction =
                pop_argument_state::<Instruction>(size, caller.data_mut());
            caller
                .data()
                .wsv
                .write()
                .unwrap()
                .execute_instruction(instruction);
        },
    );
    linker.func_wrap(
        "iroha",
        "execute_query",
        |mut caller: Caller<'_, State>, size: u32| {
            let query: Query = pop_argument_state::<Query>(size, caller.data_mut());
            let query_result = caller.data().wsv.write().unwrap().execute_query(query);
            push_argument_state(query_result, caller.data_mut())
        },
    );

    let push = instance
        .get_func(&mut store, "push")
        .expect("`push` was not an exported function");
    let push = push.typed::<u32, (), _>(&store)?;
    let pop = instance
        .get_func(&mut store, "pop")
        .expect("`pop` was not an exported function");
    let pop = pop.typed::<(), u32, _>(&store)?;

    let account = Account {
        name: "alice in wonderland".to_owned(),
        balance: 0,
    };
    let account_bytes = push_argument_wasm(account, push, &mut store)?;

    let execute = instance
        .get_func(&mut store, "execute")
        .expect("`execute` was not an exported function");
    let execute = execute.typed::<u32, (), _>(&store)?;
    let result = execute.call(&mut store, account_bytes)?;
    println!("Name has letters: {:?}", result);
    Ok(())
}

fn push_argument_wasm<T: Encode>(
    argument: T,
    push_fn: TypedFunc<u32, ()>,
    store: &mut Store<()>,
) -> Result<u32> {
    let mut bytes = argument.encode();
    let size = bytes.len();
    bytes.reverse();
    for byte in bytes {
        push_fn.call(&mut *store, byte as u32)?
    }
    Ok(size as u32)
}

fn pop_argument_wasm<T: Decode>(
    size: u32,
    pop_fn: TypedFunc<(), u32>,
    store: &mut Store<()>,
) -> Result<T> {
    let mut bytes: Vec<u8> = Vec::new();
    for _ in 0..size {
        bytes.push(pop_fn.call(&mut *store, ())? as u8);
    }
    let argument = T::decode(&mut &bytes[..]).map_err(|err| anyhow!(err.to_string()))?;
    Ok(argument)
}

fn pop_argument_state<T: Decode>(size: u32, state: &mut State) -> T {
    let mut bytes = Vec::new();
    for _ in 0..size {
        bytes.push(state.stack.pop().unwrap());
    }
    T::decode(&mut &bytes[..]).expect("Failed to decode")
}

fn push_argument_state<T: Encode>(argument: T, state: &mut State) -> u32 {
    let mut bytes = argument.encode();
    let size = bytes.len();
    bytes.reverse();
    for byte in bytes {
        state.stack.push(byte as u8)
    }
    size as u32
}
