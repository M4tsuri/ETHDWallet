use stm32f4xx_hal::serial::config::InvalidConfig;

#[repr(u8)]
pub enum Error {
    HalInitError,
    AccountIdxOOB,
    CryptoError,
    InvalidSerialConfig,
    InvalidInstruction,
    SerialDataCorrupted
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
