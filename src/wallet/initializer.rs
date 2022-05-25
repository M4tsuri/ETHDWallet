use core::{ptr::addr_of, intrinsics::size_of, slice, iter::repeat};

use chacha20::{
    ChaCha20,
    cipher::{
        StreamCipher, 
        generic_array::{sequence::Split, GenericArray}, consts::U12
    }
};

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs;
use k256::{self, ecdsa::SigningKey, elliptic_curve::sec1::ToEncodedPoint};
use rand::Rng;
use sha3::{Keccak256, Digest};
use stm32f4xx_hal::flash::FlashExt;

use crate::{global::*, update_global, input::KeyInputBuffer, wallet::{WALLET_SECTOR, SECTIOR_BASE, WALLET_REPEAT}};

use super::{ACCOUNT_NUM, utils::get_cipher, wallet};
use super::{Wallet, WALLET};
use crate::error::Result;

/// check if the wallet is initialized. If not, initialize it.
pub fn try_initialize_wallet() -> Result<()> {
    if !wallet().initialized {
        // TODO get a user input password from keyboard
        let passcode = KeyInputBuffer::wait_for_key();
        initialize_wallet(passcode);
    }

    Ok(())
}

pub fn write_wallet(mut wallet: Wallet) {
    wallet.crc = 0;

    let stack_wallet_addr = addr_of!(wallet);
    let flash_wallet_addr = unsafe { addr_of!(WALLET) };
    let wallet_size = size_of::<Wallet>();

    // program the wallet to flash
    update_global!(|mut flash: Option<FLASH>, mut led: Option<LED>| {
        let wallet_slice = unsafe {
            slice::from_raw_parts(stack_wallet_addr as *const u8, wallet_size)
        };
        wallet.crc = crc32fast::hash(wallet_slice);

        let mut unlocked = flash.unlocked();
        unlocked.erase(WALLET_SECTOR).unwrap();
        unlocked.program(
            flash_wallet_addr as usize - SECTIOR_BASE, 
            repeat(wallet_slice.iter()).take(WALLET_REPEAT).flatten()
        ).unwrap();

        led.set_high();
    })
}

fn initialize_wallet(passcode: [u8; 8]) {
    let iv: [u8; 12] = update_global!(|mut rng: Option<RNG>, mut delay: Option<DELAY>| {
        let delay_time: u32 = rng.gen();
        delay.delay_us(delay_time % 4300);
        rng.gen()
    });

    let mut wallet = Wallet::new();
    
    wallet.chacha_iv = iv;

    let mut cipher = get_cipher(passcode, &iv);
    
    cipher.apply_keystream(&mut wallet.zone.zkmagic);
    initialize_accounts(&mut cipher, &mut wallet);
    // initialize OTP
    wallet.initialized = true;

    write_wallet(wallet);
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
        let points =  &key.verifying_key()
            .to_encoded_point(false);
        let pubkey_slice = &points.as_bytes()[1..];

        let mut pubkey: [u8; 64] = [0; 64]; 
        pubkey.copy_from_slice(pubkey_slice);

        keccak.update(pubkey_slice);
        let (_, addr): (GenericArray<u8, U12>, _) = keccak.finalize().split();
        
        ctx.zone.keys[i] = privkey;
        ctx.addrs[i] = addr.into();
        ctx.pubkeys[i] = pubkey;
    }
}
