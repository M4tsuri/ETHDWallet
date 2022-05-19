mod initializer;
mod safe_zone;
mod utils;

pub(super) const ACCOUNT_NUM: usize = 32;
/// length of OTP secret, which is randomly generated when initializing
pub(super) const OTP_SECRET_LEN: usize = 64;

