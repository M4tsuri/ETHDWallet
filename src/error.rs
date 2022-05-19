pub enum Error {
    HalInitError,
    AccountIdxOOB,
    CryptoError
}

impl From<k256::ecdsa::Error> for Error {
    fn from(_: k256::ecdsa::Error) -> Self {
        Self::CryptoError
    }
}

pub type Result<T> = core::result::Result<T, Error>;
