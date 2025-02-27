// jkcoxson
// Experimental system for wrapping a C callback in safe Rust

use std::ffi::CStr;
use std::os::raw::c_void;

use crate::bindings as unsafe_bindings;
use crate::idevice::IDeviceEvent;
use std::any::Any;

pub struct IDeviceEventCallback {
    pub(crate) _function_pointer: Box<dyn FnMut(IDeviceEvent, &dyn Any)>,
    pub(crate) _data: Box<dyn Any>,
    pub(crate) _udid_filter: Option<String>,
}

impl IDeviceEventCallback {
    pub fn new(
        function: Box<dyn FnMut(IDeviceEvent, &dyn Any)>,
        _data: Box<dyn Any>,
        _udid_filter: Option<String>,
    ) -> Self {
        IDeviceEventCallback {
            _function_pointer: function,
            _data,
            _udid_filter,
        }
    }

    pub fn call(&mut self, event: IDeviceEvent) {
        (self._function_pointer)(event, self._data.as_ref());
    }
}

pub unsafe extern "C" fn idevice_event_callback(
    event: *const unsafe_bindings::idevice_event_t,
    user_data: *mut c_void,
) {
    let event: IDeviceEvent = (*event).into();

    let callback = &mut *(user_data as *mut IDeviceEventCallback);

    if let Some(ref filter_udid) = callback._udid_filter {
        let event_udid = event.udid();

        if event_udid != *filter_udid {
            return;
        }
    }

    callback.call(event);
}
