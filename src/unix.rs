use crate::{Caps, Error};
use nusb::transfer;
use std::{mem, time::Duration};

pub struct DeviceInfo {
    inner: nusb::DeviceInfo,
}

impl DeviceInfo {
    pub fn enumerate() -> Result<Vec<Self>, Error> {
        Ok(nusb::list_devices()?
            .filter_map(|inner| {
                if inner
                    .interfaces()
                    .any(|inf| inf.class() == UsbClass::Video as _)
                {
                    Some(DeviceInfo { inner })
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn product_string(&self) -> Option<&str> {
        self.inner.product_string()
    }

    pub fn product_id(&self) -> u16 {
        self.inner.product_id()
    }

    pub fn vendor_id(&self) -> u16 {
        self.inner.vendor_id()
    }

    pub fn open(&self) -> Result<Device, Error> {
        let inner = self.inner.open()?;
        let Some(inf) = inner
            .configurations()
            .flat_map(|c| c.interface_alt_settings())
            .find(|i| {
                i.class() == UsbClass::Video as _ && i.subclass() == UsbClass::VideoControl as _
            })
        else {
            return Err(Error::InterfaceNotFound);
        };

        let mut it_unit = 0;
        let mut pu_unit = 0;

        for d in inf.descriptors() {
            if d[1] == DescriptorType::CSInterface as _ {
                if d[2] == DescriptorType::VCInputTerminal as _ && it_unit == 0 {
                    it_unit = d[3];
                } else if d[2] == DescriptorType::VCProcessingUnit as _ && pu_unit == 0 {
                    pu_unit = d[3];
                }
            }
        }

        Ok(Device {
            inf_no: inf.interface_number(),
            it_unit,
            pu_unit,
            inner,
        })
    }
}

pub struct Device {
    inf_no: u8,
    it_unit: u8,
    pu_unit: u8,
    inner: nusb::Device,
}

impl Device {
    pub fn get<const C: usize>(
        &self,
        req: Request,
        control_code: u8,
        unit: u8,
    ) -> Result<[u8; C], Error> {
        let mut data = [0; C];
        let _ = self.inner.control_in_blocking(
            transfer::Control {
                control_type: transfer::ControlType::Class,
                recipient: transfer::Recipient::Interface,
                request: req as _,
                value: (control_code as u16) << 8,
                index: (unit as u16) << 8 | self.inf_no as u16,
            },
            &mut data,
            Duration::from_secs(1),
        )?;
        Ok(data)
    }

    pub fn set(&self, control_code: u8, unit: u8, data: &[u8]) -> Result<(), Error> {
        let _ = self.inner.control_out_blocking(
            transfer::Control {
                control_type: transfer::ControlType::Class,
                recipient: transfer::Recipient::Interface,
                request: Request::SetCur as _,
                value: (control_code as u16) << 8,
                index: (unit as u16) << 8 | self.inf_no as u16,
            },
            data,
            Duration::from_secs(1),
        )?;
        Ok(())
    }

    pub fn zoom_abs_caps(&self) -> Result<Caps, Error> {
        let min = self.get::<2>(Request::GetMin, Control::ZoomAbs as _, self.it_unit)?;
        let max = self.get::<2>(Request::GetMax, Control::ZoomAbs as _, self.it_unit)?;
        let res = self.get::<2>(Request::GetRes, Control::ZoomAbs as _, self.it_unit)?;
        let def = self.get::<2>(Request::GetDef, Control::ZoomAbs as _, self.it_unit)?;
        Ok(Caps {
            min: u16::from_ne_bytes(min) as _,
            max: u16::from_ne_bytes(max) as _,
            res: u16::from_ne_bytes(res) as _,
            def: u16::from_ne_bytes(def) as _,
        })
    }

    pub fn zoom_abs(&self) -> Result<i32, Error> {
        let cur = self.get::<2>(Request::GetCur, Control::ZoomAbs as _, self.it_unit)?;
        Ok(u16::from_ne_bytes(cur) as _)
    }

    pub fn zoom_abs_set(&self, value: i32) -> Result<(), Error> {
        let data: u16 = value as _;
        self.set(Control::ZoomAbs as _, self.it_unit, &data.to_ne_bytes())
    }

    pub fn zoom_rel_caps(&self) -> Result<Caps, Error> {
        let min = self.get::<3>(Request::GetMin, Control::ZoomRel as _, self.it_unit)?;
        let max = self.get::<3>(Request::GetMax, Control::ZoomRel as _, self.it_unit)?;
        let res = self.get::<3>(Request::GetRes, Control::ZoomRel as _, self.it_unit)?;
        let def = self.get::<3>(Request::GetDef, Control::ZoomRel as _, self.it_unit)?;
        Ok(Caps {
            min: min[0] as _,
            max: max[0] as _,
            res: res[0] as _,
            def: def[0] as _,
        })
    }

    pub fn zoom_rel(&self) -> Result<i32, Error> {
        let cur = self.get::<3>(Request::GetCur, Control::ZoomRel as _, self.it_unit)?;
        Ok(cur[0] as _)
    }

    pub fn zoom_rel_set(&self, value: i32) -> Result<(), Error> {
        let caps = self.zoom_rel_caps()?;
        let data = [value as u8, 1, caps.res as u8];
        self.set(Control::ZoomRel as _, self.it_unit, &data)
    }

    pub fn pan_abs_caps(&self) -> Result<Caps, Error> {
        let min = self.get::<8>(Request::GetMin, Control::PanTiltAbs as _, self.it_unit)?;
        let max = self.get::<8>(Request::GetMax, Control::PanTiltAbs as _, self.it_unit)?;
        let res = self.get::<8>(Request::GetRes, Control::PanTiltAbs as _, self.it_unit)?;
        let def = self.get::<8>(Request::GetDef, Control::PanTiltAbs as _, self.it_unit)?;
        Ok(Caps {
            min: i32::from_ne_bytes(min[..4].try_into().unwrap()),
            max: i32::from_ne_bytes(max[..4].try_into().unwrap()),
            res: i32::from_ne_bytes(res[..4].try_into().unwrap()),
            def: i32::from_ne_bytes(def[..4].try_into().unwrap()),
        })
    }

    pub fn pan_abs(&self) -> Result<i32, Error> {
        let cur = self.get::<8>(Request::GetCur, Control::PanTiltAbs as _, self.it_unit)?;
        Ok(i32::from_ne_bytes(cur[..4].try_into().unwrap()))
    }

    pub fn pan_abs_set(&self, value: i32) -> Result<(), Error> {
        let tilt_cur = self.tilt_abs()?;
        let data = [value, tilt_cur];
        let data: [u8; 8] = unsafe { mem::transmute(data) };
        self.set(Control::PanTiltAbs as _, self.it_unit, &data)
    }

    pub fn pan_rel_caps(&self) -> Result<Caps, Error> {
        let min = self.get::<4>(Request::GetMin, Control::PanTiltRel as _, self.it_unit)?;
        let max = self.get::<4>(Request::GetMax, Control::PanTiltRel as _, self.it_unit)?;
        let res = self.get::<4>(Request::GetRes, Control::PanTiltRel as _, self.it_unit)?;
        let def = self.get::<4>(Request::GetDef, Control::PanTiltRel as _, self.it_unit)?;
        Ok(Caps {
            min: min[0] as _,
            max: max[0] as _,
            res: res[0] as _,
            def: def[0] as _,
        })
    }

    pub fn pan_rel(&self) -> Result<i32, Error> {
        let cur = self.get::<4>(Request::GetCur, Control::PanTiltRel as _, self.it_unit)?;
        Ok(cur[0] as _)
    }

    pub fn pan_rel_set(&self, value: i32) -> Result<(), Error> {
        let pan_caps = self.pan_rel_caps()?;
        let tilt_caps = self.tilt_rel_caps()?;
        let tilt_cur = self.tilt_rel()?;
        let data = [
            value as u8,
            pan_caps.res as u8,
            tilt_cur as u8,
            tilt_caps.res as u8,
        ];
        self.set(Control::PanTiltRel as _, self.it_unit, &data)
    }

    pub fn tilt_abs_caps(&self) -> Result<Caps, Error> {
        let min = self.get::<8>(Request::GetMin, Control::PanTiltAbs as _, self.it_unit)?;
        let max = self.get::<8>(Request::GetMax, Control::PanTiltAbs as _, self.it_unit)?;
        let res = self.get::<8>(Request::GetRes, Control::PanTiltAbs as _, self.it_unit)?;
        let def = self.get::<8>(Request::GetDef, Control::PanTiltAbs as _, self.it_unit)?;
        Ok(Caps {
            min: i32::from_ne_bytes(min[4..].try_into().unwrap()),
            max: i32::from_ne_bytes(max[4..].try_into().unwrap()),
            res: i32::from_ne_bytes(res[4..].try_into().unwrap()),
            def: i32::from_ne_bytes(def[4..].try_into().unwrap()),
        })
    }

    pub fn tilt_abs(&self) -> Result<i32, Error> {
        let cur = self.get::<8>(Request::GetCur, Control::PanTiltAbs as _, self.it_unit)?;
        Ok(i32::from_ne_bytes(cur[4..].try_into().unwrap()))
    }

    pub fn tilt_abs_set(&self, value: i32) -> Result<(), Error> {
        let pan_cur = self.pan_abs()?;
        let data = [pan_cur, value];
        let data: [u8; 8] = unsafe { mem::transmute(data) };
        self.set(Control::PanTiltAbs as _, self.it_unit, &data)
    }

    pub fn tilt_rel_caps(&self) -> Result<Caps, Error> {
        let min = self.get::<4>(Request::GetMin, Control::PanTiltRel as _, self.it_unit)?;
        let max = self.get::<4>(Request::GetMax, Control::PanTiltRel as _, self.it_unit)?;
        let res = self.get::<4>(Request::GetRes, Control::PanTiltRel as _, self.it_unit)?;
        let def = self.get::<4>(Request::GetDef, Control::PanTiltRel as _, self.it_unit)?;
        Ok(Caps {
            min: min[2] as _,
            max: max[2] as _,
            res: res[2] as _,
            def: def[2] as _,
        })
    }

    pub fn tilt_rel(&self) -> Result<i32, Error> {
        let cur = self.get::<4>(Request::GetCur, Control::PanTiltRel as _, self.it_unit)?;
        Ok(cur[2] as _)
    }

    pub fn tilt_rel_set(&self, value: i32) -> Result<(), Error> {
        let pan_caps = self.pan_rel_caps()?;
        let pan_cur = self.pan_rel()?;
        let tilt_caps = self.tilt_rel_caps()?;
        let data = [
            pan_cur as u8,
            pan_caps.res as u8,
            value as u8,
            tilt_caps.res as u8,
        ];
        self.set(Control::PanTiltRel as _, self.it_unit, &data)
    }
}

#[repr(u8)]
pub enum Request {
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

#[repr(u8)]
enum DescriptorType {
    CSInterface = 0x24,
    VCInputTerminal = 0x02,
    VCProcessingUnit = 0x05,
}
