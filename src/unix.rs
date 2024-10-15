use std::{mem, time::Duration};

use crate::{Error, Caps, RelOperation};
use nusb::transfer;

pub struct DeviceInfo {
    inner: nusb::DeviceInfo,
}

impl DeviceInfo {
    pub fn enumerate() -> Result<Vec<Self>, Error> {
        Ok(nusb::list_devices()?.map(|inner| DeviceInfo { inner }).collect())
    }

    pub fn product_id(&self) -> u16 {
        self.inner.product_id()
    }

    pub fn vendor_id(&self) -> u16 {
        self.inner.vendor_id()
    }

    pub fn open(&self) -> Result<Device, Error> {
        let inner = self.inner.open()?;
        let Some(inf) = self.inner.interfaces().find(|inf| inf.class() == UsbClass::Video as _ && inf.subclass() == UsbClass::VideoControl as _) else {
            return Err(Error::InterfaceNotFound);
        };
        Ok(Device { inner, inf: inf.clone() })
    }
}

pub struct Device {
    inner: nusb::Device,
    inf: nusb::InterfaceInfo,
}

impl Device {
    fn caps<const C: usize>(&self, req: Request, control: Control, unit: u8) -> Result<[u8; C], Error> {
        let mut data = [0; C];
        let _ = self.inner.control_in_blocking(transfer::Control {
            control_type: transfer::ControlType::Class,
            recipient: transfer::Recipient::Interface,
            request: req as _,
            value: (control as u16) << 8,
            index: (unit as u16) << 8 | self.inf.interface_number() as u16,
        }, &mut data, Duration::from_secs(1))?;
        Ok(data)
    }

    pub fn zoom_abs_caps(&self, unit: u8) -> Result<Caps, Error> {
        let min = self.caps::<2>(Request::GetMin, Control::ZoomAbs, unit)?;
        let max = self.caps::<2>(Request::GetMax, Control::ZoomAbs, unit)?;
        let step = self.caps::<2>(Request::GetRes, Control::ZoomAbs, unit)?;
        let def = self.caps::<2>(Request::GetDef, Control::ZoomAbs, unit)?;
        let cur = self.caps::<2>(Request::GetCur, Control::ZoomAbs, unit)?;
        Ok(Caps {
            min: u16::from_ne_bytes(min) as _,
            max: u16::from_ne_bytes(max) as _,
            step: u16::from_ne_bytes(step) as _,
            def: u16::from_ne_bytes(def) as _,
            cur: u16::from_ne_bytes(cur) as _,
        })
    }

    pub fn zoom_rel_caps(&self, unit: u8) -> Result<Caps, Error> {
        let min = self.caps::<3>(Request::GetMin, Control::ZoomRel, unit)?;
        let max = self.caps::<3>(Request::GetMax, Control::ZoomRel, unit)?;
        let step = self.caps::<3>(Request::GetRes, Control::ZoomRel, unit)?;
        let def = self.caps::<3>(Request::GetDef, Control::ZoomRel, unit)?;
        let cur = self.caps::<3>(Request::GetCur, Control::ZoomRel, unit)?;
        Ok(Caps {
            min: min[0] as _,
            max: max[0] as _,
            step: step[0] as _,
            def: def[0] as _,
            cur: cur[0] as _,
        })
    }

    pub fn pan_abs_caps(&self, unit: u8) -> Result<Caps, Error> {
        let min = self.caps::<8>(Request::GetMin, Control::PanTiltAbs, unit)?;
        let max = self.caps::<8>(Request::GetMax, Control::PanTiltAbs, unit)?;
        let step = self.caps::<8>(Request::GetRes, Control::PanTiltAbs, unit)?;
        let def = self.caps::<8>(Request::GetDef, Control::PanTiltAbs, unit)?;
        let cur = self.caps::<8>(Request::GetCur, Control::PanTiltAbs, unit)?;
        Ok(Caps {
            min: i32::from_ne_bytes(min[..4].try_into().unwrap()),
            max: i32::from_ne_bytes(max[..4].try_into().unwrap()),
            step: i32::from_ne_bytes(step[..4].try_into().unwrap()),
            def: i32::from_ne_bytes(def[..4].try_into().unwrap()),
            cur: i32::from_ne_bytes(cur[..4].try_into().unwrap()),
        })
    }

    pub fn pan_rel_caps(&self, unit: u8) -> Result<Caps, Error> {
        let min = self.caps::<4>(Request::GetMin, Control::PanTiltRel, unit)?;
        let max = self.caps::<4>(Request::GetMax, Control::PanTiltRel, unit)?;
        let step = self.caps::<4>(Request::GetRes, Control::PanTiltRel, unit)?;
        let def = self.caps::<4>(Request::GetDef, Control::PanTiltRel, unit)?;
        let cur = self.caps::<4>(Request::GetCur, Control::PanTiltRel, unit)?;
        Ok(Caps {
            min: min[0] as _,
            max: max[0] as _,
            step: step[0] as _,
            def: def[0] as _,
            cur: cur[0] as _,
        })
    }

    pub fn tilt_abs_caps(&self, unit: u8) -> Result<Caps, Error> {
        let min = self.caps::<8>(Request::GetMin, Control::PanTiltAbs, unit)?;
        let max = self.caps::<8>(Request::GetMax, Control::PanTiltAbs, unit)?;
        let step = self.caps::<8>(Request::GetRes, Control::PanTiltAbs, unit)?;
        let def = self.caps::<8>(Request::GetDef, Control::PanTiltAbs, unit)?;
        let cur = self.caps::<8>(Request::GetCur, Control::PanTiltAbs, unit)?;
        Ok(Caps {
            min: i32::from_ne_bytes(min[4..].try_into().unwrap()),
            max: i32::from_ne_bytes(max[4..].try_into().unwrap()),
            step: i32::from_ne_bytes(step[4..].try_into().unwrap()),
            def: i32::from_ne_bytes(def[4..].try_into().unwrap()),
            cur: i32::from_ne_bytes(cur[4..].try_into().unwrap()),
        })
    }

    pub fn tilt_rel_caps(&self, unit: u8) -> Result<Caps, Error> {
        let min = self.caps::<4>(Request::GetMin, Control::PanTiltRel, unit)?;
        let max = self.caps::<4>(Request::GetMax, Control::PanTiltRel, unit)?;
        let step = self.caps::<4>(Request::GetRes, Control::PanTiltRel, unit)?;
        let def = self.caps::<4>(Request::GetDef, Control::PanTiltRel, unit)?;
        let cur = self.caps::<4>(Request::GetCur, Control::PanTiltRel, unit)?;
        Ok(Caps {
            min: min[2] as _,
            max: max[2] as _,
            step: step[2] as _,
            def: def[2] as _,
            cur: cur[2] as _,
        })
    }

    pub fn auto_focus_caps(&self, unit: u8) -> Result<Caps, Error> {
        let min = self.caps::<1>(Request::GetMin, Control::AutoFocus, unit)?;
        let max = self.caps::<1>(Request::GetMax, Control::AutoFocus, unit)?;
        let step = self.caps::<1>(Request::GetRes, Control::AutoFocus, unit)?;
        let def = self.caps::<1>(Request::GetDef, Control::AutoFocus, unit)?;
        let cur = self.caps::<1>(Request::GetCur, Control::AutoFocus, unit)?;
        Ok(Caps {
            min: min[0] as _,
            max: max[0] as _,
            step: step[0] as _,
            def: def[0] as _,
            cur: cur[0] as _,
        })
    }

    fn set(&self, req: Request, control: Control, unit: u8, data: &[u8]) -> Result<(), Error> {
        let _ = self.inner.control_out_blocking(transfer::Control {
            control_type: transfer::ControlType::Class,
            recipient: transfer::Recipient::Interface,
            request: req as _,
            value: (control as u16) << 8,
            index: (unit as u16) << 8 | self.inf.interface_number() as u16,
        }, data, Duration::from_secs(1))?;
        Ok(())
    }

    pub fn zoom_abs(&self, value: i32, unit: u8) -> Result<(), Error> {
        let data: u16 = value as _;
        self.set(Request::SetCur, Control::ZoomAbs, unit, &data.to_ne_bytes())
    }

    pub fn zoom_rel(&self, operation: RelOperation, unit: u8) -> Result<(), Error> {
        let caps = self.zoom_rel_caps(unit)?;
        let d: i8 = match operation {
            RelOperation::Passitive => 1,
            RelOperation::Negative => -1,
            RelOperation::Stop => 0,
        };
        let data = [d as u8, 1, caps.step as u8];
        self.set(Request::SetCur, Control::ZoomRel, unit, &data)
    }

    pub fn pan_abs(&self, value: i32, unit: u8) -> Result<(), Error> {
        let tilt_caps = self.tilt_abs_caps(unit)?;
        let data = [value, tilt_caps.cur];
        let data: [u8; 8] = unsafe { mem::transmute(data) };
        self.set(Request::SetCur, Control::PanTiltAbs, unit, &data)
    }

    pub fn pan_rel(&self, operation: RelOperation, unit: u8) -> Result<(), Error> {
        let pan_caps = self.pan_rel_caps(unit)?;
        let tilt_caps = self.tilt_rel_caps(unit)?;
        let d: i8 = match operation {
            RelOperation::Passitive => 1,
            RelOperation::Negative => -1,
            RelOperation::Stop => 0,
        };
        let data = [d as u8, pan_caps.step as u8, tilt_caps.cur as u8, tilt_caps.step as u8];
        self.set(Request::SetCur, Control::PanTiltRel, unit, &data)
    }

    pub fn tilt_abs(&self, value: i32, unit: u8) -> Result<(), Error> {
        let pan_caps = self.pan_abs_caps(unit)?;
        let data = [pan_caps.cur, value];
        let data: [u8; 8] = unsafe { mem::transmute(data) };
        self.set(Request::SetCur, Control::PanTiltAbs, unit, &data)
    }

    pub fn tilt_rel(&self, operation: RelOperation, unit: u8) -> Result<(), Error> {
        let pan_caps = self.pan_rel_caps(unit)?;
        let tilt_caps = self.tilt_rel_caps(unit)?;
        let d: i8 = match operation {
            RelOperation::Passitive => 1,
            RelOperation::Negative => -1,
            RelOperation::Stop => 0,
        };
        let data = [pan_caps.cur as u8, pan_caps.step as u8, d as u8, tilt_caps.step as u8];
        self.set(Request::SetCur, Control::PanTiltRel, unit, &data)
    }

    pub fn auto_focus(&self, data: &[u8], unit: u8) -> Result<(), Error> {
        self.set(Request::SetCur, Control::AutoFocus, unit, data)
    }
}

#[repr(u8)]
enum Request {
    SetCur = 0x01,
    GetCur = 0x81,
    GetMin = 0x82,
    GetMax = 0x83,
    GetRes = 0x84,
    GetLen = 0x85,
    GetInfo = 0x86,
    GetDef = 0x87,
}

#[repr(u8)]
enum Control {
    AutoFocus = 0x02,
    ZoomAbs = 0x0b,
    ZoomRel = 0x0c,
    PanTiltAbs = 0x0d,
    PanTiltRel = 0x0e,
}

#[repr(u8)]
enum UsbClass {
    Video = 0x0e,
    VideoControl = 0x01,
}
