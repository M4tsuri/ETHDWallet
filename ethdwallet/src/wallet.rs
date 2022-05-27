use core::{cmp::Ordering, intrinsics::size_of, ptr::addr_of, slice};

use chacha20::cipher::StreamCipher;
use cortex_m::{interrupt::free, prelude::*};
use rand::Rng;

use crate::{
    input::FIXED_KEY_LEN, 
    error::{Error, Result}, 
    update_global, 
    global::{CIPHER, RNG, DELAY}, set_global
};

use self::{safe_zone::{SafeZone, EthAddr, ZKPLAIN, Signature}, utils::get_cipher, initializer::write_wallet};

pub mod initializer;
pub mod safe_zone;
pub mod utils;

pub type PubKey = [u8; 64];

pub(super) const ACCOUNT_NUM: usize = 32;
/// length of OTP secret, which is randomly generated when initializing
pub(super) const OTP_SECRET_LEN: usize = 64;

pub const WALLET_SIZE: usize = size_of::<Wallet>();
pub const WALLET_REPEAT: usize = 128 * 1024 / size_of::<Wallet>();

/// This static variable is in the flash memory, so its persistent
/// The initial value will be changed after the wallet is initialized
#[link_section = ".wallet"]
#[no_mangle]
pub static mut WALLET: [Wallet; WALLET_REPEAT] = [Wallet { 
    initialized: false, 
    zone: SafeZone {
        zkmagic: ZKPLAIN,
        keys: [[0; 32]; ACCOUNT_NUM],
        otp_secret: [0; 64],
    },
    chacha_iv: [0; 12], 
    addrs: [[0; 20]; ACCOUNT_NUM],
    pubkeys: [[0; 64]; ACCOUNT_NUM],
    crc: 2004029866,
}; WALLET_REPEAT];

pub fn wallet() -> &'static Wallet {
    let mut crcs = [0; WALLET_REPEAT];

    unsafe {
        let mut valid_count = 0;

        for i in 0..WALLET_REPEAT {
            let mut wallet = WALLET[i];
            let crc = wallet.crc;
            wallet.crc = 0;
            let wallet_slice = slice::from_raw_parts(
                addr_of!(wallet) as *const u8, 
                WALLET_SIZE
            );

            if crc32fast::hash(wallet_slice) == crc {
                crcs[i] = crc;
                valid_count += 1;  
            }
            wallet.crc = crc;
        };

        if valid_count == 0 {
            panic!("cannot recover wallet data.")
        }

        let mut counter = [0; 5];

        for i in 0..5 {
            counter[i] = crcs.iter().filter(|crc| 
                (**crc != 0) && (**crc == crcs[i])
            ).count();
        }
        
        let (correct, count) = counter.iter().enumerate()
            .max_by(|a, b| 
                a.1.cmp(b.1) 
            ).unwrap();
        
        if *count == valid_count {
            return &WALLET[correct]
        }

        let wallet = WALLET[correct];

        write_wallet(wallet);
        &WALLET[0]
    }
}

pub const WALLET_SECTOR: u8 = 8;
pub const SECTIOR_BASE: usize = 0x0800_0000;

#[repr(align(16))]
#[derive(Clone, Copy)]
pub struct Wallet {
    pub initialized: bool,
    pub zone: SafeZone,
    /// the iv of chacha, randomly generated.
    /// this field is not encrypted 
    pub chacha_iv: [u8; 12],
    pub addrs: [EthAddr; ACCOUNT_NUM],
    pub pubkeys: [PubKey; ACCOUNT_NUM],
    pub crc: u32
}

impl Wallet {
    pub const fn new() -> Self {
        Self { 
            initialized: false, 
            zone: SafeZone {
                zkmagic: ZKPLAIN,
                keys: [[0; 32]; ACCOUNT_NUM],
                otp_secret: [0; 64],
            },
            chacha_iv: [0; 12], 
            addrs: [[0; 20]; ACCOUNT_NUM],
            pubkeys: [[0; 64]; ACCOUNT_NUM],
            crc: 0,
        }
    }

    pub fn sign_raw(&self, idx: usize, raw: &[u8]) -> Result<Signature> {
        update_global!(|
            mut cipher: Option<CIPHER>, 
            mut rng: Option<RNG>,
            mut delay: Option<DELAY>
        | {
            let delay_time: u32 = rng.gen();
            delay.delay_us(delay_time % 10000);
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


