use stm32f4xx_hal::{
    serial::{
        config::InvalidConfig,
        self
    }, 
    nb
};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Error {
    HalInitError,
    AccountIdxOOB,
    CryptoError,
    InvalidSerialConfig,
    InvalidInstruction,
    SerialDataCorrupted,
    WrongPassword,
    SerialTxError
}

impl From<&mut Error> for Error  {
    fn from(e: &mut Error) -> Self {
        *e
    }
}

impl From<serial::Error> for Error {
    fn from(_: serial::Error) -> Self {
        Self::SerialTxError
    }
}

impl From<k256::ecdsa::Error> for Error {
    fn from(_: k256::ecdsa::Error) -> Self {
        Self::CryptoError
    }
}

impl From<InvalidConfig> for Error {
    fn from(_: InvalidConfig) -> Self {
        Self::InvalidSerialConfig
    }
}

pub type Result<T> = core::result::Result<T, Error>;
