#pragma once

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

namespace uvc_control {

using ErrorCode = int32_t;

using DeviceInfoList = void*;

using Device = void*;

constexpr static const ErrorCode ERROR_CODE_SUCCESS = 0;

constexpr static const ErrorCode ERROR_CODE_UNKNOWN = -1;

extern "C" {

ErrorCode uvc_control_enumerate(DeviceInfoList *p_list);

void uvc_control_info_list_drop(DeviceInfoList list);

uintptr_t uvc_control_info_list_len(const DeviceInfoList *list);

uint16_t uvc_control_info_product_id(const DeviceInfoList *list, uintptr_t index);

uint16_t uvc_control_info_vendor_id(const DeviceInfoList *list, uintptr_t index);

ErrorCode uvc_control_info_open(const DeviceInfoList *list, uintptr_t index, Device *p_device);

void uvc_control_device_drop(Device device);

ErrorCode uvc_control_device_zoom_abs_caps(const Device *device,
                                           int32_t *min,
                                           int32_t *max,
                                           int32_t *res,
                                           int32_t *def);

ErrorCode uvc_control_device_zoom_abs(const Device *device, int32_t *cur);

ErrorCode uvc_control_device_zoom_abs_set(const Device *device, int32_t value);

ErrorCode uvc_control_device_zoom_rel_caps(const Device *device,
                                           int32_t *min,
                                           int32_t *max,
                                           int32_t *res,
                                           int32_t *def);

ErrorCode uvc_control_device_zoom_rel(const Device *device, int32_t *cur);

ErrorCode uvc_control_device_zoom_rel_set(const Device *device, int32_t value);

ErrorCode uvc_control_device_pan_abs_caps(const Device *device,
                                          int32_t *min,
                                          int32_t *max,
                                          int32_t *res,
                                          int32_t *def);

ErrorCode uvc_control_device_pan_abs(const Device *device, int32_t *cur);

ErrorCode uvc_control_device_pan_abs_set(const Device *device, int32_t value);

ErrorCode uvc_control_device_pan_rel_caps(const Device *device,
                                          int32_t *min,
                                          int32_t *max,
                                          int32_t *res,
                                          int32_t *def);

ErrorCode uvc_control_device_pan_rel(const Device *device, int32_t *cur);

ErrorCode uvc_control_device_pan_rel_set(const Device *device, int32_t value);

ErrorCode uvc_control_device_tilt_abs_caps(const Device *device,
                                           int32_t *min,
                                           int32_t *max,
                                           int32_t *res,
                                           int32_t *def);

ErrorCode uvc_control_device_tilt_abs(const Device *device, int32_t *cur);

ErrorCode uvc_control_device_tilt_abs_set(const Device *device, int32_t value);

ErrorCode uvc_control_device_tilt_rel_caps(const Device *device,
                                           int32_t *min,
                                           int32_t *max,
                                           int32_t *res,
                                           int32_t *def);

ErrorCode uvc_control_device_tilt_rel(const Device *device, int32_t *cur);

ErrorCode uvc_control_device_tilt_rel_set(const Device *device, int32_t value);

ErrorCode uvc_control_device_unix_set(const Device *device,
                                      uint8_t control_code,
                                      uint8_t unit,
                                      const uint8_t *data_ptr,
                                      uintptr_t data_len);

ErrorCode uvc_control_device_win_set(const Device *device, int32_t control_code, int32_t value);

ErrorCode uvc_control_device_win_set_xu(const Device *device,
                                        const char *set,
                                        uint32_t id,
                                        uint8_t *data_ptr,
                                        uintptr_t data_len);

} // extern "C"

} // namespace uvc_control
