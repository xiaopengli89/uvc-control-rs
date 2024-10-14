use crate::{Caps, Error, RelOperation};
use std::{mem, ptr};
use windows::core::{Interface, HSTRING, PWSTR};
use windows::Media::{Capture, Devices};
use windows::Win32::Media::{DirectShow, KernelStreaming, MediaFoundation};
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
        let am_control: DirectShow::IAMCameraControl = source.cast()?;

        let mc = Capture::MediaCapture::new()?;
        let settings = Capture::MediaCaptureInitializationSettings::new()?;
        settings.SetVideoDeviceId(&HSTRING::from(&self.id))?;
        mc.InitializeWithSettingsAsync(&settings)?.get()?;

        let controller = mc.VideoDeviceController()?;

        Ok(Device {
            num_nodes,
            ks_control,
            am_control,
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
    am_control: DirectShow::IAMCameraControl,
    zoom: Devices::MediaDeviceControl,
    pan: Devices::MediaDeviceControl,
    tilt: Devices::MediaDeviceControl,
    _controller: Devices::VideoDeviceController,
    _mc: Capture::MediaCapture,
}

impl Device {
    fn caps(
        &self,
        control: KernelStreaming::KSPROPERTY_VIDCAP_CAMERACONTROL,
    ) -> Result<Caps, Error> {
        let mut min = 0;
        let mut max = 0;
        let mut step = 0;
        let mut def = 0;
        let mut flags = 0;
        unsafe {
            self.am_control.GetRange(
                control.0, &mut min, &mut max, &mut step, &mut def, &mut flags,
            )
        }?;
        let mut cur = 0;
        unsafe { self.am_control.Get(control.0, &mut cur, &mut flags) }?;
        Ok(Caps {
            min,
            max,
            step,
            def,
            cur,
        })
    }

    pub fn zoom_caps(&self) -> Result<Caps, Error> {
        self.caps(KernelStreaming::KSPROPERTY_CAMERACONTROL_ZOOM)
    }

    pub fn pan_caps(&self) -> Result<Caps, Error> {
        self.caps(KernelStreaming::KSPROPERTY_CAMERACONTROL_PAN)
    }

    pub fn tilt_caps(&self) -> Result<Caps, Error> {
        self.caps(KernelStreaming::KSPROPERTY_CAMERACONTROL_TILT)
    }

    pub fn zoom_abs(&self, value: i32) -> Result<(), Error> {
        Ok(unsafe {
            self.am_control.Set(
                KernelStreaming::KSPROPERTY_CAMERACONTROL_ZOOM.0,
                value,
                DirectShow::CameraControl_Flags_Manual.0,
            )
        }?)
    }

    pub fn zoom_rel(&self, operation: RelOperation) -> Result<(), Error> {
        Ok(unsafe {
            self.am_control.Set(
                KernelStreaming::KSPROPERTY_CAMERACONTROL_ZOOM_RELATIVE.0,
                match operation {
                    RelOperation::Passitive => 1,
                    RelOperation::Negative => -1,
                    RelOperation::Stop => 0,
                },
                DirectShow::CameraControl_Flags_Manual.0,
            )
        }?)
    }

    pub fn pan_absolute(&self, value: i32) -> Result<(), Error> {
        Ok(unsafe {
            self.am_control.Set(
                KernelStreaming::KSPROPERTY_CAMERACONTROL_PAN.0,
                value,
                DirectShow::CameraControl_Flags_Manual.0,
            )
        }?)
    }

    pub fn pan_relative(&self, operation: RelOperation) -> Result<(), Error> {
        Ok(unsafe {
            self.am_control.Set(
                KernelStreaming::KSPROPERTY_CAMERACONTROL_PAN_RELATIVE.0,
                match operation {
                    RelOperation::Passitive => 1,
                    RelOperation::Negative => -1,
                    RelOperation::Stop => 0,
                },
                DirectShow::CameraControl_Flags_Manual.0,
            )
        }?)
    }

    pub fn tilt_absolute(&self, value: i32) -> Result<(), Error> {
        Ok(unsafe {
            self.am_control.Set(
                KernelStreaming::KSPROPERTY_CAMERACONTROL_TILT.0,
                value,
                DirectShow::CameraControl_Flags_Manual.0,
            )
        }?)
    }

    pub fn tilt_relative(&self, operation: RelOperation) -> Result<(), Error> {
        Ok(unsafe {
            self.am_control.Set(
                KernelStreaming::KSPROPERTY_CAMERACONTROL_TILT_RELATIVE.0,
                match operation {
                    RelOperation::Passitive => 1,
                    RelOperation::Negative => -1,
                    RelOperation::Stop => 0,
                },
                DirectShow::CameraControl_Flags_Manual.0,
            )
        }?)
    }

    // {a8bd5df2-1a98-474e-8dd0-d92672d194fa}, 2, [2]
    pub fn set_xu(&self, set: &str, id: u32, data: &[u8]) -> Result<(), Error> {
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
