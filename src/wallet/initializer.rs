use chacha20::{
    ChaCha20,
    cipher::{
        KeyIvInit, StreamCipher, 
        generic_array::sequence::Split
    }
};
use cortex_m::{interrupt::free, singleton};
use k256::{self, ecdsa::SigningKey, elliptic_curve::sec1::ToEncodedPoint};
use rand::Rng;
use sha3::{Keccak256, Digest};

use crate::RNG;

use super::{safe_zone::{SafeZone, Context}, ACCOUNT_NUM, utils::get_cipher};

/// check if the wallet is initialized. If not, initialize it.
pub fn try_initialize_wallet() -> &'static Context {
    // TODO check if wallet is initialized, i.e. initialize the zone
    // TODO get a user input password from keyboard

    // let passcode = todo!();
    // initialize_wallet(passcode)
    let ctx = singleton!(:
        Context = Context::load()
    ).unwrap();

    if !ctx.initialized {
        // initialize_wallet(passcode, ctx);
    }

    ctx
}

fn initialize_wallet(passcode: &[u8], ctx: &mut Context) {
    let iv = free(|cs| {
        let mut rng = RNG.borrow(cs).take().unwrap();
        let iv: [u8; 12] = rng.gen();
        RNG.borrow(cs).set(Some(rng));
        iv
    });
    
    ctx.chacha_iv = iv;

    let mut cipher = get_cipher(passcode, &iv);
    
    cipher.apply_keystream(&mut ctx.zone.zkmagic);
    initialize_accounts(&mut cipher, ctx);
    // initialize OTP

}

fn initialize_accounts(cipher: &mut ChaCha20, ctx: &mut Context) {
    // generate accounts
    let mut rng = free(|cs| {
        RNG.borrow(cs).take().unwrap()
    });

    for i in 0..ACCOUNT_NUM {
        let key = SigningKey::random(&mut rng);

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

    free(|cs| {
        RNG.borrow(cs).set(Some(rng))
    });
}
