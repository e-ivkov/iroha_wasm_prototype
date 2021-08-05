# Demo of transactions with WASM for Iroha2

## Considerations

This demo tries to both be ratrher simple and use an overall data model and architecture similar to Iroha2, to showcase the viability of WASM integration.

## Smart Contract

Simple smart contract that is submitted to iroha does the following:
- Decrements by 1 alice's balance if alice has > 10
- Increments by 1 otherwise

The Smart Contract can communicate with Iroha2 only through Instructions and Query execution - therefore making the model secure and easily guarded by permissions. This simple example in its code executes query when asks for balance and executes instruction to mint or burn.

## How to run

1. Use latest nightly (1.56.0-nightly tested).
2. Compile `example_smartcontract` with `cargo build --release --target wasm32-unknown-unknown`
3. Move `example_smartcontract.wasm` into `iroha`
4. In `iroha` do `cargo run`
5. You can modify and play with alice balance in main function code.