pub enum Error {
    HalInitError
}

pub type Result<T> = core::result::Result<T, Error>;
