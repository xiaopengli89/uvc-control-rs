use std::{
    ffi::{c_char, c_void},
    mem,
    ops::{Deref, DerefMut},
    ptr, slice,
};

macro_rules! opaque_type {
    ($ffi: ident => $rust: ty) => {
        impl Drop for $ffi {
            fn drop(&mut self) {
                unsafe {
                    let _ = Box::from_raw(self.0 as *mut $rust);
                }
            }
        }

        impl Deref for $ffi {
            type Target = $rust;

            fn deref(&self) -> &Self::Target {
                unsafe { &*(self.0 as *mut $rust) }
            }
        }

        impl DerefMut for $ffi {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { &mut *(self.0 as *mut $rust) }
            }
        }

        impl From<$rust> for $ffi {
            fn from(v: $rust) -> Self {
                $ffi(Box::into_raw(Box::new(v)) as _)
            }
        }

        impl From<$ffi> for $rust {
            fn from(v: $ffi) -> Self {
                unsafe {
                    let r = *Box::from_raw(v.0 as *mut $rust);
                    mem::forget(v);
                    r
                }
            }
        }
    };
}

type ErrorCode = i32;

pub const ERROR_CODE_SUCCESS: ErrorCode = 0;
pub const ERROR_CODE_UNKNOWN: ErrorCode = -1;

#[repr(transparent)]
pub struct DeviceInfoList(*mut c_void);
opaque_type!(DeviceInfoList => Vec<crate::DeviceInfo>);

#[repr(transparent)]
pub struct Device(*mut c_void);
opaque_type!(Device => crate::Device);

#[no_mangle]
pub unsafe extern "C" fn uvc_control_enumerate(p_list: *mut DeviceInfoList) -> ErrorCode {
    let Ok(list) = crate::DeviceInfo::enumerate() else {
        return ERROR_CODE_UNKNOWN;
    };
    ptr::write(p_list, list.into());
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub extern "C" fn uvc_control_info_list_drop(list: DeviceInfoList) {
    let _ = list;
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_info_list_len(list: &DeviceInfoList) -> usize {
    list.len()
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_info_product_id(list: &DeviceInfoList, index: usize) -> u16 {
    list[index].product_id()
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_info_vendor_id(list: &DeviceInfoList, index: usize) -> u16 {
    list[index].vendor_id()
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_info_open(
    list: &DeviceInfoList,
    index: usize,
    p_device: *mut Device,
) -> ErrorCode {
    let Ok(device) = list[index].open() else {
        return ERROR_CODE_UNKNOWN;
    };
    ptr::write(p_device, device.into());
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub extern "C" fn uvc_control_device_drop(device: Device) {
    let _ = device;
}

// Zoom Abs
#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_zoom_abs_caps(
    device: &Device,
    min: &mut i32,
    max: &mut i32,
    res: &mut i32,
    def: &mut i32,
) -> ErrorCode {
    let Ok(caps) = device.zoom_abs_caps() else {
        return ERROR_CODE_UNKNOWN;
    };
    *min = caps.min;
    *max = caps.max;
    *res = caps.res;
    *def = caps.def;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_zoom_abs(device: &Device, cur: &mut i32) -> ErrorCode {
    let Ok(cur_r) = device.zoom_abs() else {
        return ERROR_CODE_UNKNOWN;
    };
    *cur = cur_r;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_zoom_abs_set(device: &Device, value: i32) -> ErrorCode {
    let Ok(_) = device.zoom_abs_set(value) else {
        return ERROR_CODE_UNKNOWN;
    };
    ERROR_CODE_SUCCESS
}

// Zoom Rel
#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_zoom_rel_caps(
    device: &Device,
    min: &mut i32,
    max: &mut i32,
    res: &mut i32,
    def: &mut i32,
) -> ErrorCode {
    let Ok(caps) = device.zoom_rel_caps() else {
        return ERROR_CODE_UNKNOWN;
    };
    *min = caps.min;
    *max = caps.max;
    *res = caps.res;
    *def = caps.def;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_zoom_rel(device: &Device, cur: &mut i32) -> ErrorCode {
    let Ok(cur_r) = device.zoom_rel() else {
        return ERROR_CODE_UNKNOWN;
    };
    *cur = cur_r;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_zoom_rel_set(device: &Device, value: i32) -> ErrorCode {
    let Ok(_) = device.zoom_rel_set(value) else {
        return ERROR_CODE_UNKNOWN;
    };
    ERROR_CODE_SUCCESS
}

// Pan Abs
#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_pan_abs_caps(
    device: &Device,
    min: &mut i32,
    max: &mut i32,
    res: &mut i32,
    def: &mut i32,
) -> ErrorCode {
    let Ok(caps) = device.pan_abs_caps() else {
        return ERROR_CODE_UNKNOWN;
    };
    *min = caps.min;
    *max = caps.max;
    *res = caps.res;
    *def = caps.def;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_pan_abs(device: &Device, cur: &mut i32) -> ErrorCode {
    let Ok(cur_r) = device.pan_abs() else {
        return ERROR_CODE_UNKNOWN;
    };
    *cur = cur_r;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_pan_abs_set(device: &Device, value: i32) -> ErrorCode {
    let Ok(_) = device.pan_abs_set(value) else {
        return ERROR_CODE_UNKNOWN;
    };
    ERROR_CODE_SUCCESS
}

// Pan Rel
#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_pan_rel_caps(
    device: &Device,
    min: &mut i32,
    max: &mut i32,
    res: &mut i32,
    def: &mut i32,
) -> ErrorCode {
    let Ok(caps) = device.pan_rel_caps() else {
        return ERROR_CODE_UNKNOWN;
    };
    *min = caps.min;
    *max = caps.max;
    *res = caps.res;
    *def = caps.def;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_pan_rel(device: &Device, cur: &mut i32) -> ErrorCode {
    let Ok(cur_r) = device.pan_rel() else {
        return ERROR_CODE_UNKNOWN;
    };
    *cur = cur_r;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_pan_rel_set(device: &Device, value: i32) -> ErrorCode {
    let Ok(_) = device.pan_rel_set(value) else {
        return ERROR_CODE_UNKNOWN;
    };
    ERROR_CODE_SUCCESS
}

// Tilt Abs
#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_tilt_abs_caps(
    device: &Device,
    min: &mut i32,
    max: &mut i32,
    res: &mut i32,
    def: &mut i32,
) -> ErrorCode {
    let Ok(caps) = device.tilt_abs_caps() else {
        return ERROR_CODE_UNKNOWN;
    };
    *min = caps.min;
    *max = caps.max;
    *res = caps.res;
    *def = caps.def;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_tilt_abs(device: &Device, cur: &mut i32) -> ErrorCode {
    let Ok(cur_r) = device.tilt_abs() else {
        return ERROR_CODE_UNKNOWN;
    };
    *cur = cur_r;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_tilt_abs_set(device: &Device, value: i32) -> ErrorCode {
    let Ok(_) = device.tilt_abs_set(value) else {
        return ERROR_CODE_UNKNOWN;
    };
    ERROR_CODE_SUCCESS
}

// Tilt Rel
#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_tilt_rel_caps(
    device: &Device,
    min: &mut i32,
    max: &mut i32,
    res: &mut i32,
    def: &mut i32,
) -> ErrorCode {
    let Ok(caps) = device.tilt_rel_caps() else {
        return ERROR_CODE_UNKNOWN;
    };
    *min = caps.min;
    *max = caps.max;
    *res = caps.res;
    *def = caps.def;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_tilt_rel(device: &Device, cur: &mut i32) -> ErrorCode {
    let Ok(cur_r) = device.tilt_rel() else {
        return ERROR_CODE_UNKNOWN;
    };
    *cur = cur_r;
    ERROR_CODE_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_tilt_rel_set(device: &Device, value: i32) -> ErrorCode {
    let Ok(_) = device.tilt_rel_set(value) else {
        return ERROR_CODE_UNKNOWN;
    };
    ERROR_CODE_SUCCESS
}

#[allow(unused_mut, unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_unix_set(
    device: &Device,
    control_code: u8,
    unit: u8,
    data_ptr: *const u8,
    data_len: usize,
) -> ErrorCode {
    let mut r = ERROR_CODE_UNKNOWN;

    #[cfg(unix)]
    {
        let data = slice::from_raw_parts(data_ptr, data_len);
        if device.set(control_code, unit, data).is_ok() {
            r = ERROR_CODE_SUCCESS;
        }
    }

    r
}

#[allow(unused_mut, unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_win_set(
    device: &Device,
    control_code: i32,
    value: i32,
) -> ErrorCode {
    let mut r = ERROR_CODE_UNKNOWN;

    #[cfg(windows)]
    if device.set(control_code, value).is_ok() {
        r = ERROR_CODE_SUCCESS;
    }

    r
}

#[allow(unused_mut, unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn uvc_control_device_win_set_xu(
    device: &Device,
    set: *const c_char,
    id: u32,
    data_ptr: *mut u8,
    data_len: usize,
) -> ErrorCode {
    let mut r = ERROR_CODE_UNKNOWN;

    #[cfg(windows)]
    if let Ok(set) = std::ffi::CStr::from_ptr(set).to_str() {
        let data = slice::from_raw_parts_mut(data_ptr, data_len);
        if device.set_xu(set, id, data).is_ok() {
            r = ERROR_CODE_SUCCESS;
        }
    }

    r
}
