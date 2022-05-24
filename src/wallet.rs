use core::cmp::Ordering;

use chacha20::cipher::StreamCipher;
use cortex_m::{interrupt::free};

use crate::{
    input::FIXED_KEY_LEN, 
    error::{Error, Result}, 
    update_global, 
    global::CIPHER, set_global
};

use self::{safe_zone::{SafeZone, EthAddr, ZKPLAIN, Signature}, utils::get_cipher};

pub mod initializer;
pub mod safe_zone;
pub mod utils;

pub(super) const ACCOUNT_NUM: usize = 32;
/// length of OTP secret, which is randomly generated when initializing
pub(super) const OTP_SECRET_LEN: usize = 64;

/// This static variable is in the flash memory, so its persistent
/// The initial value will be changed after the wallet is initialized
#[link_section = ".wallet"]
#[no_mangle]
pub static WALLET: Wallet = Wallet { 
    initialized: false, 
    zone: SafeZone {
        zkmagic: ZKPLAIN,
        keys: [[0; 32]; ACCOUNT_NUM],
        otp_secret: [0; 64],
    }, 
    chacha_iv: [0; 12], 
    addrs: [[0; 20]; ACCOUNT_NUM]
};

pub const WALLET_SECTOR: u8 = 8;

#[repr(align(16))]
#[derive(Clone, Copy)]
pub struct Wallet {
    pub initialized: bool,
    pub zone: SafeZone,
    /// the iv of chacha, randomly generated.
    /// this field is not encrypted 
    pub chacha_iv: [u8; 12],
    pub addrs: [EthAddr; ACCOUNT_NUM],
}

impl Wallet {
    pub fn sign_raw(&self, idx: usize, raw: &[u8]) -> Result<Signature> {
        update_global!(|mut cipher: Option<CIPHER>| {
            self.zone.sign_raw(idx, raw, &mut cipher)
        })
    }

    /// verify if passcode is correct
    /// fill the cipher in wallet
    pub fn fill_cipher(&self, passcode: [u8; FIXED_KEY_LEN]) -> Result<()> {
        let mut cipher = get_cipher(passcode, &self.chacha_iv);
        let mut zkmagic = self.zone.zkmagic;
        cipher.apply_keystream(&mut zkmagic);

        let cipher = match zkmagic.cmp(&ZKPLAIN) {
            Ordering::Equal => cipher,
            _ => return Err(Error::WrongPassword)
        };

        free(|cs| {
            set_global!(CIPHER, cipher, cs);
        });
        
        Ok(())
    }
}


