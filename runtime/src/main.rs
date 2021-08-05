use std::{
    iter,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use data_model::{Account, Instruction, Query, Stack};
use wasmtime::*;
use wsv::WSV;

mod wsv;

struct State {
    wsv: Arc<RwLock<WSV>>,
    stack: Vec<u8>,
}

struct WasmStack<'a> {
    push_fn: TypedFunc<u32, ()>,
    pop_fn: TypedFunc<(), u32>,
    store: &'a mut Store<State>,
}

impl<'a> WasmStack<'a> {
    pub fn from_store(store: &'a mut Store<State>, instance: &Instance) -> WasmStack<'a> {
        let push_fn = instance
            .get_func(&mut *store, "push")
            .expect("`push` was not an exported function");
        let push_fn = push_fn.typed::<u32, (), _>(&store).unwrap();
        let pop_fn = instance
            .get_func(&mut *store, "pop")
            .expect("`pop` was not an exported function");
        let pop_fn = pop_fn.typed::<(), u32, _>(&store).unwrap();
        WasmStack {
            store,
            push_fn,
            pop_fn,
        }
    }
}

impl<'a> Stack for WasmStack<'a> {
    fn push(&mut self, byte: u8) {
        self.push_fn.call(&mut *self.store, byte as u32).unwrap()
    }

    fn pop(&mut self) -> u8 {
        self.pop_fn.call(&mut *self.store, ()).unwrap() as u8
    }
}

impl Stack for State {
    fn push(&mut self, byte: u8) {
        self.stack.push(byte)
    }

    fn pop(&mut self) -> u8 {
        self.stack.pop().unwrap()
    }
}

fn main() -> Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    linker
        .func_wrap(
            "stack",
            "push",
            |mut caller: Caller<'_, State>, value: u32| caller.data_mut().stack.push(value as u8),
        )
        .unwrap();
    linker
        .func_wrap("stack", "pop", |mut caller: Caller<'_, State>| {
            caller.data_mut().stack.pop().unwrap() as u32
        })
        .unwrap();
    linker
        .func_wrap(
            "iroha",
            "execute_instruction",
            |mut caller: Caller<'_, State>, size: u32| {
                let instruction: Instruction = caller.data_mut().pop_argument::<Instruction>(size);
                caller
                    .data()
                    .wsv
                    .write()
                    .unwrap()
                    .execute_instruction(instruction);
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "iroha",
            "execute_query",
            |mut caller: Caller<'_, State>, size: u32| {
                let query: Query = caller.data_mut().pop_argument::<Query>(size);
                let query_result = caller.data().wsv.write().unwrap().execute_query(query);
                caller.data_mut().push_argument(query_result)
            },
        )
        .unwrap();

    let module = Module::from_file(&engine, "example_smartcontract.wasm")?;
    let wsv = Arc::new(RwLock::new(WSV {
        accounts: iter::once((
            "alice".to_owned(),
            Account {
                name: "alice".to_owned(),
                balance: 100,
            },
        ))
        .collect(),
    }));
    let mut store = Store::new(
        &engine,
        State {
            wsv: wsv.clone(),
            stack: Vec::new(),
        },
    );
    let instance = linker.instantiate(&mut store, &module)?;

    let account_bytes =
        WasmStack::from_store(&mut store, &instance).push_argument("alice".to_owned());

    let execute = instance
        .get_func(&mut store, "execute")
        .expect("`execute` was not an exported function");
    let execute = execute.typed::<u32, (), _>(&store)?;
    execute.call(&mut store, account_bytes)?;
    println!("WSV: {:?}", wsv);
    Ok(())
}
