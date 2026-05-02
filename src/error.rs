#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NoMadt,
    TrampolineTooHigh,
    TrampolineNotAligned,
    L4TableAddrTooHigh,
}

pub type Result<T = ()> = core::result::Result<T, Error>;
