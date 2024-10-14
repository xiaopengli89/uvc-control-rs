#[cfg(windows)]
pub use windows::{Device, DeviceInfo};

#[cfg(windows)]
mod windows;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Win(#[from] ::windows::core::Error),
}

pub fn foo() {
    let devices = nusb::list_devices().unwrap();
    for info in devices {
        // info.interfaces();
        // let dev = info.open().unwrap();
        // let inf = dev.claim_interface(0).unwrap();
        // inf.control_out_blocking(control, data, timeout);
    }
}
