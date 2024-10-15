#[cfg(windows)]
pub use windows::{Device, DeviceInfo};
#[cfg(unix)]
pub use unix::{Device, DeviceInfo};

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(unix)]
    #[error("{0}")]
    Usb(#[from] nusb::Error),
    #[cfg(unix)]
    #[error("interface not found")]
    InterfaceNotFound,
    #[cfg(unix)]
    #[error("{0}")]
    UbsTransfer(#[from] nusb::transfer::TransferError),
    #[cfg(windows)]
    #[error("{0}")]
    Win(#[from] ::windows::core::Error),
}

#[derive(Debug)]
pub enum RelOperation {
    Passitive,
    Negative,
    Stop,
}

#[derive(Debug)]
pub struct Caps {
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub def: i32,
    pub cur: i32,
}
