use core::{ptr::addr_of, intrinsics::size_of, slice};

use chacha20::{
    ChaCha20,
    cipher::{
        StreamCipher, 
        generic_array::sequence::Split
    }
};

use k256::{self, ecdsa::SigningKey, elliptic_curve::sec1::ToEncodedPoint};
use rand::Rng;
use sha3::{Keccak256, Digest};
use stm32f4xx_hal::flash::FlashExt;

use crate::{global::*, update_global, input::KeyInputBuffer, wallet::WALLET_SECTOR};

use super::{ACCOUNT_NUM, utils::get_cipher};
use super::{Wallet, WALLET};
use crate::error::Result;

/// check if the wallet is initialized. If not, initialize it.
pub fn try_initialize_wallet() -> Result<()> {
    if WALLET.initialized {
        // TODO get a user input password from keyboard
        let passcode = KeyInputBuffer::wait_for_key();
        initialize_wallet(passcode);
    }

    Ok(())
}

fn initialize_wallet(passcode: [u8; 8]) {
    let iv: [u8; 12] = update_global!(|mut rng: Option<RNG>| {
        rng.gen()
    });

    let mut wallet = WALLET;
    
    wallet.chacha_iv = iv;

    let mut cipher = get_cipher(passcode, &iv);
    
    cipher.apply_keystream(&mut wallet.zone.zkmagic);
    initialize_accounts(&mut cipher, &mut wallet);
    // initialize OTP
    wallet.initialized = true;

    let wallet_addr = addr_of!(wallet);
    let wallet_size = size_of::<Wallet>();

    // program the wallet to flash
    update_global!(|mut flash: Option<FLASH>| {
        let wallet_slice = unsafe {
            slice::from_raw_parts(wallet_addr as *const u8, wallet_size)
        };

        let mut unlocked = flash.unlocked();
        unlocked.erase(WALLET_SECTOR).unwrap();
        unlocked.program(
            wallet_addr as usize, 
            wallet_slice.iter()
        ).unwrap();
    })
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
