//! Vulkan 1.0 entry points. The DLL itself exports only the three vk_icd* functions.

use crate::driver;
use crate::vk::*;
use std::ffi::{c_char, c_void};

pub use crate::driver::{gfx_pipeline_note, gfx_pipeline_uses_hw};

fn erase(p: *const ()) -> PfnVoidFunction {
    unsafe { std::mem::transmute(p) }
}

pub unsafe fn vk_get_instance_proc_addr(instance: VkInstance, name: *const c_char) -> PfnVoidFunction {
    if name.is_null() {
        return None;
    }
    let n = cstr(name);
    if instance.is_null() {
        return match n.as_str() {
            "vkCreateInstance" => erase(vkCreateInstance as *const ()),
            "vkEnumerateInstanceExtensionProperties" => erase(vkEnumerateInstanceExtensionProperties as *const ()),
            "vkEnumerateInstanceLayerProperties" => erase(vkEnumerateInstanceLayerProperties as *const ()),
            "vkEnumerateInstanceVersion" => erase(vkEnumerateInstanceVersion as *const ()),
            _ => None,
        };
    }
    lookup(&n)
}

pub unsafe fn vk_get_device_proc_addr(device: VkDevice, name: *const c_char) -> PfnVoidFunction {
    if device.is_null() || name.is_null() {
        return None;
    }
    let n = cstr(name);
    match n.as_str() {
        "vkCreateInstance" | "vkEnumerateInstanceExtensionProperties" | "vkEnumerateInstanceLayerProperties"
        | "vkEnumerateInstanceVersion" | "vkDestroyInstance" | "vkEnumeratePhysicalDevices" => None,
        _ => lookup(&n),
    }
}

fn lookup(n: &str) -> PfnVoidFunction {
    match n {
        "vkGetInstanceProcAddr" => erase(vkGetInstanceProcAddr as *const ()),
        "vkGetDeviceProcAddr" => erase(vkGetDeviceProcAddr as *const ()),
        "vkCreateInstance" => erase(vkCreateInstance as *const ()),
        "vkDestroyInstance" => erase(vkDestroyInstance as *const ()),
        "vkEnumerateInstanceVersion" => erase(vkEnumerateInstanceVersion as *const ()),
        "vkEnumerateInstanceExtensionProperties" => erase(vkEnumerateInstanceExtensionProperties as *const ()),
        "vkEnumerateInstanceLayerProperties" => erase(vkEnumerateInstanceLayerProperties as *const ()),
        "vkEnumeratePhysicalDevices" => erase(vkEnumeratePhysicalDevices as *const ()),
        "vkGetPhysicalDeviceProperties" => erase(vkGetPhysicalDeviceProperties as *const ()),
        "vkGetPhysicalDeviceFeatures" => erase(vkGetPhysicalDeviceFeatures as *const ()),
        "vkGetPhysicalDeviceMemoryProperties" => erase(vkGetPhysicalDeviceMemoryProperties as *const ()),
        "vkGetPhysicalDeviceQueueFamilyProperties" => erase(vkGetPhysicalDeviceQueueFamilyProperties as *const ()),
        "vkGetPhysicalDeviceFormatProperties" => erase(vkGetPhysicalDeviceFormatProperties as *const ()),
        "vkGetPhysicalDeviceImageFormatProperties" => erase(vkGetPhysicalDeviceImageFormatProperties as *const ()),
        "vkGetPhysicalDeviceSparseImageFormatProperties" => {
            erase(vkGetPhysicalDeviceSparseImageFormatProperties as *const ())
        }
        "vkGetPhysicalDeviceProperties2" | "vkGetPhysicalDeviceProperties2KHR" => {
            erase(vkGetPhysicalDeviceProperties2 as *const ())
        }
        "vkGetPhysicalDeviceFeatures2" | "vkGetPhysicalDeviceFeatures2KHR" => erase(vkGetPhysicalDeviceFeatures2 as *const ()),
        "vkGetPhysicalDeviceMemoryProperties2" | "vkGetPhysicalDeviceMemoryProperties2KHR" => {
            erase(vkGetPhysicalDeviceMemoryProperties2 as *const ())
        }
        "vkEnumerateDeviceExtensionProperties" => erase(vkEnumerateDeviceExtensionProperties as *const ()),
        "vkCreateDevice" => erase(vkCreateDevice as *const ()),
        "vkDestroyDevice" => erase(vkDestroyDevice as *const ()),
        "vkGetDeviceQueue" => erase(vkGetDeviceQueue as *const ()),
        "vkQueueSubmit" => erase(vkQueueSubmit as *const ()),
        "vkCreateBuffer" => erase(vkCreateBuffer as *const ()),
        "vkDestroyBuffer" => erase(vkDestroyBuffer as *const ()),
        "vkGetBufferMemoryRequirements" => erase(vkGetBufferMemoryRequirements as *const ()),
        "vkBindBufferMemory" => erase(vkBindBufferMemory as *const ()),
        "vkAllocateMemory" => erase(vkAllocateMemory as *const ()),
        "vkFreeMemory" => erase(vkFreeMemory as *const ()),
        "vkMapMemory" => erase(vkMapMemory as *const ()),
        "vkUnmapMemory" => erase(vkUnmapMemory as *const ()),
        "vkCreateImage" => erase(vkCreateImage as *const ()),
        "vkDestroyImage" => erase(vkDestroyImage as *const ()),
        "vkGetImageMemoryRequirements" => erase(vkGetImageMemoryRequirements as *const ()),
        "vkBindImageMemory" => erase(vkBindImageMemory as *const ()),
        "vkCreateImageView" => erase(vkCreateImageView as *const ()),
        "vkDestroyImageView" => erase(vkDestroyImageView as *const ()),
        "vkCreateShaderModule" => erase(vkCreateShaderModule as *const ()),
        "vkDestroyShaderModule" => erase(vkDestroyShaderModule as *const ()),
        "vkCreateGraphicsPipelines" => erase(vkCreateGraphicsPipelines as *const ()),
        "vkCreateComputePipelines" => erase(vkCreateComputePipelines as *const ()),
        "vkDestroyPipeline" => erase(vkDestroyPipeline as *const ()),
        "vkCreatePipelineLayout" => erase(vkCreatePipelineLayout as *const ()),
        "vkDestroyPipelineLayout" => erase(vkDestroyPipelineLayout as *const ()),
        "vkCreateDescriptorSetLayout" => erase(vkCreateDescriptorSetLayout as *const ()),
        "vkDestroyDescriptorSetLayout" => erase(vkDestroyDescriptorSetLayout as *const ()),
        "vkCreateDescriptorPool" => erase(vkCreateDescriptorPool as *const ()),
        "vkDestroyDescriptorPool" => erase(vkDestroyDescriptorPool as *const ()),
        "vkAllocateDescriptorSets" => erase(vkAllocateDescriptorSets as *const ()),
        "vkUpdateDescriptorSets" => erase(vkUpdateDescriptorSets as *const ()),
        "vkCreateRenderPass" => erase(vkCreateRenderPass as *const ()),
        "vkDestroyRenderPass" => erase(vkDestroyRenderPass as *const ()),
        "vkCreateFramebuffer" => erase(vkCreateFramebuffer as *const ()),
        "vkDestroyFramebuffer" => erase(vkDestroyFramebuffer as *const ()),
        "vkCreateCommandPool" => erase(vkCreateCommandPool as *const ()),
        "vkDestroyCommandPool" => erase(vkDestroyCommandPool as *const ()),
        "vkAllocateCommandBuffers" => erase(vkAllocateCommandBuffers as *const ()),
        "vkBeginCommandBuffer" => erase(vkBeginCommandBuffer as *const ()),
        "vkEndCommandBuffer" => erase(vkEndCommandBuffer as *const ()),
        "vkCmdBindPipeline" => erase(vkCmdBindPipeline as *const ()),
        "vkCmdBindVertexBuffers" => erase(vkCmdBindVertexBuffers as *const ()),
        "vkCmdBindDescriptorSets" => erase(vkCmdBindDescriptorSets as *const ()),
        "vkCmdSetViewport" => erase(vkCmdSetViewport as *const ()),
        "vkCmdSetScissor" => erase(vkCmdSetScissor as *const ()),
        "vkCmdBeginRenderPass" => erase(vkCmdBeginRenderPass as *const ()),
        "vkCmdEndRenderPass" => erase(vkCmdEndRenderPass as *const ()),
        "vkCmdDraw" => erase(vkCmdDraw as *const ()),
        "vkCmdDispatch" => erase(vkCmdDispatch as *const ()),
        "vkCmdPipelineBarrier" => erase(vkCmdPipelineBarrier as *const ()),
        "vkCmdCopyBufferToImage" => erase(vkCmdCopyBufferToImage as *const ()),
        "vkCmdCopyImageToBuffer" => erase(vkCmdCopyImageToBuffer as *const ()),
        "vkCreateSampler" => erase(vkCreateSampler as *const ()),
        "vkDestroySampler" => erase(vkDestroySampler as *const ()),
        "vkCreateWin32SurfaceKHR" => erase(vkCreateWin32SurfaceKHR as *const ()),
        "vkDestroySurfaceKHR" => erase(vkDestroySurfaceKHR as *const ()),
        "vkGetPhysicalDeviceSurfaceSupportKHR" => erase(vkGetPhysicalDeviceSurfaceSupportKHR as *const ()),
        "vkGetPhysicalDeviceSurfaceCapabilitiesKHR" => erase(vkGetPhysicalDeviceSurfaceCapabilitiesKHR as *const ()),
        "vkGetPhysicalDeviceSurfaceFormatsKHR" => erase(vkGetPhysicalDeviceSurfaceFormatsKHR as *const ()),
        "vkGetPhysicalDeviceSurfacePresentModesKHR" => erase(vkGetPhysicalDeviceSurfacePresentModesKHR as *const ()),
        "vkGetPhysicalDeviceWin32PresentationSupportKHR" => {
            erase(vkGetPhysicalDeviceWin32PresentationSupportKHR as *const ())
        }
        "vkCreateSwapchainKHR" => erase(vkCreateSwapchainKHR as *const ()),
        "vkDestroySwapchainKHR" => erase(vkDestroySwapchainKHR as *const ()),
        "vkGetSwapchainImagesKHR" => erase(vkGetSwapchainImagesKHR as *const ()),
        "vkCreateSemaphore" => erase(vkCreateSemaphore as *const ()),
        "vkDestroySemaphore" => erase(vkDestroySemaphore as *const ()),
        "vkAcquireNextImageKHR" => erase(vkAcquireNextImageKHR as *const ()),
        "vkQueuePresentKHR" => erase(vkQueuePresentKHR as *const ()),
        _ => None,
    }
}

fn write_count(count: *mut u32, n: u32, dst: *mut u8, stride: usize, src: &[u8]) -> VkResult {
    unsafe {
        if count.is_null() {
            return ERROR_INITIALIZATION_FAILED;
        }
        if dst.is_null() {
            *count = n;
            return SUCCESS;
        }
        let room = *count;
        let write = room.min(n);
        for i in 0..write as usize {
            std::ptr::copy_nonoverlapping(src[i * stride..].as_ptr(), dst.add(i * stride), stride);
        }
        *count = write;
        if room < n { INCOMPLETE } else { SUCCESS }
    }
}

const DEVICE_EXTS: [&str; 2] = [EXT_SWAPCHAIN, EXT_DYNAMIC_RENDERING];

fn ext_prop(name: &str) -> VkExtensionProperties {
    VkExtensionProperties { extensionName: ext_bytes(name), specVersion: 1 }
}

pub unsafe extern "system" fn vkGetInstanceProcAddr(instance: VkInstance, name: *const c_char) -> PfnVoidFunction {
    vk_get_instance_proc_addr(instance, name)
}
pub unsafe extern "system" fn vkGetDeviceProcAddr(device: VkDevice, name: *const c_char) -> PfnVoidFunction {
    vk_get_device_proc_addr(device, name)
}
pub unsafe extern "system" fn vkEnumerateInstanceVersion(p: *mut u32) -> VkResult {
    if p.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *p = API_VERSION_1_0;
    SUCCESS
}
pub unsafe extern "system" fn vkEnumerateInstanceLayerProperties(count: *mut u32, _props: *mut c_void) -> VkResult {
    if count.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *count = 0;
    SUCCESS
}
pub unsafe extern "system" fn vkEnumerateInstanceExtensionProperties(
    layer: *const c_char, count: *mut u32, props: *mut VkExtensionProperties,
) -> VkResult {
    if !layer.is_null() && *layer != 0 {
        eprintln!("gfx: no layers");
        return ERROR_LAYER_NOT_PRESENT;
    }
    let items: Vec<_> = instance_extension_names().iter().map(|n| ext_prop(n)).collect();
    let bytes = std::slice::from_raw_parts(items.as_ptr() as *const u8, items.len() * std::mem::size_of::<VkExtensionProperties>());
    write_count(count, items.len() as u32, props as *mut u8, std::mem::size_of::<VkExtensionProperties>(), bytes)
}
pub unsafe extern "system" fn vkCreateInstance(
    info: *const VkInstanceCreateInfo, _alloc: *const c_void, out: *mut VkInstance,
) -> VkResult {
    driver::create_instance(info, out)
}
pub unsafe extern "system" fn vkDestroyInstance(instance: VkInstance, _alloc: *const c_void) {
    driver::destroy_instance(instance);
}
pub unsafe extern "system" fn vkEnumeratePhysicalDevices(
    instance: VkInstance, count: *mut u32, devices: *mut VkPhysicalDevice,
) -> VkResult {
    driver::enumerate_physical_devices(instance, count, devices)
}
pub unsafe extern "system" fn vkGetPhysicalDeviceProperties(p: VkPhysicalDevice, out: *mut VkPhysicalDeviceProperties) {
    if let Some(d) = driver::phys(p) { driver::fill_props(d, out); }
}
pub unsafe extern "system" fn vkGetPhysicalDeviceFeatures(p: VkPhysicalDevice, out: *mut VkPhysicalDeviceFeatures) {
    if p.is_null() || out.is_null() { return; }
    *out = zeroed();
}
pub unsafe extern "system" fn vkGetPhysicalDeviceMemoryProperties(p: VkPhysicalDevice, out: *mut VkPhysicalDeviceMemoryProperties) {
    if let Some(d) = driver::phys(p) { driver::fill_mem(d, out); }
}
pub unsafe extern "system" fn vkGetPhysicalDeviceQueueFamilyProperties(
    p: VkPhysicalDevice, count: *mut u32, props: *mut VkQueueFamilyProperties,
) {
    if count.is_null() || driver::phys(p).is_none() { return; }
    if props.is_null() { *count = 1; return; }
    if *count == 0 { return; }
    if let Some(d) = driver::phys(p) { *props = driver::queue_props(d); }
    *count = 1;
}
fn format_feature_pair(fmt: i32) -> (u32, u32) {
    let color = FORMAT_FEATURE_SAMPLED_IMAGE_BIT
        | FORMAT_FEATURE_COLOR_ATTACHMENT_BIT
        | FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT
        | FORMAT_FEATURE_BLIT_SRC_BIT
        | FORMAT_FEATURE_BLIT_DST_BIT;
    match fmt {
        FORMAT_R8G8B8A8_UNORM | FORMAT_B8G8R8A8_UNORM => (color, 0),
        FORMAT_R32G32B32A32_SFLOAT => (
            FORMAT_FEATURE_SAMPLED_IMAGE_BIT
                | FORMAT_FEATURE_COLOR_ATTACHMENT_BIT
                | FORMAT_FEATURE_BLIT_SRC_BIT
                | FORMAT_FEATURE_BLIT_DST_BIT,
            FORMAT_FEATURE_VERTEX_BUFFER_BIT,
        ),
        FORMAT_R32_SFLOAT | FORMAT_R32G32_SFLOAT | FORMAT_R32G32B32_SFLOAT => (0, FORMAT_FEATURE_VERTEX_BUFFER_BIT),
        FORMAT_D32_SFLOAT | FORMAT_D24_UNORM_S8_UINT => (FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT, 0),
        _ => (0, 0),
    }
}

pub unsafe extern "system" fn vkGetPhysicalDeviceFormatProperties(
    p: VkPhysicalDevice, format: i32, out: *mut VkFormatProperties,
) {
    if out.is_null() {
        return;
    }
    let (image, buffer) = if driver::phys(p).is_some() { format_feature_pair(format) } else { (0, 0) };
    *out = VkFormatProperties {
        linearTilingFeatures: image,
        optimalTilingFeatures: image,
        bufferFeatures: buffer,
    };
}

pub unsafe extern "system" fn vkGetPhysicalDeviceImageFormatProperties(
    p: VkPhysicalDevice, format: i32, ty: i32, tiling: i32, usage: u32, flags: u32, out: *mut VkImageFormatProperties,
) -> VkResult {
    if out.is_null() || driver::phys(p).is_none() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let (image, _) = format_feature_pair(format);
    let tiling_ok = tiling == IMAGE_TILING_OPTIMAL || tiling == IMAGE_TILING_LINEAR;
    let mut unsupported = !tiling_ok || ty != IMAGE_TYPE_2D || flags != 0 || image == 0;
    let mut left = usage;
    let mut take = |left: &mut u32, bit: u32, have: bool| {
        if *left & bit != 0 {
            if !have {
                unsupported = true;
            }
            *left &= !bit;
        }
    };
    take(&mut left, IMAGE_USAGE_TRANSFER_SRC_BIT, true);
    take(&mut left, IMAGE_USAGE_TRANSFER_DST_BIT, true);
    take(&mut left, IMAGE_USAGE_SAMPLED_BIT, image & FORMAT_FEATURE_SAMPLED_IMAGE_BIT != 0);
    take(&mut left, IMAGE_USAGE_COLOR_ATTACHMENT_BIT, image & FORMAT_FEATURE_COLOR_ATTACHMENT_BIT != 0);
    take(
        &mut left,
        IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
        image & FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT != 0,
    );
    if left != 0 {
        unsupported = true;
    }
    if unsupported {
        *out = zeroed();
        return ERROR_FORMAT_NOT_SUPPORTED;
    }
    *out = VkImageFormatProperties {
        maxExtent: VkExtent3D { width: 8192, height: 8192, depth: 1 },
        maxMipLevels: 1,
        maxArrayLayers: 1,
        sampleCounts: SAMPLE_COUNT_1_BIT,
        maxResourceSize: 8192 * 8192 * 16,
    };
    SUCCESS
}

pub unsafe extern "system" fn vkGetPhysicalDeviceSparseImageFormatProperties(
    p: VkPhysicalDevice, _format: i32, _ty: i32, _samples: u32, _usage: u32, _tiling: i32,
    count: *mut u32, _props: *mut c_void,
) {
    if count.is_null() || driver::phys(p).is_none() {
        return;
    }
    *count = 0;
}

pub unsafe extern "system" fn vkGetPhysicalDeviceProperties2(p: VkPhysicalDevice, out: *mut VkPhysicalDeviceProperties2) {
    if out.is_null() { return; }
    if let Some(d) = driver::phys(p) { (*out).properties = d.props_copy(); }
}
pub unsafe extern "system" fn vkGetPhysicalDeviceFeatures2(p: VkPhysicalDevice, out: *mut VkPhysicalDeviceFeatures2) {
    if p.is_null() || out.is_null() { return; }
    (*out).features = zeroed();
}
pub unsafe extern "system" fn vkGetPhysicalDeviceMemoryProperties2(p: VkPhysicalDevice, out: *mut VkPhysicalDeviceMemoryProperties2) {
    if out.is_null() { return; }
    if let Some(d) = driver::phys(p) { (*out).memoryProperties = d.mem_copy(); }
}
pub unsafe extern "system" fn vkEnumerateDeviceExtensionProperties(
    _p: VkPhysicalDevice, layer: *const c_char, count: *mut u32, props: *mut VkExtensionProperties,
) -> VkResult {
    if !layer.is_null() && *layer != 0 {
        eprintln!("gfx: no layers");
        return ERROR_LAYER_NOT_PRESENT;
    }
    let items: Vec<_> = DEVICE_EXTS.iter().map(|n| ext_prop(n)).collect();
    let bytes = std::slice::from_raw_parts(items.as_ptr() as *const u8, items.len() * std::mem::size_of::<VkExtensionProperties>());
    write_count(count, items.len() as u32, props as *mut u8, std::mem::size_of::<VkExtensionProperties>(), bytes)
}
pub unsafe extern "system" fn vkCreateDevice(
    physical: VkPhysicalDevice, info: *const VkDeviceCreateInfo, _alloc: *const c_void, out: *mut VkDevice,
) -> VkResult {
    if info.is_null() { return ERROR_INITIALIZATION_FAILED; }
    driver::create_device(physical, &*info, out)
}
pub unsafe extern "system" fn vkDestroyDevice(device: VkDevice, _alloc: *const c_void) { driver::destroy_device(device); }
pub unsafe extern "system" fn vkGetDeviceQueue(device: VkDevice, _family: u32, _index: u32, out: *mut VkQueue) {
    if !out.is_null() { *out = driver::get_queue(device); }
}
pub unsafe extern "system" fn vkQueueSubmit(
    queue: VkQueue, count: u32, infos: *const VkSubmitInfo, _fence: VkFence,
) -> VkResult {
    if count == 0 || infos.is_null() { return SUCCESS; }
    let infos = std::slice::from_raw_parts(infos, count as usize);
    for info in infos {
        let r = driver::queue_submit(queue, info);
        if r != SUCCESS { return r; }
    }
    SUCCESS
}
pub unsafe extern "system" fn vkAllocateMemory(
    _device: VkDevice, info: *const VkMemoryAllocateInfo, _alloc: *const c_void, out: *mut VkDeviceMemory,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::alloc_memory((*info).allocationSize) as VkDeviceMemory;
    SUCCESS
}
pub unsafe extern "system" fn vkFreeMemory(_device: VkDevice, mem: VkDeviceMemory, _alloc: *const c_void) {
    driver::free_memory(mem);
}
pub unsafe extern "system" fn vkMapMemory(
    _device: VkDevice, mem: VkDeviceMemory, offset: VkDeviceSize, _size: VkDeviceSize, _flags: VkFlags, out: *mut *mut c_void,
) -> VkResult {
    driver::map_memory(mem, offset, out)
}
pub unsafe extern "system" fn vkUnmapMemory(_device: VkDevice, mem: VkDeviceMemory) { driver::unmap_memory(mem); }
pub unsafe extern "system" fn vkCreateBuffer(
    _device: VkDevice, info: *const VkBufferCreateInfo, _alloc: *const c_void, out: *mut VkBuffer,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_buffer((*info).size) as VkBuffer;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyBuffer(_device: VkDevice, buffer: VkBuffer, _alloc: *const c_void) {
    driver::destroy_buffer(buffer);
}
pub unsafe extern "system" fn vkGetBufferMemoryRequirements(_device: VkDevice, buffer: VkBuffer, out: *mut VkMemoryRequirements) {
    if out.is_null() { return; }
    let size = if buffer.is_null() { 0 } else { (*(buffer as *mut driver::Buffer)).size_of() };
    *out = driver::buffer_requirements(size);
}
pub unsafe extern "system" fn vkBindBufferMemory(
    _device: VkDevice, buffer: VkBuffer, mem: VkDeviceMemory, offset: VkDeviceSize,
) -> VkResult {
    driver::bind_buffer(buffer, mem, offset)
}
pub unsafe extern "system" fn vkCreateImage(
    _device: VkDevice, info: *const VkImageCreateInfo, _alloc: *const c_void, out: *mut VkImage,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    match driver::create_image(&*info) {
        Ok(img) => { *out = img as VkImage; SUCCESS }
        Err(e) => e,
    }
}
pub unsafe extern "system" fn vkDestroyImage(_device: VkDevice, image: VkImage, _alloc: *const c_void) {
    driver::destroy_image(image);
}
pub unsafe extern "system" fn vkGetImageMemoryRequirements(_device: VkDevice, image: VkImage, out: *mut VkMemoryRequirements) {
    if out.is_null() || image.is_null() { return; }
    *out = driver::image_requirements(&*(image as *mut driver::Image));
}
pub unsafe extern "system" fn vkBindImageMemory(
    _device: VkDevice, _image: VkImage, _mem: VkDeviceMemory, _offset: VkDeviceSize,
) -> VkResult {
    SUCCESS
}
pub unsafe extern "system" fn vkCreateImageView(
    _device: VkDevice, info: *const VkImageViewCreateInfo, _alloc: *const c_void, out: *mut VkImageView,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_image_view((*info).image) as VkImageView;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyImageView(_device: VkDevice, view: VkImageView, _alloc: *const c_void) {
    driver::destroy_image_view(view);
}
pub unsafe extern "system" fn vkCreateShaderModule(
    _device: VkDevice, info: *const VkShaderModuleCreateInfo, _alloc: *const c_void, out: *mut VkShaderModule,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_shader((*info).pCode, (*info).codeSize) as VkShaderModule;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyShaderModule(_device: VkDevice, module: VkShaderModule, _alloc: *const c_void) {
    driver::destroy_shader(module);
}
pub unsafe extern "system" fn vkCreateGraphicsPipelines(
    device: VkDevice, _cache: VkPipelineCache, count: u32, infos: *const VkGraphicsPipelineCreateInfo,
    _alloc: *const c_void, out: *mut VkPipeline,
) -> VkResult {
    if out.is_null() || (count > 0 && infos.is_null()) { return ERROR_INITIALIZATION_FAILED; }
    for i in 0..count as usize {
        *out.add(i) = driver::create_graphics(device, &*infos.add(i)) as VkPipeline;
    }
    SUCCESS
}
pub unsafe extern "system" fn vkCreateComputePipelines(
    device: VkDevice, _cache: VkPipelineCache, count: u32, infos: *const VkComputePipelineCreateInfo,
    _alloc: *const c_void, out: *mut VkPipeline,
) -> VkResult {
    if out.is_null() || (count > 0 && infos.is_null()) { return ERROR_INITIALIZATION_FAILED; }
    for i in 0..count as usize {
        *out.add(i) = driver::create_compute(device, &*infos.add(i)) as VkPipeline;
    }
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyPipeline(_device: VkDevice, pipeline: VkPipeline, _alloc: *const c_void) {
    driver::destroy_pipeline(pipeline);
}
pub unsafe extern "system" fn vkCreatePipelineLayout(
    _device: VkDevice, _info: *const VkPipelineLayoutCreateInfo, _alloc: *const c_void, out: *mut VkPipelineLayout,
) -> VkResult {
    if out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_layout() as VkPipelineLayout;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyPipelineLayout(_device: VkDevice, layout: VkPipelineLayout, _alloc: *const c_void) {
    driver::destroy_layout(layout);
}
pub unsafe extern "system" fn vkCreateDescriptorSetLayout(
    _device: VkDevice, _info: *const VkDescriptorSetLayoutCreateInfo, _alloc: *const c_void, out: *mut VkDescriptorSetLayout,
) -> VkResult {
    if out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_set_layout() as VkDescriptorSetLayout;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyDescriptorSetLayout(_device: VkDevice, layout: VkDescriptorSetLayout, _alloc: *const c_void) {
    driver::destroy_set_layout(layout);
}
pub unsafe extern "system" fn vkCreateDescriptorPool(
    _device: VkDevice, _info: *const VkDescriptorPoolCreateInfo, _alloc: *const c_void, out: *mut VkDescriptorPool,
) -> VkResult {
    if out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_pool() as VkDescriptorPool;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyDescriptorPool(_device: VkDevice, pool: VkDescriptorPool, _alloc: *const c_void) {
    driver::destroy_pool(pool);
}
pub unsafe extern "system" fn vkAllocateDescriptorSets(
    _device: VkDevice, info: *const VkDescriptorSetAllocateInfo, out: *mut VkDescriptorSet,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    for i in 0..(*info).descriptorSetCount as usize {
        *out.add(i) = driver::alloc_set((*info).descriptorPool) as VkDescriptorSet;
    }
    SUCCESS
}
pub unsafe extern "system" fn vkUpdateDescriptorSets(
    _device: VkDevice, count: u32, writes: *const VkWriteDescriptorSet, _copies: u32, _copy: *const c_void,
) {
    if writes.is_null() { return; }
    for i in 0..count as usize {
        driver::update_write(&*writes.add(i));
    }
}
pub unsafe extern "system" fn vkCreateRenderPass(
    _device: VkDevice, _info: *const VkRenderPassCreateInfo, _alloc: *const c_void, out: *mut VkRenderPass,
) -> VkResult {
    if out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_render_pass() as VkRenderPass;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyRenderPass(_device: VkDevice, pass: VkRenderPass, _alloc: *const c_void) {
    driver::destroy_render_pass(pass);
}
pub unsafe extern "system" fn vkCreateFramebuffer(
    _device: VkDevice, info: *const VkFramebufferCreateInfo, _alloc: *const c_void, out: *mut VkFramebuffer,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_framebuffer(&*info) as VkFramebuffer;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyFramebuffer(_device: VkDevice, fb: VkFramebuffer, _alloc: *const c_void) {
    driver::destroy_framebuffer(fb);
}
pub unsafe extern "system" fn vkCreateCommandPool(
    device: VkDevice, _info: *const VkCommandPoolCreateInfo, _alloc: *const c_void, out: *mut VkCommandPool,
) -> VkResult {
    if out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_cmd_pool(device) as VkCommandPool;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroyCommandPool(_device: VkDevice, pool: VkCommandPool, _alloc: *const c_void) {
    driver::destroy_pool_cmd(pool);
}
pub unsafe extern "system" fn vkAllocateCommandBuffers(
    device: VkDevice, info: *const VkCommandBufferAllocateInfo, out: *mut VkCommandBuffer,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    for i in 0..(*info).commandBufferCount as usize {
        *out.add(i) = driver::alloc_cmd(device, (*info).commandPool) as VkCommandBuffer;
    }
    SUCCESS
}
pub unsafe extern "system" fn vkBeginCommandBuffer(cmd: VkCommandBuffer, _info: *const VkCommandBufferBeginInfo) -> VkResult {
    driver::begin_cmd(cmd)
}
pub unsafe extern "system" fn vkEndCommandBuffer(_cmd: VkCommandBuffer) -> VkResult { SUCCESS }
pub unsafe extern "system" fn vkCmdBindPipeline(cmd: VkCommandBuffer, point: i32, pipeline: VkPipeline) {
    driver::bind_pipeline(cmd, point, pipeline);
}
pub unsafe extern "system" fn vkCmdBindVertexBuffers(
    cmd: VkCommandBuffer, first: u32, count: u32, buffers: *const VkBuffer, offsets: *const VkDeviceSize,
) {
    driver::bind_vertex(cmd, first, count, buffers, offsets);
}
pub unsafe extern "system" fn vkCmdBindDescriptorSets(
    cmd: VkCommandBuffer, _point: i32, _layout: VkPipelineLayout, first: u32, count: u32,
    sets: *const VkDescriptorSet, _dyn: u32, _offs: *const u32,
) {
    driver::bind_sets(cmd, first, count, sets);
}
pub unsafe extern "system" fn vkCmdSetViewport(cmd: VkCommandBuffer, _first: u32, count: u32, viewports: *const VkViewport) {
    if count > 0 && !viewports.is_null() { driver::set_viewport(cmd, &*viewports); }
}
pub unsafe extern "system" fn vkCmdSetScissor(cmd: VkCommandBuffer, _first: u32, count: u32, scissors: *const VkRect2D) {
    if count > 0 && !scissors.is_null() { driver::set_scissor(cmd, &*scissors); }
}
pub unsafe extern "system" fn vkCmdBeginRenderPass(cmd: VkCommandBuffer, info: *const VkRenderPassBeginInfo, _contents: i32) {
    if !info.is_null() { driver::begin_render_pass(cmd, &*info); }
}
pub unsafe extern "system" fn vkCmdEndRenderPass(_cmd: VkCommandBuffer) {}
pub unsafe extern "system" fn vkCmdDraw(
    cmd: VkCommandBuffer, vertex_count: u32, instance_count: u32, first_vertex: u32, _first_instance: u32,
) {
    driver::cmd_draw(cmd, vertex_count, instance_count, first_vertex);
}
pub unsafe extern "system" fn vkCmdDispatch(cmd: VkCommandBuffer, x: u32, y: u32, z: u32) {
    driver::cmd_dispatch(cmd, x, y, z);
}
pub unsafe extern "system" fn vkCmdPipelineBarrier(
    _cmd: VkCommandBuffer, _src: VkFlags, _dst: VkFlags, _dep: VkFlags,
    _mb: u32, _pmb: *const c_void, _bb: u32, _pbb: *const c_void, _ib: u32, _pib: *const c_void,
) {}
pub unsafe extern "system" fn vkCmdCopyBufferToImage(
    cmd: VkCommandBuffer, src: VkBuffer, dst: VkImage, _layout: i32, count: u32, regions: *const VkBufferImageCopy,
) {
    if count > 0 && !regions.is_null() { driver::cmd_copy_b2i(cmd, src, dst, &*regions); }
}
pub unsafe extern "system" fn vkCmdCopyImageToBuffer(
    cmd: VkCommandBuffer, src: VkImage, _layout: i32, dst: VkBuffer, count: u32, regions: *const VkBufferImageCopy,
) {
    if count > 0 && !regions.is_null() { driver::cmd_copy_i2b(cmd, src, dst, &*regions); }
}
pub unsafe extern "system" fn vkCreateSampler(
    _device: VkDevice, info: *const VkSamplerCreateInfo, _alloc: *const c_void, out: *mut VkSampler,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_sampler(&*info) as VkSampler;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroySampler(_device: VkDevice, sampler: VkSampler, _alloc: *const c_void) {
    driver::destroy_sampler(sampler);
}
pub unsafe extern "system" fn vkCreateWin32SurfaceKHR(
    _instance: VkInstance, info: *const VkWin32SurfaceCreateInfoKHR, _alloc: *const c_void, out: *mut VkSurfaceKHR,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_surface((*info).hwnd) as VkSurfaceKHR;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroySurfaceKHR(_instance: VkInstance, surface: VkSurfaceKHR, _alloc: *const c_void) {
    driver::destroy_surface(surface);
}
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceSupportKHR(
    _p: VkPhysicalDevice, _queue: u32, _surface: VkSurfaceKHR, supported: *mut VkBool32,
) -> VkResult {
    if supported.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *supported = 1;
    SUCCESS
}
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
    p: VkPhysicalDevice, surface: VkSurfaceKHR, out: *mut VkSurfaceCapabilitiesKHR,
) -> VkResult {
    if out.is_null() || surface.is_null() || driver::phys(p).is_none() {
        return ERROR_INITIALIZATION_FAILED;
    }
    // 0xFFFFFFFF tells the client to choose the swapchain size.
    *out = VkSurfaceCapabilitiesKHR {
        minImageCount: 1,
        maxImageCount: 8,
        currentExtent: VkExtent2D { width: 0xFFFF_FFFF, height: 0xFFFF_FFFF },
        minImageExtent: VkExtent2D { width: 1, height: 1 },
        maxImageExtent: VkExtent2D { width: 8192, height: 8192 },
        maxImageArrayLayers: 1,
        supportedTransforms: SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
        currentTransform: SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
        supportedCompositeAlpha: COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
        supportedUsageFlags: IMAGE_USAGE_COLOR_ATTACHMENT_BIT
            | IMAGE_USAGE_TRANSFER_SRC_BIT
            | IMAGE_USAGE_TRANSFER_DST_BIT
            | IMAGE_USAGE_SAMPLED_BIT,
    };
    SUCCESS
}
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfaceFormatsKHR(
    p: VkPhysicalDevice, surface: VkSurfaceKHR, count: *mut u32, formats: *mut VkSurfaceFormatKHR,
) -> VkResult {
    if surface.is_null() || driver::phys(p).is_none() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let items = [
        VkSurfaceFormatKHR { format: FORMAT_B8G8R8A8_UNORM, colorSpace: COLOR_SPACE_SRGB_NONLINEAR_KHR },
        VkSurfaceFormatKHR { format: FORMAT_R8G8B8A8_UNORM, colorSpace: COLOR_SPACE_SRGB_NONLINEAR_KHR },
    ];
    let bytes = std::slice::from_raw_parts(items.as_ptr() as *const u8, std::mem::size_of_val(&items));
    write_count(count, items.len() as u32, formats as *mut u8, std::mem::size_of::<VkSurfaceFormatKHR>(), bytes)
}
pub unsafe extern "system" fn vkGetPhysicalDeviceSurfacePresentModesKHR(
    p: VkPhysicalDevice, surface: VkSurfaceKHR, count: *mut u32, modes: *mut i32,
) -> VkResult {
    if surface.is_null() || driver::phys(p).is_none() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let items = [PRESENT_MODE_FIFO_KHR, PRESENT_MODE_IMMEDIATE_KHR];
    let bytes = std::slice::from_raw_parts(items.as_ptr() as *const u8, std::mem::size_of_val(&items));
    write_count(count, items.len() as u32, modes as *mut u8, std::mem::size_of::<i32>(), bytes)
}
pub unsafe extern "system" fn vkGetPhysicalDeviceWin32PresentationSupportKHR(p: VkPhysicalDevice, queue: u32) -> VkBool32 {
    if driver::phys(p).is_some() && queue == 0 { 1 } else { 0 }
}
pub unsafe extern "system" fn vkCreateSwapchainKHR(
    _device: VkDevice, info: *const VkSwapchainCreateInfoKHR, _alloc: *const c_void, out: *mut VkSwapchainKHR,
) -> VkResult {
    if info.is_null() || out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    match driver::create_swapchain(&*info) {
        Ok(s) => { *out = s as VkSwapchainKHR; SUCCESS }
        Err(e) => e,
    }
}
pub unsafe extern "system" fn vkDestroySwapchainKHR(_device: VkDevice, swap: VkSwapchainKHR, _alloc: *const c_void) {
    driver::destroy_swapchain(swap);
}
pub unsafe extern "system" fn vkGetSwapchainImagesKHR(
    _device: VkDevice, swap: VkSwapchainKHR, count: *mut u32, images: *mut VkImage,
) -> VkResult {
    driver::swapchain_images(swap, count, images)
}
pub unsafe extern "system" fn vkCreateSemaphore(
    _device: VkDevice, _info: *const VkSemaphoreCreateInfo, _alloc: *const c_void, out: *mut VkSemaphore,
) -> VkResult {
    if out.is_null() { return ERROR_INITIALIZATION_FAILED; }
    *out = driver::create_semaphore() as VkSemaphore;
    SUCCESS
}
pub unsafe extern "system" fn vkDestroySemaphore(_device: VkDevice, sem: VkSemaphore, _alloc: *const c_void) {
    driver::destroy_semaphore(sem);
}
pub unsafe extern "system" fn vkAcquireNextImageKHR(
    _device: VkDevice, swap: VkSwapchainKHR, _timeout: u64, semaphore: VkSemaphore, _fence: VkFence, index: *mut u32,
) -> VkResult {
    driver::acquire(swap, semaphore, index)
}
pub unsafe extern "system" fn vkQueuePresentKHR(_queue: VkQueue, info: *const VkPresentInfoKHR) -> VkResult {
    if info.is_null() { return ERROR_INITIALIZATION_FAILED; }
    driver::present(&*info)
}
