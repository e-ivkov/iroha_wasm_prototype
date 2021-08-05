#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
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

pub trait Stack {
    fn push(&mut self, byte: u8);

    fn pop(&mut self) -> u8;

    fn push_argument<T: Encode>(&mut self, argument: T) -> u32 {
        let mut bytes = argument.encode();
        let size = bytes.len();
        bytes.reverse();
        for byte in bytes {
            self.push(byte)
        }
        size as u32
    }

    fn pop_argument<T: Decode>(&mut self, size: u32) -> T {
        let mut bytes: Vec<u8> = Vec::new();
        for _ in 0..size {
            bytes.push(self.pop());
        }
        let argument = T::decode(&mut &bytes[..]).expect("Failed to decode");
        argument
    }
}
