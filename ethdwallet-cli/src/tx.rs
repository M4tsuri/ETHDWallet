use serde::{Serialize, Deserialize};
use serlp::types::{biguint, byte_array};
use num::BigUint;

use crate::Signature;

// (nonce, gasprice, startgas, to, value, data, chainid, 0, 0)
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct UnsignedTx {
    pub nonce: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
    #[serde(with = "byte_array")]
    pub to: [u8; 20],
    #[serde(with = "biguint")]
    pub value: BigUint,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub chainid: u32,
    pub _zero1: u8,
    pub _zero2: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct SignedTx {
    pub nonce: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
    #[serde(with = "byte_array")]
    pub to: [u8; 20],
    #[serde(with = "biguint")]
    pub value: BigUint,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub chainid: u32,
    #[serde(with = "byte_array")]
    pub r: [u8; 32],
    #[serde(with = "byte_array")]
    pub s: [u8; 32],
}

impl UnsignedTx {
    pub fn into_signed(self, sig: Signature) -> SignedTx {
        SignedTx {
            nonce: self.nonce,
            gas_price: self.gas_price,
            gas_limit: self.gas_limit,
            to: self.to,
            value: self.value,
            data: self.data,
            chainid: sig.v as u32 + self.chainid * 2 + 35,
            r: sig.r,
            s: sig.s,
        }
    }
}
