#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NoMadt,
    TrampolineTooHigh,
    TrampolineNotAligned,
    L4TableAddrTooHigh,
    // Timed out while waiting for ap to start
    StartupTimeout,
}

pub type Result<T = ()> = core::result::Result<T, Error>;
