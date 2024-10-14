#[cfg(windows)]
pub use windows::{Device, DeviceInfo};

#[cfg(windows)]
mod windows;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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

pub fn foo() {
    let devices = nusb::list_devices().unwrap();
    for _info in devices {
        // info.interfaces();
        // let dev = info.open().unwrap();
        // let inf = dev.claim_interface(0).unwrap();
        // inf.control_out_blocking(control, data, timeout);
    }
}
