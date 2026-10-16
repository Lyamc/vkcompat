//! Loader entry points. The rest of the API is reached through vkGetInstanceProcAddr.

use crate::entry::vk_get_instance_proc_addr;
use crate::vk::{PfnVoidFunction, VkInstance, VkResult, ERROR_INITIALIZATION_FAILED, SUCCESS};
use std::ffi::c_char;

#[no_mangle]
pub unsafe extern "system" fn vk_icdGetInstanceProcAddr(
    instance: VkInstance,
    name: *const c_char,
) -> PfnVoidFunction {
    vk_get_instance_proc_addr(instance, name)
}

#[no_mangle]
pub unsafe extern "system" fn vk_icdGetPhysicalDeviceProcAddr(
    _instance: VkInstance,
    _name: *const c_char,
) -> PfnVoidFunction {
    None
}

/// Loader interface versions below 5 stay as requested. 0 and anything above 5 become 5.
#[no_mangle]
pub unsafe extern "system" fn vk_icdNegotiateLoaderICDInterfaceVersion(
    version: *mut u32,
) -> VkResult {
    if version.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    if *version == 0 || *version > 5 {
        *version = 5;
    }
    SUCCESS
}
