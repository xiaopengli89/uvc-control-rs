use crate::{Caps, Error};
use std::{mem, ptr};
use windows::core::{Interface, GUID, HSTRING, PWSTR};
use windows::Win32::Devices::Usb;
use windows::Win32::Foundation;
use windows::Win32::Media::{DirectShow, KernelStreaming, MediaFoundation};
use windows::Win32::System::{Com, IO};

pub struct DeviceInfo {
    inner: MediaFoundation::IMFActivate,
    product_string: Option<String>,
    product_id: u16,
    vendor_id: u16,
    symbolic_link: String,
}

unsafe impl Send for DeviceInfo {}

unsafe impl Sync for DeviceInfo {}

impl DeviceInfo {
    pub fn enumerate() -> Result<Vec<Self>, Error> {
        let mut apt_type = Com::APTTYPE::default();
        let mut apt_qualifier = Com::APTTYPEQUALIFIER::default();
        if unsafe { Com::CoGetApartmentType(&mut apt_type, &mut apt_qualifier) }.is_err() {
            unsafe { Com::CoInitializeEx(None, Com::COINIT_MULTITHREADED) }.ok()?;
        }

        let mut attr = None;
        unsafe { MediaFoundation::MFCreateAttributes(&mut attr, 1) }?;
        let attr = attr.unwrap();
        unsafe {
            attr.SetGUID(
                &MediaFoundation::MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                &MediaFoundation::MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
            )
        }?;

        let mut list = ptr::null_mut();
        let mut count = 0;
        unsafe { MediaFoundation::MFEnumDeviceSources(&attr, &mut list, &mut count) }?;

        let re_pid = regex::Regex::new("[Pp][Ii][Dd]_([A-Fa-f0-9]+)").unwrap();
        let re_vid = regex::Regex::new("[Vv][Ii][Dd]_([A-Fa-f0-9]+)").unwrap();

        let mut device_infos = Vec::with_capacity(count as _);

        for i in 0..count as usize {
            let inner = unsafe { ptr::read(list.add(i)) }.unwrap();

            let get_string = |guid: &GUID| {
                let mut ws = PWSTR::null();
                let mut len = 0;
                if unsafe { inner.GetAllocatedString(guid, &mut ws, &mut len) }.is_err() {
                    return None;
                }

                let s = unsafe { ws.to_string() };
                unsafe { Com::CoTaskMemFree(Some(ws.as_ptr() as _)) };
                s.ok()
            };

            let id = get_string(
                &MediaFoundation::MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
            );
            let Some(id) = id else {
                continue;
            };

            let product_id = re_pid
                .captures(&id)
                .and_then(|caps| caps.get(1))
                .and_then(|m| u16::from_str_radix(m.as_str(), 16).ok())
                .unwrap_or_default();
            let vendor_id = re_vid
                .captures(&id)
                .and_then(|caps| caps.get(1))
                .and_then(|m| u16::from_str_radix(m.as_str(), 16).ok())
                .unwrap_or_default();

            let product_string = get_string(&MediaFoundation::MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME);

            device_infos.push(Self {
                inner,
                product_string,
                product_id,
                vendor_id,
                symbolic_link: id,
            });
        }

        unsafe { Com::CoTaskMemFree(Some(list as _)) };

        Ok(device_infos)
    }

    pub fn product_string(&self) -> Option<&str> {
        self.product_string.as_deref()
    }

    pub fn product_id(&self) -> u16 {
        self.product_id
    }

    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    pub fn symbolic_link(&self) -> &str {
        &self.symbolic_link
    }

    pub fn get_descriptor(
        &self,
        h: Foundation::HANDLE,
        connection_index: u32,
    ) -> Result<(), Error> {
        let req = Usb::USB_DESCRIPTOR_REQUEST {
            ConnectionIndex: connection_index,
            SetupPacket: Usb::USB_DESCRIPTOR_REQUEST_0 {
                bmRequest: 0x80, // Standard, IN
                bRequest: 6,     // GET_DESCRIPTOR
                wValue: (Usb::USB_CONFIGURATION_DESCRIPTOR_TYPE as u16) << 8,
                wIndex: 0,
                wLength: 4 << 10, // 4KB
            },
            Data: [0],
        };
        let mut buffer = vec![0u8; mem::size_of_val(&req) + req.SetupPacket.wLength as usize];
        unsafe {
            ptr::copy_nonoverlapping(<*const _>::cast(&req), &mut buffer, mem::size_of_val(&req))
        };

        let mut bytes_returned = 0;
        unsafe {
            IO::DeviceIoControl(
                h,
                Usb::IOCTL_USB_GET_DESCRIPTOR_FROM_NODE_CONNECTION,
                Some(<*const _>::cast(&buffer)),
                buffer.len() as _,
                Some(<*mut _>::cast(&mut buffer)),
                buffer.len() as _,
                Some(&mut bytes_returned),
                None,
            )
        }?;

        let offset = 12;
        let data = &buffer[offset..offset + bytes_returned as usize];

        // parse

        Ok(())
    }

    pub fn open(&self) -> Result<Device, Error> {
        let source: MediaFoundation::IMFMediaSource = unsafe { self.inner.ActivateObject() }?;
        let topology_info: KernelStreaming::IKsTopologyInfo = source.cast()?;
        let num_nodes = unsafe { topology_info.NumNodes() }?;
        let ks_control: KernelStreaming::IKsControl = source.cast()?;
        let am_control: DirectShow::IAMCameraControl = source.cast()?;

        Ok(Device {
            num_nodes,
            ks_control,
            am_control,
        })
    }
}

pub struct Device {
    num_nodes: u32,
    ks_control: KernelStreaming::IKsControl,
    am_control: DirectShow::IAMCameraControl,
}

unsafe impl Send for Device {}

unsafe impl Sync for Device {}

impl Device {
    pub fn caps(&self, control_code: i32) -> Result<Caps, Error> {
        let mut min = 0;
        let mut max = 0;
        let mut res = 0;
        let mut def = 0;
        let mut flags = 0;
        unsafe {
            self.am_control.GetRange(
                control_code,
                &mut min,
                &mut max,
                &mut res,
                &mut def,
                &mut flags,
            )
        }?;
        Ok(Caps { min, max, res, def })
    }

    pub fn get(&self, control_code: i32) -> Result<i32, Error> {
        let mut cur = 0;
        let mut flags = 0;
        unsafe { self.am_control.Get(control_code, &mut cur, &mut flags) }?;
        Ok(cur)
    }

    pub fn set(&self, control_code: i32, value: i32) -> Result<(), Error> {
        Ok(unsafe {
            self.am_control.Set(
                control_code,
                value,
                DirectShow::CameraControl_Flags_Manual.0,
            )
        }?)
    }

    pub fn get_xu<const C: usize>(&self, set: &str, id: u32) -> Result<[u8; C], Error> {
        let mut data = [0u8; C];

        let mut property = KernelStreaming::KSP_NODE::default();
        property.Property.Anonymous.Anonymous.Set =
            unsafe { Com::CLSIDFromString(&HSTRING::from(set)) }?;
        property.Property.Anonymous.Anonymous.Id = id;
        property.Property.Anonymous.Anonymous.Flags =
            KernelStreaming::KSPROPERTY_TYPE_GET | KernelStreaming::KSPROPERTY_TYPE_TOPOLOGY;

        for node_id in 0..self.num_nodes {
            property.NodeId = node_id;

            let mut r = 0;
            if unsafe {
                self.ks_control.KsProperty(
                    &property.Property,
                    mem::size_of_val(&property) as _,
                    data.as_mut_ptr() as _,
                    mem::size_of_val(&data) as _,
                    &mut r,
                )
            }
            .is_ok()
            {
                break;
            };
        }

        Ok(data)
    }

    // {a8bd5df2-1a98-474e-8dd0-d92672d194fa}, 2, [2]
    pub fn set_xu(&self, set: &str, id: u32, data: &mut [u8]) -> Result<(), Error> {
        let mut property = KernelStreaming::KSP_NODE::default();
        property.Property.Anonymous.Anonymous.Set =
            unsafe { Com::CLSIDFromString(&HSTRING::from(set)) }?;
        property.Property.Anonymous.Anonymous.Id = id;
        property.Property.Anonymous.Anonymous.Flags =
            KernelStreaming::KSPROPERTY_TYPE_SET | KernelStreaming::KSPROPERTY_TYPE_TOPOLOGY;

        for node_id in 0..self.num_nodes {
            property.NodeId = node_id;

            let mut r = 0;
            if unsafe {
                self.ks_control.KsProperty(
                    &property.Property,
                    mem::size_of_val(&property) as _,
                    data.as_mut_ptr() as _,
                    mem::size_of_val(data) as _,
                    &mut r,
                )
            }
            .is_ok()
            {
                break;
            };
        }

        Ok(())
    }

    pub fn zoom_abs_caps(&self) -> Result<Caps, Error> {
        self.caps(KernelStreaming::KSPROPERTY_CAMERACONTROL_ZOOM.0)
    }

    pub fn zoom_abs(&self) -> Result<i32, Error> {
        self.get(KernelStreaming::KSPROPERTY_CAMERACONTROL_ZOOM.0)
    }

    pub fn zoom_abs_set(&self, value: i32) -> Result<(), Error> {
        self.set(KernelStreaming::KSPROPERTY_CAMERACONTROL_ZOOM.0, value)
    }

    pub fn zoom_rel_caps(&self) -> Result<Caps, Error> {
        self.caps(KernelStreaming::KSPROPERTY_CAMERACONTROL_ZOOM_RELATIVE.0)
    }

    pub fn zoom_rel(&self) -> Result<i32, Error> {
        self.get(KernelStreaming::KSPROPERTY_CAMERACONTROL_ZOOM_RELATIVE.0)
    }

    pub fn zoom_rel_set(&self, value: i32) -> Result<(), Error> {
        self.set(
            KernelStreaming::KSPROPERTY_CAMERACONTROL_ZOOM_RELATIVE.0,
            value,
        )
    }

    pub fn pan_abs_caps(&self) -> Result<Caps, Error> {
        self.caps(KernelStreaming::KSPROPERTY_CAMERACONTROL_PAN.0)
    }

    pub fn pan_abs(&self) -> Result<i32, Error> {
        self.get(KernelStreaming::KSPROPERTY_CAMERACONTROL_PAN.0)
    }

    pub fn pan_abs_set(&self, value: i32) -> Result<(), Error> {
        self.set(KernelStreaming::KSPROPERTY_CAMERACONTROL_PAN.0, value)
    }

    pub fn pan_rel_caps(&self) -> Result<Caps, Error> {
        self.caps(KernelStreaming::KSPROPERTY_CAMERACONTROL_PAN_RELATIVE.0)
    }

    pub fn pan_rel(&self) -> Result<i32, Error> {
        self.get(KernelStreaming::KSPROPERTY_CAMERACONTROL_PAN_RELATIVE.0)
    }

    pub fn pan_rel_set(&self, value: i32) -> Result<(), Error> {
        self.set(
            KernelStreaming::KSPROPERTY_CAMERACONTROL_PAN_RELATIVE.0,
            value,
        )
    }

    pub fn tilt_abs_caps(&self) -> Result<Caps, Error> {
        self.caps(KernelStreaming::KSPROPERTY_CAMERACONTROL_TILT.0)
    }

    pub fn tilt_abs(&self) -> Result<i32, Error> {
        self.get(KernelStreaming::KSPROPERTY_CAMERACONTROL_TILT.0)
    }

    pub fn tilt_abs_set(&self, value: i32) -> Result<(), Error> {
        self.set(KernelStreaming::KSPROPERTY_CAMERACONTROL_TILT.0, value)
    }

    pub fn tilt_rel_caps(&self) -> Result<Caps, Error> {
        self.caps(KernelStreaming::KSPROPERTY_CAMERACONTROL_TILT_RELATIVE.0)
    }

    pub fn tilt_rel(&self) -> Result<i32, Error> {
        self.get(KernelStreaming::KSPROPERTY_CAMERACONTROL_TILT_RELATIVE.0)
    }

    pub fn tilt_rel_set(&self, value: i32) -> Result<(), Error> {
        self.set(
            KernelStreaming::KSPROPERTY_CAMERACONTROL_TILT_RELATIVE.0,
            value,
        )
    }
}
