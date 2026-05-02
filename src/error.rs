#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NoMadt,
    AddrTooHigh,
    AddrNotAligned,
}

pub type Result<T = ()> = core::result::Result<T, Error>;
