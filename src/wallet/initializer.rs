use chacha20::{
    ChaCha20,
    cipher::{
        StreamCipher, 
        generic_array::sequence::Split
    }
};
use cortex_m::{interrupt::free, singleton};
use k256::{self, ecdsa::SigningKey, elliptic_curve::sec1::ToEncodedPoint};
use rand::Rng;
use sha3::{Keccak256, Digest};

use crate::{global::*, update_global};

use super::{ACCOUNT_NUM, utils::get_cipher};
use super::Wallet;

/// check if the wallet is initialized. If not, initialize it.
pub fn try_initialize_wallet() -> &'static mut Wallet {
    let ctx = singleton!(:
        Wallet = Wallet::load()
    ).unwrap();

    if !ctx.initialized {
        // TODO get a user input password from keyboard
        // initialize_wallet(passcode, ctx);
    }

    ctx
}

fn initialize_wallet(passcode: [u8; 8], ctx: &mut Wallet) {
    let iv: [u8; 12] = update_global!(|mut rng: Option<RNG>| {
        rng.gen()
    });
    
    ctx.chacha_iv = iv;

    let mut cipher = get_cipher(passcode, &iv);
    
    cipher.apply_keystream(&mut ctx.zone.zkmagic);
    initialize_accounts(&mut cipher, ctx);
    // initialize OTP
}

/// generate accounts
fn initialize_accounts(cipher: &mut ChaCha20, ctx: &mut Wallet) {
    for i in 0..ACCOUNT_NUM {
        let key = update_global!(|mut rng: Option<RNG>| {
            SigningKey::random(&mut rng)
        });

        let mut privkey: [u8; 32] = key.to_bytes().into();
        cipher.apply_keystream(&mut privkey);

        let mut keccak = Keccak256::default();
        keccak.update(key.verifying_key()
            .to_encoded_point(false)
            .as_bytes());
        let (addr, _) = keccak.finalize().split();
        
        ctx.zone.keys[i] = privkey;
        ctx.addrs[i] = addr.into();
    }
}
