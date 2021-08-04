use anyhow::Result;
use data_model::Account;
use parity_scale_codec::Encode;
use wasmtime::*;

fn main() -> Result<()> {
    // An engine stores and configures global compilation settings like
    // optimization level, enabled wasm features, etc.
    let engine = Engine::default();

    // We start off by creating a `Module` which represents a compiled form
    // of our input wasm module. In this case it'll be JIT-compiled after
    // we parse the text format.
    let module = Module::from_file(&engine, "example_smartcontract.wasm")?;

    // A `Store` is what will own instances, functions, globals, etc. All wasm
    // items are stored within a `Store`, and it's what we'll always be using to
    // interact with the wasm world. Custom data can be stored in stores but for
    // now we just use `()`.
    let mut store = Store::new(&engine, ());

    // With a compiled `Module` we can then instantiate it, creating
    // an `Instance` which we can actually poke at functions on.
    let instance = Instance::new(&mut store, &module, &[])?;

    let push = instance
        .get_func(&mut store, "push")
        .expect("`execute` was not an exported function");
    let push = push.typed::<u32, (), _>(&store)?;

    let account_bytes = Account {
        name: "alice in wonderland".to_owned(),
        surname: "rabbit".to_owned(),
    }
    .encode();

    for byte in &account_bytes[..] {
        push.call(&mut store, *byte as u32)?
    }

    let execute = instance
        .get_func(&mut store, "execute")
        .expect("`execute` was not an exported function");
    let execute = execute.typed::<u32, u32, _>(&store)?;
    let result = execute.call(&mut store, account_bytes.len() as u32)?;
    println!("Name has letters: {:?}", result);
    Ok(())
}
