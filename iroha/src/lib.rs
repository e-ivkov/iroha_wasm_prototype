use std::collections::HashMap;

use data_model::{Account, AccountName, Instruction, Query, QueryResult, Stack};
use std::sync::{Arc, RwLock};
use wasmtime::*;

pub mod client;

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

struct WasmRuntime {
    engine: Engine,
    linker: Linker<State>,
}

impl WasmRuntime {
    pub fn new() -> WasmRuntime {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        linker
            .func_wrap(
                "stack",
                "push",
                |mut caller: Caller<'_, State>, value: u32| {
                    caller.data_mut().stack.push(value as u8)
                },
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
                    let instruction: Instruction =
                        caller.data_mut().pop_argument::<Instruction>(size);
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
        WasmRuntime { linker, engine }
    }

    pub fn execute(&mut self, wsv: Arc<RwLock<WSV>>, file_name: &str, account_name: &str) {
        let module = Module::from_file(&self.engine, file_name).unwrap();
        let mut store = Store::new(
            &self.engine,
            State {
                wsv,
                stack: Vec::new(),
            },
        );
        let instance = self.linker.instantiate(&mut store, &module).unwrap();
        let account_bytes =
            WasmStack::from_store(&mut store, &instance).push_argument(account_name.to_owned());

        let execute = instance
            .get_func(&mut store, "execute")
            .expect("`execute` was not an exported function");
        let execute = execute.typed::<u32, (), _>(&store).unwrap();
        execute.call(&mut store, account_bytes).unwrap();
    }
}

pub struct Iroha {
    runtime: WasmRuntime,
    pub wsv: Arc<RwLock<WSV>>,
}

impl Iroha {
    pub fn new(accounts: Vec<Account>) -> Iroha {
        Iroha {
            runtime: WasmRuntime::new(),
            wsv: Arc::new(RwLock::new(WSV {
                accounts: accounts
                    .into_iter()
                    .map(|account| (account.name.clone(), account))
                    .collect(),
            })),
        }
    }
}

pub struct Transaction {
    account_name: AccountName,
    payload: ExecutablePayload,
}

impl Transaction {
    pub fn with_instruction(instruction: Instruction, account_name: AccountName) -> Self {
        Transaction {
            account_name,
            payload: ExecutablePayload::Instruction(instruction),
        }
    }

    pub fn with_wasm(wasm_file_name: String, account_name: AccountName) -> Self {
        Transaction {
            account_name,
            payload: ExecutablePayload::Wasm(wasm_file_name),
        }
    }

    pub fn execute(&self, iroha: &mut Iroha) {
        self.payload.execute(iroha, &self.account_name)
    }
}

enum ExecutablePayload {
    Instruction(Instruction),
    Wasm(String),
}

impl ExecutablePayload {
    pub fn execute(&self, iroha: &mut Iroha, account_name: &str) {
        match self {
            ExecutablePayload::Instruction(instruction) => iroha
                .wsv
                .write()
                .unwrap()
                .execute_instruction(instruction.to_owned()),
            ExecutablePayload::Wasm(file_name) => {
                iroha
                    .runtime
                    .execute(iroha.wsv.clone(), file_name, account_name)
            }
        }
    }
}

#[derive(Debug)]
pub struct WSV {
    pub accounts: HashMap<AccountName, Account>,
}

impl WSV {
    pub fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Mint(amount, account) => {
                self.accounts.get_mut(&account).unwrap().balance += amount
            }
            Instruction::Burn(amount, account) => {
                self.accounts.get_mut(&account).unwrap().balance -= amount
            }
        }
    }

    pub fn execute_query(&self, query: Query) -> QueryResult {
        match query {
            Query::GetBalance(account) => {
                QueryResult::Balance(self.accounts.get(&account).unwrap().balance)
            }
        }
    }
}
