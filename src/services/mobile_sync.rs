// jkcoxson

use std::{
    ffi::{c_uint, CString},
    os::raw::c_char,
};

use crate::{
    bindings as unsafe_bindings, error::MobileSyncError, idevice::Device,
    services::lockdownd::LockdowndService,
};

use plist_plus::{Plist, PlistType};

#[derive(Debug, Clone)]
pub struct MobileSyncClient<'a> {
    pub(crate) pointer: unsafe_bindings::mobilesync_client_t,
    phantom: std::marker::PhantomData<&'a Device>,
}

#[derive(Debug)]
pub struct MobileSyncAnchor {
    c_struct: Box<unsafe_bindings::mobilesync_anchors>,
    device_anchor: CString,
    computer_anchor: CString,
}

impl MobileSyncClient<'_> {
    /// Creates a new mobile sync service from a lockdown service
    /// # Arguments
    /// * `device` - The device to connect to
    /// * `descriptor` - The lockdown service to connect on
    /// # Returns
    /// A struct containing the handle to the connection
    ///
    /// ***Verified:*** False
    pub fn new(device: Device, descriptor: LockdowndService) -> Result<Self, MobileSyncError> {
        let mut pointer: unsafe_bindings::mobilesync_client_t = std::ptr::null_mut();
        let result = unsafe {
            unsafe_bindings::mobilesync_client_new(device.pointer, descriptor.pointer, &mut pointer)
        }
        .into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(MobileSyncClient {
            pointer,
            phantom: std::marker::PhantomData,
        })
    }

    /// Starts a new connection and adds a mobile sync to it
    /// # Arguments
    /// * `device` - The device to connect to
    /// * `label` - The label for the connection
    /// # Returns
    /// A struct containing the handle to the connection
    ///
    /// ***Verified:*** False
    pub fn start_service(
        device: Device,
        label: impl Into<String>,
    ) -> Result<Self, MobileSyncError> {
        let label_c_string = CString::new(label.into()).unwrap();
        let mut pointer: unsafe_bindings::mobilesync_client_t = std::ptr::null_mut();
        let result = unsafe {
            unsafe_bindings::mobilesync_client_start_service(
                device.pointer,
                &mut pointer,
                label_c_string.as_ptr(),
            )
        }
        .into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(MobileSyncClient {
            pointer,
            phantom: std::marker::PhantomData,
        })
    }

    /// Receives a message from the service.
    /// Blocks until a full plist has been received
    /// # Arguments
    /// *none*
    /// # Returns
    /// A plist containing the message
    ///
    /// ***Verified:*** False
    pub fn receive(&self) -> Result<Plist, MobileSyncError> {
        let mut plist: unsafe_bindings::plist_t = std::ptr::null_mut();
        let result =
            unsafe { unsafe_bindings::mobilesync_receive(self.pointer, &mut plist) }.into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(plist.into())
    }

    /// Sends a message to the service
    /// # Arguments
    /// * `message` - The message to send
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn send(&self, message: Plist) -> Result<(), MobileSyncError> {
        let result =
            unsafe { unsafe_bindings::mobilesync_send(self.pointer, message.get_pointer()) }.into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Starts the syncing of data
    /// # Arguments
    /// * `data_class` - The identifiers to sync
    /// * `anchors` - The sync anchors to base off of
    /// * `computer_data_class_version` - The class version on the host
    /// * `sync_type` - The type of sync to perform
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn start(
        &self,
        data_class: impl Into<String>,
        mut anchors: Vec<MobileSyncAnchor>,
        computer_data_class_version: u64,
        sync_type: MobileSyncType,
    ) -> Result<(), (String, MobileSyncError)> {
        let data_class_c_string = CString::new(data_class.into()).unwrap();

        let mut anchor_ptrs: Vec<*mut unsafe_bindings::mobilesync_anchors> =
            anchors.iter_mut().map(|v| v.as_c_struct_ptr()).collect();
        anchor_ptrs.push(std::ptr::null_mut());

        let mut device_data_class_version = 0;

        let mut error_description = std::ptr::null_mut();

        let result = unsafe {
            unsafe_bindings::mobilesync_start(
                self.pointer,
                data_class_c_string.as_ptr(),
                anchor_ptrs[0],
                computer_data_class_version,
                &mut sync_type.into(),
                &mut device_data_class_version,
                &mut error_description,
            )
        }
        .into();

        if result != MobileSyncError::Success {
            return Err((
                unsafe { std::ffi::CStr::from_ptr(error_description) }
                    .to_string_lossy()
                    .into_owned(),
                result,
            ));
        }

        Ok(())
    }

    /// Cancels a sync request
    /// # Arguments
    /// * `reason` - The reason for cancelling the sync
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn cancel(&self, reason: impl Into<String>) -> Result<(), MobileSyncError> {
        let reason_c_string = CString::new(reason.into()).unwrap();

        let result =
            unsafe { unsafe_bindings::mobilesync_cancel(self.pointer, reason_c_string.as_ptr()) }
                .into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Finishes the current sync
    /// # Arguments
    /// *none*
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn finish(&self) -> Result<(), MobileSyncError> {
        let result = unsafe { unsafe_bindings::mobilesync_finish(self.pointer) }.into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Gets all sync records from the device
    /// # Arguments
    /// *none*
    /// # Returns
    /// The data, whether it's the end of the data and the anchors
    ///
    /// ***Verified:*** False
    pub fn get_all_records_from_device(&self) -> Result<(Plist, bool, Plist), MobileSyncError> {
        let result =
            unsafe { unsafe_bindings::mobilesync_get_all_records_from_device(self.pointer) }.into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        self.receive_changes()
    }

    /// Gets all the changes from the device
    /// # Arguments
    /// *none*
    /// # Returns
    /// The data, whether it's the end of the data and the anchors
    ///
    /// ***Verified:*** False
    pub fn get_changes_from_device(&self) -> Result<(Plist, bool, Plist), MobileSyncError> {
        let result =
            unsafe { unsafe_bindings::mobilesync_get_changes_from_device(self.pointer) }.into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        self.receive_changes()
    }

    /// Clears the records on the device
    /// # Arguments
    /// *none*
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn clear_all_records_on_device(&self) -> Result<(), MobileSyncError> {
        let result =
            unsafe { unsafe_bindings::mobilesync_clear_all_records_on_device(self.pointer) }.into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Receive changes from the device
    /// # Arguments
    /// *none* Returns
    /// The data, whether it's the end of the data and the anchors
    ///
    /// ***Verified:*** False
    pub fn receive_changes(&self) -> Result<(Plist, bool, Plist), MobileSyncError> {
        let mut plist: unsafe_bindings::plist_t = std::ptr::null_mut();
        let mut has_more_changes = 0;
        let mut anchor: unsafe_bindings::plist_t = std::ptr::null_mut();

        let result = unsafe {
            unsafe_bindings::mobilesync_receive_changes(
                self.pointer,
                &mut plist,
                &mut has_more_changes,
                &mut anchor,
            )
        }
        .into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok((plist.into(), has_more_changes != 0, anchor.into()))
    }

    /// Acknoledge the changes from the device to continue sync
    /// # Arguments
    /// *none*
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn acknowledge_changes_from_device(&self) -> Result<(), MobileSyncError> {
        let result =
            unsafe { unsafe_bindings::mobilesync_acknowledge_changes_from_device(self.pointer) }
                .into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Tells the client that the host is ready to send changes
    /// # Arguments
    /// *none*
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn ready_to_send_changes_from_computer(&self) -> Result<(), MobileSyncError> {
        let result = unsafe {
            unsafe_bindings::mobilesync_ready_to_send_changes_from_computer(self.pointer)
        }
        .into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Send changes to the device
    /// # Arguments
    /// * `entities` - The changes to send in a plist
    /// * `is_lanst` - Tells the device if it's the last change
    /// * `actions` - Additional actions the device should perform
    ///
    /// ***Verified:*** False
    pub fn send_changes(
        &self,
        entities: Plist,
        is_last: bool,
        actions: Option<Plist>,
    ) -> Result<(), MobileSyncError> {
        let actions = actions
            .as_ref()
            .map_or(std::ptr::null_mut(), |v| v.get_pointer());

        let result = unsafe {
            unsafe_bindings::mobilesync_send_changes(
                self.pointer,
                entities.get_pointer(),
                is_last.into(),
                actions,
            )
        }
        .into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Remaps the identifiers on the device
    /// # Arguments
    /// * `mapping` - The new mappings the device should use
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn remap_identifiers(&self, mapping: Plist) -> Result<(), MobileSyncError> {
        if mapping.plist_type != PlistType::Array {
            return Err(MobileSyncError::InvalidArg);
        }

        let result = unsafe {
            unsafe_bindings::mobilesync_remap_identifiers(self.pointer, &mut mapping.get_pointer())
        }
        .into();

        if result != MobileSyncError::Success {
            return Err(result);
        }

        Ok(())
    }
}

impl MobileSyncAnchor {
    pub fn new(device_anchor: impl Into<String>, computer_anchor: impl Into<String>) -> Self {
        let device_anchor_c_string = CString::new(device_anchor.into()).unwrap();
        let computer_anchor_c_string = CString::new(computer_anchor.into()).unwrap();
        let c_struct = unsafe_bindings::mobilesync_anchors {
            device_anchor: device_anchor_c_string.as_ptr() as *mut c_char,
            computer_anchor: computer_anchor_c_string.as_ptr() as *mut c_char,
        };
        MobileSyncAnchor {
            c_struct: Box::new(c_struct),
            device_anchor: device_anchor_c_string,
            computer_anchor: computer_anchor_c_string,
        }
    }

    pub(crate) fn as_c_struct_ptr(&mut self) -> *mut unsafe_bindings::mobilesync_anchors {
        self.c_struct.as_mut()
    }

    pub fn device_anchor(&self) -> &str {
        self.device_anchor.as_c_str().to_str().unwrap()
    }

    pub fn computer_anchor(&self) -> &str {
        self.computer_anchor.as_c_str().to_str().unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileSyncType {
    Fast,
    Slow,
    Reset,
}

impl From<MobileSyncType> for c_uint {
    fn from(type_: MobileSyncType) -> Self {
        match type_ {
            MobileSyncType::Fast => 0,
            MobileSyncType::Slow => 1,
            MobileSyncType::Reset => 2,
        }
    }
}

impl Drop for MobileSyncClient<'_> {
    fn drop(&mut self) {
        unsafe {
            unsafe_bindings::mobilesync_client_free(self.pointer);
        }
    }
}
