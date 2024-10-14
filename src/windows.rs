use crate::Error;
use std::{mem, ptr};
use windows::core::{Interface, HSTRING, PWSTR};
use windows::Media::{Capture, Devices};
use windows::Win32::Media::{KernelStreaming, MediaFoundation};
use windows::Win32::System::Com;

pub struct DeviceInfo {
    inner: MediaFoundation::IMFActivate,
    id: String,
    product_id: u16,
    vendor_id: u16,
}

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

            let mut symbolic_link = PWSTR::null();
            let mut len = 0;
            if unsafe {
                inner.GetAllocatedString(
                    &MediaFoundation::MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                    &mut symbolic_link,
                    &mut len,
                )
            }
            .is_err()
            {
                continue;
            }

            let id = unsafe { symbolic_link.to_string() };

            unsafe { Com::CoTaskMemFree(Some(symbolic_link.as_ptr() as _)) };

            let Ok(id) = id else {
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

            device_infos.push(Self {
                inner,
                id,
                product_id,
                vendor_id,
            });
        }

        unsafe { Com::CoTaskMemFree(Some(list as _)) };

        Ok(device_infos)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn product_id(&self) -> u16 {
        self.product_id
    }

    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    pub fn open(&self) -> Result<Device, Error> {
        let source: MediaFoundation::IMFMediaSource = unsafe { self.inner.ActivateObject() }?;
        let topology_info: KernelStreaming::IKsTopologyInfo = source.cast()?;
        let num_nodes = unsafe { topology_info.NumNodes() }?;
        let ks_control: KernelStreaming::IKsControl = source.cast()?;

        let mc = Capture::MediaCapture::new()?;
        let settings = Capture::MediaCaptureInitializationSettings::new()?;
        settings.SetVideoDeviceId(&HSTRING::from(&self.id))?;
        mc.InitializeWithSettingsAsync(&settings)?.get()?;

        let controller = mc.VideoDeviceController()?;

        Ok(Device {
            num_nodes,
            ks_control,
            zoom: controller.Zoom()?,
            pan: controller.Pan()?,
            tilt: controller.Tilt()?,
            _controller: controller,
            _mc: mc,
        })
    }
}

pub struct Device {
    num_nodes: u32,
    ks_control: KernelStreaming::IKsControl,
    zoom: Devices::MediaDeviceControl,
    pan: Devices::MediaDeviceControl,
    tilt: Devices::MediaDeviceControl,
    _controller: Devices::VideoDeviceController,
    _mc: Capture::MediaCapture,
}

impl Device {
    pub fn zoom_caps(&self) -> Result<[f64; 3], Error> {
        let cap = self.zoom.Capabilities()?;
        Ok([cap.Step()?, cap.Min()?, cap.Max()?])
    }

    pub fn zoom_get(&self) -> Result<f64, Error> {
        let mut v = 0.;
        self.zoom.TryGetValue(&mut v)?;
        Ok(v)
    }

    pub fn zoom_set(&self, v: f64) -> Result<(), Error> {
        self.zoom.TrySetValue(v)?;
        Ok(())
    }

    pub fn pan_caps(&self) -> Result<[f64; 3], Error> {
        let cap = self.pan.Capabilities()?;
        Ok([cap.Step()?, cap.Min()?, cap.Max()?])
    }

    pub fn pan_get(&self) -> Result<f64, Error> {
        let mut v = 0.;
        self.pan.TryGetValue(&mut v)?;
        Ok(v)
    }

    pub fn pan_set(&self, v: f64) -> Result<(), Error> {
        self.pan.TrySetValue(v)?;
        Ok(())
    }

    pub fn tilt_caps(&self) -> Result<[f64; 3], Error> {
        let cap = self.tilt.Capabilities()?;
        Ok([cap.Step()?, cap.Min()?, cap.Max()?])
    }

    pub fn tilt_get(&self) -> Result<f64, Error> {
        let mut v = 0.;
        self.tilt.TryGetValue(&mut v)?;
        Ok(v)
    }

    pub fn tilt_set(&self, v: f64) -> Result<(), Error> {
        self.tilt.TrySetValue(v)?;
        Ok(())
    }

    // {a8bd5df2-1a98-474e-8dd0-d92672d194fa}, 2, [2]
    pub fn set_auto_focus(&self, set: &str, id: u32, data: &[u8]) -> Result<(), Error> {
        for node_id in 0..self.num_nodes {
            let mut property = KernelStreaming::KSP_NODE::default();
            property.Property.Anonymous.Anonymous.Set =
                unsafe { Com::CLSIDFromString(&HSTRING::from(set)) }?;
            property.Property.Anonymous.Anonymous.Id = id;
            property.Property.Anonymous.Anonymous.Flags =
                KernelStreaming::KSPROPERTY_TYPE_SET | KernelStreaming::KSPROPERTY_TYPE_TOPOLOGY;
            property.NodeId = node_id;

            let mut r = 0;
            if unsafe {
                self.ks_control.KsProperty(
                    &property.Property,
                    mem::size_of_val(&property) as _,
                    data.as_ptr() as *const _ as _,
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
}
