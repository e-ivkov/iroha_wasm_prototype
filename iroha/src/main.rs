use anyhow::Result;
use data_model::Account;
use iroha::{client::Client, Iroha, Transaction};

fn main() -> Result<()> {
    let mut iroha = Iroha::new(vec![Account {
        name: "alice".to_owned(),
        balance: 100,
    }]);
    println!("WSV before WASM execution: {:?}", iroha.wsv);
    let mut client = Client::new(&mut iroha);
    // The WASM contract decrements by 1 if alice has > 10 and increments by 1 otherwise.
    client.submit_transaction(Transaction::with_wasm(
        "example_smartcontract.wasm".to_owned(),
        "alice".to_owned(),
    ));
    println!("WSV after WASM execution: {:?}", iroha.wsv);
    Ok(())
}
