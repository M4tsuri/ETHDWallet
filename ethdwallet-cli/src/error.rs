use num::bigint::ParseBigIntError;
use num_enum::FromPrimitive;

#[derive(Debug)]
pub enum Error {
    InvalidHexMsg,
    InvalidValue,
    SerialCorrupted,
    SerialTimeout(serialport::Error),
    IoError(std::io::Error),
    WalletError(WalletError),
    Web3Error(web3::Error),
    ErrorAddressFormat,
    RlpError(serlp::error::Error)
}

impl From<serlp::error::Error> for Error {
    fn from(e: serlp::error::Error) -> Self {
        Self::RlpError(e)
    }
}

impl From<ParseBigIntError> for Error {
    fn from(_: ParseBigIntError) -> Self {
        Self::InvalidValue
    }
}

impl From<web3::Error> for Error {
    fn from(e: web3::Error) -> Self {
        Self::Web3Error(e)
    }
}

impl From<serialport::Error> for Error {
    fn from(e: serialport::Error) -> Self {
        Self::SerialTimeout(e)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(_: hex::FromHexError) -> Self {
        Self::InvalidHexMsg
    }
}


impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum WalletError {
    HalInitError,
    AccountIdxOOB,
    CryptoError,
    InvalidSerialConfig,
    InvalidInstruction,
    SerialDataCorrupted,
    WrongPassword,
    SerialTxError,
    #[num_enum(default)]
    UnknownError
}
