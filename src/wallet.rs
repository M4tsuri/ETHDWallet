use chacha20::ChaCha20;
use cortex_m::singleton;

use self::safe_zone::{SafeZone, EthAddr};

pub mod initializer;
pub mod safe_zone;
pub mod utils;

pub(super) const ACCOUNT_NUM: usize = 32;
/// length of OTP secret, which is randomly generated when initializing
pub(super) const OTP_SECRET_LEN: usize = 64;

pub struct Wallet {
    pub initialized: bool,
    pub zone: &'static mut SafeZone,
    /// the iv of chacha, randomly generated.
    /// this field is not encrypted 
    pub chacha_iv: [u8; 12],
    pub addrs: [EthAddr; ACCOUNT_NUM],
    pub cipher: Option<ChaCha20>
}

impl Wallet {
    /// load wallet from flash
    pub fn load() -> Self {
        Self {
            initialized: todo!(),
            zone: singleton!(: 
                SafeZone = SafeZone::load()
            ).unwrap(), 
            chacha_iv: todo!(), 
            addrs: todo!(),
            cipher: None
        }
    }

    /// save this wallet to flash
    pub fn save(&self) {
        todo!()
    } 
}
