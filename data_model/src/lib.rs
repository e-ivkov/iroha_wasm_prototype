#![no_std]

extern crate alloc;

use alloc::string::String;
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct Account {
    pub name: String,
    pub surname: String,
}
