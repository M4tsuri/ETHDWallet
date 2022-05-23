use chacha20::{ChaCha20, cipher::KeyIvInit};
use sha3::{Keccak256, Digest};

use crate::input::FIXED_KEY_LEN;

pub fn get_cipher(passcode: [u8; FIXED_KEY_LEN], iv: &[u8; 12]) -> ChaCha20 {
    let mut keccak = Keccak256::default();
    keccak.update(passcode);
    // the chacha key
    let digest = keccak.finalize();
    ChaCha20::new(&digest, iv.into())
}
