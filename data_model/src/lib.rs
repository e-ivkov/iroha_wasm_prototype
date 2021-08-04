#![no_std]

extern crate alloc;

use alloc::string::String;
use parity_scale_codec::{Decode, Encode};

pub type AccountName = String;

#[derive(Encode, Decode, Debug)]
pub enum Instruction {
    Mint(u32, AccountName),
    Burn(u32, AccountName),
}

#[derive(Encode, Decode, Debug)]
pub enum Query {
    GetBalance(AccountName),
}

#[derive(Encode, Decode, Debug)]
pub enum QueryResult {
    Balance(u32),
}

#[derive(Encode, Decode, Debug)]
pub struct Account {
    pub name: AccountName,
    pub balance: u32,
}
