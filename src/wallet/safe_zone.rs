use core::mem::size_of;

use chacha20::{cipher::{StreamCipher, StreamCipherSeek}, ChaCha20};

use crate::error::Error;
use super::{OTP_SECRET_LEN, ACCOUNT_NUM};

/// plaintext of the zkmagic field in encrypted safe zone
pub const ZKPLAIN: [u8; 32] = [
    0x11, 0x45, 0x14, 0x19, 0x19, 0x81, 0x00, 0x0a,
    0x62, 0x79, 0x5f, 0x6d, 0x34, 0x74, 0x73, 0x75,
    0x72, 0x69, 0x45, 0x54, 0x48, 0x44, 0x57, 0x61, 
    0x6c, 0x6c, 0x65, 0x74, 0x0a, 0xf0, 0x9f, 0x94
];

pub type PrivKey = [u8; 32];
pub type EthAddr = [u8; 20];

#[derive(Clone, Copy)]
pub struct SafeZone {
    // this magic allow us to decrypt the safe zone without knowing the passcode
    // this field is zero for an uninitialized wallet
    pub zkmagic: [u8; 32],
    // accounts as bytes
    pub keys: [PrivKey; ACCOUNT_NUM],
    pub otp_secret: [u8; OTP_SECRET_LEN],
}

#[derive(Clone, Copy)]
pub struct Signature {
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub v: u8
}

impl SafeZone {
    /// sign a raw transaction, returns the signature
    /// the cipher is guaranteed to be correct
    pub(super) fn sign_raw(
        &self, idx: usize, raw: &[u8], cipher: &mut ChaCha20
    ) -> Result<Signature, Error> {
        use k256::ecdsa::{
            SigningKey,
            recoverable::Signature as RSignature
        };
        use k256::ecdsa::signature::Signer;

        // recover signing key
        let offset = 32 + size_of::<PrivKey>() * idx;
        let mut key = *self.keys.get(idx)
            .ok_or(Error::AccountIdxOOB)?;
        cipher.seek(offset);
        cipher.apply_keystream(&mut key);
        let sign_key = SigningKey::from_bytes(&key)?;

        // sign digest
        let sig: RSignature = sign_key.try_sign(raw)?;

        let pubkey = sig.recover_verifying_key(raw)?;
        if pubkey != sign_key.verifying_key() {
            return Err(Error::CryptoError)
        }

        Ok(Signature { 
            r: sig.r().to_bytes().into(), 
            s: sig.s().to_bytes().into(), 
            v: sig.recovery_id().into()
        })
    }
}
