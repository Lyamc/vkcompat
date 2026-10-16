//! Vulkan objects and command execution. Hardware draws that fail drop the hw flag
//! so the pixel test reports the compiler log instead of a silent CPU fallback.

use crate::emit::{self, Dialect, Emitted};
use crate::gpu::{self, Backend, DrawDesc, FixedDesc, Tex};
use crate::raster::{self, FragTarget, Rect, Viewport};
use crate::spirv::{self, FfInfo, FfKind, ImageBind, Io, Module};
use crate::vk::*;
use std::ffi::{c_char, c_void, CStr, CString};

#[repr(C)]
pub struct Instance {
    pub magic: usize,
    phys: Vec<Box<PhysDevice>>,
}

#[repr(C)]
pub struct PhysDevice {
    pub magic: usize,
    desc: gpu::PhysDesc,
    props: VkPhysicalDeviceProperties,
    memory: VkPhysicalDeviceMemoryProperties,
    queue_props: VkQueueFamilyProperties,
}

#[repr(C)]
pub struct Device {
    pub magic: usize,
    phys: *mut PhysDevice,
    backend: Option<Box<dyn Backend>>,
    force_sw: bool,
    queue: *mut Queue,
}

#[repr(C)]
pub struct Queue {
    pub magic: usize,
    dev: *mut Device,
}

pub struct Memory {
    bytes: Vec<u8>,
    mapped: bool,
    buffers: Vec<*mut Buffer>,
}

pub struct Buffer {
    size: u64,
    mem: *mut Memory,
    offset: u64,
    cpu_newer: bool,
}

pub struct Image {
    width: u32,
    height: u32,
    format: i32,
    store: Vec<u8>,
    defined: bool,
}

pub struct ImageView {
    image: *mut Image,
}

pub struct ShaderModule {
    words: Vec<u32>,
}

pub struct Pipeline {
    pub hw: bool,
    pub sw: bool,
    pub compute: bool,
    pub ff_hw: bool,
    ff: FfInfo,
    vs: Option<Module>,
    fs: Option<Module>,
    cs: Option<Module>,
    emitted: Option<Emitted>,
    note: CString,
    topology: i32,
    polygon: i32,
    stride: u32,
    attrs: Vec<(u32, u32, i32)>,
    components: u8,
}

#[derive(Clone, Copy)]
struct Slot {
    kind: i32,
    buffer: *mut Buffer,
    offset: u64,
    range: u64,
    image: *mut Image,
    nearest: bool,
    clamp: bool,
}

impl Slot {
    fn empty() -> Self {
        Self {
            kind: 0,
            buffer: std::ptr::null_mut(),
            offset: 0,
            range: 0,
            image: std::ptr::null_mut(),
            nearest: true,
            clamp: true,
        }
    }
}

pub struct DescriptorSet {
    slots: [Slot; 16],
}

pub struct DescriptorPool {
    sets: Vec<*mut DescriptorSet>,
}

pub struct Sampler {
    nearest: bool,
    clamp: bool,
}

pub struct RenderPass {
    _pad: u32,
}

pub struct Framebuffer {
    views: Vec<*mut ImageView>,
    width: u32,
    height: u32,
}

pub struct PipelineLayout {
    _pad: u32,
}
pub struct SetLayout {
    _pad: u32,
}
pub struct PipelineCache {
    _pad: u32,
}

pub struct Surface {
    hwnd: isize,
}

pub struct Swapchain {
    images: Vec<Box<Image>>,
    hwnd: isize,
    acquired: u32,
}

pub struct Semaphore {
    signaled: bool,
}
pub struct Fence {
    signaled: bool,
}

#[derive(Clone, Copy)]
struct VbBind {
    buffer: *mut Buffer,
    offset: u64,
}

struct DrawSnap {
    pipe: *mut Pipeline,
    vb: *mut Buffer,
    vb_offset: u64,
    set: *mut DescriptorSet,
    vp: Viewport,
    sc: Rect,
    image: *mut Image,
    clear: [f32; 4],
    vertex_count: u32,
    first_vertex: u32,
    instance_count: u32,
}

struct DispatchSnap {
    pipe: *mut Pipeline,
    set: *mut DescriptorSet,
    groups: [u32; 3],
}

enum Cmd {
    Clear { image: *mut Image, color: [f32; 4] },
    CopyB2I { src: *mut Buffer, dst: *mut Image, src_off: u64, w: u32, h: u32 },
    CopyI2B { src: *mut Image, dst: *mut Buffer, dst_off: u64, w: u32, h: u32 },
    Draw(DrawSnap),
    Dispatch(DispatchSnap),
}

#[repr(C)]
pub struct CmdBuf {
    pub magic: usize,
    dev: *mut Device,
    cmds: Vec<Cmd>,
    pipe_gfx: *mut Pipeline,
    pipe_cs: *mut Pipeline,
    vbs: [VbBind; 4],
    sets: [*mut DescriptorSet; 4],
    vp: Viewport,
    sc: Rect,
    target: *mut Image,
    clear: [f32; 4],
}

pub struct CommandPool {
    bufs: Vec<*mut CmdBuf>,
}

fn leak<T>(v: T) -> *mut T {
    Box::into_raw(Box::new(v))
}

fn note_from(text: impl Into<String>) -> CString {
    let s = text.into().replace('\0', " ");
    CString::new(s).unwrap_or_else(|_| CString::new("").unwrap())
}

fn empty_note() -> CString {
    CString::new("").unwrap()
}

pub fn gfx_pipeline_uses_hw(pipeline: VkPipeline) -> i32 {
    let p = pipeline as *mut Pipeline;
    if p.is_null() {
        return 0;
    }
    unsafe {
        if (*p).hw || (*p).ff_hw {
            1
        } else {
            0
        }
    }
}

pub fn gfx_pipeline_note(pipeline: VkPipeline) -> *const c_char {
    let p = pipeline as *mut Pipeline;
    if p.is_null() {
        return b"\0".as_ptr() as *const c_char;
    }
    unsafe { (*p).note.as_ptr() }
}

fn check_extensions(count: u32, names: *const *const c_char, allowed: &[&str]) -> VkResult {
    if count == 0 || names.is_null() {
        return SUCCESS;
    }
    for i in 0..count as usize {
        let p = unsafe { *names.add(i) };
        let s = unsafe { cstr(p) };
        if !allowed.iter().any(|a| *a == s) {
            eprintln!("gfx: unsupported extension {s}");
            return ERROR_EXTENSION_NOT_PRESENT;
        }
    }
    SUCCESS
}

fn api_compatible(info: *const VkApplicationInfo) -> bool {
    if info.is_null() {
        return true;
    }
    let v = unsafe { (*info).apiVersion };
    if v == 0 {
        return true;
    }
    let major = version_major(v);
    let minor = version_minor(v);
    !(major > 1 || (major == 1 && minor > 0))
}

pub unsafe fn create_instance(info: *const VkInstanceCreateInfo, out: *mut VkInstance) -> VkResult {
    if info.is_null() || out.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let info = &*info;
    if info.enabledLayerCount != 0 {
        eprintln!("gfx: no layers");
        return ERROR_LAYER_NOT_PRESENT;
    }
    let ext = check_extensions(
        info.enabledExtensionCount,
        info.ppEnabledExtensionNames,
        crate::vk::instance_extension_names(),
    );
    if ext != SUCCESS {
        return ext;
    }
    if !info.pApplicationInfo.is_null() && !api_compatible(info.pApplicationInfo) {
        return ERROR_INCOMPATIBLE_DRIVER;
    }
    let descs = gpu::enumerate();
    let mut phys = Vec::new();
    for d in descs {
        let props = advertised_properties(&d.name, d.vendor, d.device, d.device_type);
        phys.push(Box::new(PhysDevice {
            magic: ICD_LOADER_MAGIC,
            desc: d,
            props,
            memory: advertised_memory(),
            queue_props: advertised_queue(),
        }));
    }
    let inst = leak(Instance {
        magic: ICD_LOADER_MAGIC,
        phys,
    });
    *out = inst as VkInstance;
    SUCCESS
}

pub unsafe fn destroy_instance(instance: VkInstance) {
    if instance.is_null() {
        return;
    }
    drop(Box::from_raw(instance as *mut Instance));
}

pub unsafe fn enumerate_physical_devices(
    instance: VkInstance,
    count: *mut u32,
    devices: *mut VkPhysicalDevice,
) -> VkResult {
    if instance.is_null() || count.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let inst = &*(instance as *mut Instance);
    let n = inst.phys.len() as u32;
    if devices.is_null() {
        *count = n;
        return SUCCESS;
    }
    let room = *count;
    let write = room.min(n) as usize;
    for i in 0..write {
        *devices.add(i) = (&*inst.phys[i]) as *const PhysDevice as VkPhysicalDevice;
    }
    *count = write as u32;
    if room < n {
        INCOMPLETE
    } else {
        SUCCESS
    }
}

pub unsafe fn phys<'a>(p: VkPhysicalDevice) -> Option<&'a PhysDevice> {
    if p.is_null() {
        None
    } else {
        Some(&*(p as *mut PhysDevice))
    }
}

pub unsafe fn create_device(
    physical: VkPhysicalDevice,
    info: *const VkDeviceCreateInfo,
    out: *mut VkDevice,
) -> VkResult {
    if physical.is_null() || info.is_null() || out.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let info = &*info;
    if info.enabledLayerCount != 0 {
        eprintln!("gfx: no layers");
        return ERROR_LAYER_NOT_PRESENT;
    }
    let ext = check_extensions(
        info.enabledExtensionCount,
        info.ppEnabledExtensionNames,
        &[EXT_SWAPCHAIN, EXT_DYNAMIC_RENDERING],
    );
    if ext != SUCCESS {
        return ext;
    }
    let phys = &*(physical as *mut PhysDevice);
    let backend = match phys.desc.kind {
        gpu::Kind::Cpu => None,
        _ => match gpu::open(&phys.desc) {
            Ok(b) => Some(b),
            Err(e) => {
                eprintln!("vkcompat: device open failed: {e}");
                return ERROR_INITIALIZATION_FAILED;
            }
        },
    };
    let force_sw = std::env::var("GFX_FORCE_SW").map(|v| v == "1").unwrap_or(false);
    let dev = leak(Device {
        magic: ICD_LOADER_MAGIC,
        phys: physical as *mut PhysDevice,
        backend,
        force_sw,
        queue: std::ptr::null_mut(),
    });
    let queue = leak(Queue {
        magic: ICD_LOADER_MAGIC,
        dev,
    });
    (*dev).queue = queue;
    *out = dev as VkDevice;
    gpu::logf(&format!("opened {}", phys.desc.name));
    SUCCESS
}

pub unsafe fn destroy_device(device: VkDevice) {
    if device.is_null() {
        return;
    }
    let dev = device as *mut Device;
    if !(*dev).queue.is_null() {
        drop(Box::from_raw((*dev).queue));
        (*dev).queue = std::ptr::null_mut();
    }
    drop(Box::from_raw(dev));
}

pub unsafe fn get_queue(device: VkDevice) -> VkQueue {
    if device.is_null() {
        return std::ptr::null_mut();
    }
    let dev = &*(device as *mut Device);
    dev.queue as VkQueue
}

pub unsafe fn alloc_memory(size: u64) -> *mut Memory {
    leak(Memory {
        bytes: vec![0u8; size as usize],
        mapped: false,
        buffers: Vec::new(),
    })
}

pub unsafe fn free_memory(mem: VkDeviceMemory) {
    if !mem.is_null() {
        drop(Box::from_raw(mem as *mut Memory));
    }
}

pub unsafe fn map_memory(mem: VkDeviceMemory, offset: u64, out: *mut *mut c_void) -> VkResult {
    if mem.is_null() || out.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let m = &mut *(mem as *mut Memory);
    m.mapped = true;
    *out = m.bytes.as_mut_ptr().add(offset as usize) as *mut c_void;
    SUCCESS
}

pub unsafe fn unmap_memory(mem: VkDeviceMemory) {
    if mem.is_null() {
        return;
    }
    let m = &mut *(mem as *mut Memory);
    m.mapped = false;
    for b in &m.buffers {
        if !b.is_null() {
            (**b).cpu_newer = true;
        }
    }
}

pub unsafe fn create_buffer(size: u64) -> *mut Buffer {
    leak(Buffer {
        size,
        mem: std::ptr::null_mut(),
        offset: 0,
        cpu_newer: true,
    })
}

pub unsafe fn destroy_buffer(buffer: VkBuffer) {
    if !buffer.is_null() {
        drop(Box::from_raw(buffer as *mut Buffer));
    }
}

pub fn buffer_requirements(size: u64) -> VkMemoryRequirements {
    VkMemoryRequirements {
        size: align_up(size, 256),
        alignment: 256,
        memoryTypeBits: 1,
    }
}

pub unsafe fn bind_buffer(buffer: VkBuffer, mem: VkDeviceMemory, offset: u64) -> VkResult {
    if buffer.is_null() || mem.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let b = &mut *(buffer as *mut Buffer);
    let m = &mut *(mem as *mut Memory);
    b.mem = m;
    b.offset = offset;
    b.cpu_newer = true;
    m.buffers.push(b);
    SUCCESS
}

pub unsafe fn create_image(info: &VkImageCreateInfo) -> Result<*mut Image, VkResult> {
    if info.imageType != IMAGE_TYPE_2D || info.mipLevels > 1 || info.arrayLayers > 1 || info.samples != SAMPLE_COUNT_1_BIT
    {
        return Err(ERROR_FORMAT_NOT_SUPPORTED);
    }
    if format_bpp(info.format) == 0 {
        return Err(ERROR_FORMAT_NOT_SUPPORTED);
    }
    let w = info.extent.width;
    let h = info.extent.height;
    let bpp = format_bpp(info.format) as usize;
    Ok(leak(Image {
        width: w,
        height: h,
        format: info.format,
        store: vec![0u8; (w as usize) * (h as usize) * bpp],
        defined: false,
    }))
}

pub unsafe fn destroy_image(image: VkImage) {
    if image.is_null() {
        return;
    }
    let img = &*(image as *mut Image);
    if img.format == i32::MIN {
        return;
    }
    drop(Box::from_raw(image as *mut Image));
}

pub fn image_requirements(image: &Image) -> VkMemoryRequirements {
    let bytes = image.store.len() as u64;
    VkMemoryRequirements {
        size: align_up(bytes.max(4), 256),
        alignment: 256,
        memoryTypeBits: 1,
    }
}

pub unsafe fn create_image_view(image: VkImage) -> *mut ImageView {
    leak(ImageView {
        image: image as *mut Image,
    })
}

pub unsafe fn destroy_image_view(view: VkImageView) {
    if !view.is_null() {
        drop(Box::from_raw(view as *mut ImageView));
    }
}

pub unsafe fn create_shader(code: *const u32, bytes: usize) -> *mut ShaderModule {
    let words = if code.is_null() || bytes < 4 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(code, bytes / 4).to_vec()
    };
    leak(ShaderModule { words })
}

pub unsafe fn destroy_shader(module: VkShaderModule) {
    if !module.is_null() {
        drop(Box::from_raw(module as *mut ShaderModule));
    }
}

fn module_of(module: VkShaderModule) -> Result<Module, String> {
    if module.is_null() {
        return Err("null shader module".into());
    }
    let words = unsafe { &(*(module as *mut ShaderModule)).words };
    spirv::parse(words)
}

fn components_of(fmt: i32) -> u8 {
    match fmt {
        FORMAT_R32_SFLOAT => 1,
        FORMAT_R32G32_SFLOAT => 2,
        FORMAT_R32G32B32_SFLOAT => 3,
        FORMAT_R32G32B32A32_SFLOAT => 4,
        _ => 2,
    }
}

fn blank_pipeline() -> Pipeline {
    Pipeline {
        hw: false,
        sw: false,
        compute: false,
        ff_hw: false,
        ff: FfInfo {
            kind: FfKind::None,
            color: [0.0; 4],
            uv_loc: 0,
            set: 0,
            binding: 0,
        },
        vs: None,
        fs: None,
        cs: None,
        emitted: None,
        note: empty_note(),
        topology: PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
        polygon: POLYGON_MODE_FILL,
        stride: 8,
        attrs: Vec::new(),
        components: 2,
    }
}

pub unsafe fn create_graphics(device: VkDevice, info: &VkGraphicsPipelineCreateInfo) -> *mut Pipeline {
    let dev = &*(device as *mut Device);
    let tier = dev.backend.as_ref().map(|b| b.tier());
    let dialect = tier.map(|t| t.dialect).unwrap_or(Dialect::Hlsl5);
    let mut pipe = blank_pipeline();
    if !info.pVertexInputState.is_null() {
        let vi = &*info.pVertexInputState;
        if vi.vertexBindingDescriptionCount > 0 && !vi.pVertexBindingDescriptions.is_null() {
            pipe.stride = (*vi.pVertexBindingDescriptions).stride.max(1);
        }
        if vi.vertexAttributeDescriptionCount > 0 && !vi.pVertexAttributeDescriptions.is_null() {
            let attrs = std::slice::from_raw_parts(
                vi.pVertexAttributeDescriptions,
                vi.vertexAttributeDescriptionCount as usize,
            );
            for a in attrs {
                pipe.attrs.push((a.location, a.offset, a.format));
                if a.location == 0 {
                    pipe.components = components_of(a.format);
                }
            }
        }
    }
    if !info.pInputAssemblyState.is_null() {
        pipe.topology = (*info.pInputAssemblyState).topology;
    }
    if !info.pRasterizationState.is_null() {
        pipe.polygon = (*info.pRasterizationState).polygonMode;
    }
    let stages = if info.pStages.is_null() {
        &[][..]
    } else {
        std::slice::from_raw_parts(info.pStages, info.stageCount as usize)
    };
    let mut vs = None;
    let mut fs = None;
    for stage in stages {
        match module_of(stage.module) {
            Ok(m) => {
                if stage.stage & SHADER_STAGE_VERTEX_BIT != 0 {
                    vs = Some(m);
                } else if stage.stage & SHADER_STAGE_FRAGMENT_BIT != 0 {
                    fs = Some(m);
                }
            }
            Err(e) => {
                pipe.note = note_from(e);
                pipe.sw = false;
                return leak(pipe);
            }
        }
    }
    let (Some(vs_m), Some(fs_m)) = (vs, fs) else {
        pipe.note = note_from("graphics pipeline is missing a stage");
        pipe.sw = false;
        return leak(pipe);
    };
    if dialect == Dialect::Fixed {
        pipe.ff = spirv::fixed_function(&fs_m);
        pipe.hw = false;
        pipe.ff_hw = tier.map(|t| t.fixed_function).unwrap_or(false) && !matches!(pipe.ff.kind, FfKind::None);
        pipe.sw = true;
        pipe.vs = Some(vs_m);
        pipe.fs = Some(fs_m);
        return leak(pipe);
    }
    match emit::emit_graphics(&vs_m, &fs_m, dialect) {
        Ok(e) => {
            pipe.hw = dev.backend.is_some();
            pipe.sw = true;
            pipe.emitted = Some(e);
        }
        Err(e) => {
            pipe.hw = false;
            pipe.sw = true;
            pipe.note = note_from(e);
        }
    }
    pipe.vs = Some(vs_m);
    pipe.fs = Some(fs_m);
    leak(pipe)
}

pub unsafe fn create_compute(device: VkDevice, info: &VkComputePipelineCreateInfo) -> *mut Pipeline {
    let dev = &*(device as *mut Device);
    let tier = dev.backend.as_ref().map(|b| b.tier());
    let mut pipe = blank_pipeline();
    pipe.compute = true;
    let module = match module_of(info.stage.module) {
        Ok(m) => m,
        Err(e) => {
            pipe.note = note_from(e);
            pipe.sw = false;
            return leak(pipe);
        }
    };
    let compute_hw = tier.map(|t| t.compute).unwrap_or(false);
    if !compute_hw {
        pipe.hw = false;
        pipe.sw = true;
        pipe.note = note_from("compute is software");
        pipe.cs = Some(module);
        return leak(pipe);
    }
    match emit::emit_compute(&module, Dialect::Hlsl5) {
        Ok(e) => {
            pipe.hw = true;
            pipe.sw = true;
            pipe.emitted = Some(e);
        }
        Err(e) => {
            pipe.hw = false;
            pipe.sw = true;
            pipe.note = note_from(e);
        }
    }
    pipe.cs = Some(module);
    leak(pipe)
}

pub unsafe fn destroy_pipeline(pipeline: VkPipeline) {
    if !pipeline.is_null() {
        drop(Box::from_raw(pipeline as *mut Pipeline));
    }
}

pub fn create_layout() -> *mut PipelineLayout {
    leak(PipelineLayout { _pad: 0 })
}
pub unsafe fn destroy_layout(layout: VkPipelineLayout) {
    if !layout.is_null() {
        drop(Box::from_raw(layout as *mut PipelineLayout));
    }
}
pub fn create_set_layout() -> *mut SetLayout {
    leak(SetLayout { _pad: 0 })
}
pub unsafe fn destroy_set_layout(layout: VkDescriptorSetLayout) {
    if !layout.is_null() {
        drop(Box::from_raw(layout as *mut SetLayout));
    }
}
pub fn create_pool() -> *mut DescriptorPool {
    leak(DescriptorPool { sets: Vec::new() })
}
pub unsafe fn destroy_pool(pool: VkDescriptorPool) {
    if pool.is_null() {
        return;
    }
    let p = Box::from_raw(pool as *mut DescriptorPool);
    for s in p.sets {
        if !s.is_null() {
            drop(Box::from_raw(s));
        }
    }
}
pub unsafe fn alloc_set(pool: VkDescriptorPool) -> *mut DescriptorSet {
    let set = leak(DescriptorSet {
        slots: [Slot::empty(); 16],
    });
    if !pool.is_null() {
        (*(pool as *mut DescriptorPool)).sets.push(set);
    }
    set
}

pub unsafe fn update_write(write: &VkWriteDescriptorSet) {
    if write.dstSet.is_null() {
        return;
    }
    let set = &mut *(write.dstSet as *mut DescriptorSet);
    let binding = write.dstBinding as usize;
    if binding >= set.slots.len() {
        return;
    }
    let mut slot = Slot::empty();
    slot.kind = write.descriptorType;
    if write.descriptorType == DESCRIPTOR_TYPE_UNIFORM_BUFFER || write.descriptorType == DESCRIPTOR_TYPE_STORAGE_BUFFER
    {
        if !write.pBufferInfo.is_null() {
            let info = &*write.pBufferInfo;
            slot.buffer = info.buffer as *mut Buffer;
            slot.offset = info.offset;
            slot.range = info.range;
        }
    } else if write.descriptorType == DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER && !write.pImageInfo.is_null() {
        let info = &*write.pImageInfo;
        if !info.imageView.is_null() {
            slot.image = (*(info.imageView as *mut ImageView)).image;
        }
        if !info.sampler.is_null() {
            let samp = &*(info.sampler as *mut Sampler);
            slot.nearest = samp.nearest;
            slot.clamp = samp.clamp;
        }
    }
    set.slots[binding] = slot;
}

pub unsafe fn create_sampler(info: &VkSamplerCreateInfo) -> *mut Sampler {
    leak(Sampler {
        nearest: info.magFilter != FILTER_LINEAR && info.minFilter != FILTER_LINEAR,
        clamp: info.addressModeU == SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
    })
}
pub unsafe fn destroy_sampler(sampler: VkSampler) {
    if !sampler.is_null() {
        drop(Box::from_raw(sampler as *mut Sampler));
    }
}

pub fn create_render_pass() -> *mut RenderPass {
    leak(RenderPass { _pad: 0 })
}
pub unsafe fn destroy_render_pass(pass: VkRenderPass) {
    if !pass.is_null() {
        drop(Box::from_raw(pass as *mut RenderPass));
    }
}

pub unsafe fn create_framebuffer(info: &VkFramebufferCreateInfo) -> *mut Framebuffer {
    let mut views = Vec::new();
    if info.attachmentCount > 0 && !info.pAttachments.is_null() {
        let slice = std::slice::from_raw_parts(info.pAttachments, info.attachmentCount as usize);
        for v in slice {
            views.push(*v as *mut ImageView);
        }
    }
    leak(Framebuffer {
        views,
        width: info.width,
        height: info.height,
    })
}
pub unsafe fn destroy_framebuffer(fb: VkFramebuffer) {
    if !fb.is_null() {
        drop(Box::from_raw(fb as *mut Framebuffer));
    }
}

fn fresh_cmd(dev: *mut Device) -> CmdBuf {
    CmdBuf {
        magic: ICD_LOADER_MAGIC,
        dev,
        cmds: Vec::new(),
        pipe_gfx: std::ptr::null_mut(),
        pipe_cs: std::ptr::null_mut(),
        vbs: [VbBind { buffer: std::ptr::null_mut(), offset: 0 }; 4],
        sets: [std::ptr::null_mut(); 4],
        vp: Viewport { x: 0.0, y: 0.0, w: 0.0, h: 0.0, zmin: 0.0, zmax: 1.0 },
        sc: Rect { x: 0, y: 0, w: 0, h: 0 },
        target: std::ptr::null_mut(),
        clear: [0.0, 0.0, 0.0, 1.0],
    }
}

pub unsafe fn create_cmd_pool(device: VkDevice) -> *mut CommandPool {
    let _ = device;
    leak(CommandPool { bufs: Vec::new() })
}
pub unsafe fn destroy_pool_cmd(pool: VkCommandPool) {
    if pool.is_null() {
        return;
    }
    let p = Box::from_raw(pool as *mut CommandPool);
    for b in p.bufs {
        if !b.is_null() {
            drop(Box::from_raw(b));
        }
    }
}
pub unsafe fn alloc_cmd(device: VkDevice, pool: VkCommandPool) -> *mut CmdBuf {
    let buf = leak(fresh_cmd(device as *mut Device));
    if !pool.is_null() {
        (*(pool as *mut CommandPool)).bufs.push(buf);
    }
    buf
}

pub unsafe fn begin_cmd(cmd: VkCommandBuffer) -> VkResult {
    if cmd.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    // The first word is the loader dispatch pointer once vulkan-1.dll has
    // replaced ICD_LOADER_MAGIC. Replacing the struct would drop that word.
    let c = &mut *(cmd as *mut CmdBuf);
    c.cmds.clear();
    c.pipe_gfx = std::ptr::null_mut();
    c.pipe_cs = std::ptr::null_mut();
    c.vbs = [VbBind { buffer: std::ptr::null_mut(), offset: 0 }; 4];
    c.sets = [std::ptr::null_mut(); 4];
    c.vp = Viewport { x: 0.0, y: 0.0, w: 0.0, h: 0.0, zmin: 0.0, zmax: 1.0 };
    c.sc = Rect { x: 0, y: 0, w: 0, h: 0 };
    c.target = std::ptr::null_mut();
    c.clear = [0.0, 0.0, 0.0, 1.0];
    SUCCESS
}

pub unsafe fn bind_pipeline(cmd: VkCommandBuffer, bind_point: i32, pipeline: VkPipeline) {
    if cmd.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    if bind_point == PIPELINE_BIND_POINT_COMPUTE {
        c.pipe_cs = pipeline as *mut Pipeline;
    } else {
        c.pipe_gfx = pipeline as *mut Pipeline;
    }
}

pub unsafe fn bind_vertex(cmd: VkCommandBuffer, first: u32, count: u32, buffers: *const VkBuffer, offsets: *const u64) {
    if cmd.is_null() || buffers.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    for i in 0..count {
        let slot = (first + i) as usize;
        if slot < c.vbs.len() {
            c.vbs[slot].buffer = *buffers.add(i as usize) as *mut Buffer;
            c.vbs[slot].offset = if offsets.is_null() { 0 } else { *offsets.add(i as usize) };
        }
    }
}

pub unsafe fn bind_sets(cmd: VkCommandBuffer, first: u32, count: u32, sets: *const VkDescriptorSet) {
    if cmd.is_null() || sets.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    for i in 0..count {
        let slot = (first + i) as usize;
        if slot < c.sets.len() {
            c.sets[slot] = *sets.add(i as usize) as *mut DescriptorSet;
        }
    }
}

pub unsafe fn set_viewport(cmd: VkCommandBuffer, vp: &VkViewport) {
    if cmd.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    c.vp = Viewport {
        x: vp.x,
        y: vp.y,
        w: vp.width,
        h: vp.height,
        zmin: vp.minDepth,
        zmax: vp.maxDepth,
    };
}

pub unsafe fn set_scissor(cmd: VkCommandBuffer, sc: &VkRect2D) {
    if cmd.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    c.sc = Rect {
        x: sc.offset.x,
        y: sc.offset.y,
        w: sc.extent.width as i32,
        h: sc.extent.height as i32,
    };
}

pub unsafe fn begin_render_pass(cmd: VkCommandBuffer, info: &VkRenderPassBeginInfo) {
    if cmd.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    let mut image = std::ptr::null_mut();
    if !info.framebuffer.is_null() {
        let fb = &*(info.framebuffer as *mut Framebuffer);
        if let Some(view) = fb.views.first() {
            if !view.is_null() {
                image = (**view).image;
            }
        }
    }
    let mut color = [0.0, 0.0, 0.0, 1.0];
    if info.clearValueCount > 0 && !info.pClearValues.is_null() {
        color = (*info.pClearValues).float32;
    }
    c.target = image;
    c.clear = color;
    if !image.is_null() {
        c.cmds.push(Cmd::Clear { image, color });
    }
}

pub unsafe fn cmd_draw(cmd: VkCommandBuffer, vertex_count: u32, instance_count: u32, first_vertex: u32) {
    if cmd.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    if c.pipe_gfx.is_null() || c.target.is_null() {
        return;
    }
    c.cmds.push(Cmd::Draw(DrawSnap {
        pipe: c.pipe_gfx,
        vb: c.vbs[0].buffer,
        vb_offset: c.vbs[0].offset,
        set: c.sets[0],
        vp: c.vp,
        sc: c.sc,
        image: c.target,
        clear: c.clear,
        vertex_count,
        first_vertex,
        instance_count,
    }));
}

pub unsafe fn cmd_dispatch(cmd: VkCommandBuffer, x: u32, y: u32, z: u32) {
    if cmd.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    c.cmds.push(Cmd::Dispatch(DispatchSnap {
        pipe: c.pipe_cs,
        set: c.sets[0],
        groups: [x, y, z],
    }));
}

pub unsafe fn cmd_copy_b2i(cmd: VkCommandBuffer, src: VkBuffer, dst: VkImage, region: &VkBufferImageCopy) {
    if cmd.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    c.cmds.push(Cmd::CopyB2I {
        src: src as *mut Buffer,
        dst: dst as *mut Image,
        src_off: region.bufferOffset,
        w: region.imageExtent.width,
        h: region.imageExtent.height,
    });
}

pub unsafe fn cmd_copy_i2b(cmd: VkCommandBuffer, src: VkImage, dst: VkBuffer, region: &VkBufferImageCopy) {
    if cmd.is_null() {
        return;
    }
    let c = &mut *(cmd as *mut CmdBuf);
    c.cmds.push(Cmd::CopyI2B {
        src: src as *mut Image,
        dst: dst as *mut Buffer,
        dst_off: region.bufferOffset,
        w: region.imageExtent.width,
        h: region.imageExtent.height,
    });
}

pub unsafe fn queue_submit(queue: VkQueue, info: &VkSubmitInfo) -> VkResult {
    if queue.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let q = &*(queue as *mut Queue);
    if info.commandBufferCount > 0 && !info.pCommandBuffers.is_null() {
        let bufs = std::slice::from_raw_parts(info.pCommandBuffers, info.commandBufferCount as usize);
        for b in bufs {
            if !b.is_null() {
                let cmd = &mut *(*b as *mut CmdBuf);
                let cmds = std::mem::take(&mut cmd.cmds);
                for c in cmds {
                    exec_cmd(q.dev, c);
                }
            }
        }
    }
    if info.signalSemaphoreCount > 0 && !info.pSignalSemaphores.is_null() {
        for i in 0..info.signalSemaphoreCount as usize {
            let s = *info.pSignalSemaphores.add(i);
            if !s.is_null() {
                (*(s as *mut Semaphore)).signaled = true;
            }
        }
    }
    SUCCESS
}

fn exec_cmd(dev: *mut Device, cmd: Cmd) {
    unsafe {
        match cmd {
            Cmd::Clear { image, color } => clear_image(image, color),
            Cmd::CopyB2I { src, dst, src_off, w, h } => copy_b2i(src, dst, src_off, w, h),
            Cmd::CopyI2B { src, dst, dst_off, w, h } => copy_i2b(src, dst, dst_off, w, h),
            Cmd::Draw(snap) => exec_draw(dev, snap),
            Cmd::Dispatch(snap) => exec_dispatch(dev, snap),
        }
    }
}

unsafe fn clear_image(image: *mut Image, color: [f32; 4]) {
    if image.is_null() {
        return;
    }
    let img = &mut *image;
    let mut target = FragTarget {
        pixels: &mut img.store,
        width: img.width,
        height: img.height,
        format: img.format,
    };
    raster::clear(&mut target, color);
    img.defined = true;
}

unsafe fn read_buffer(buf: *mut Buffer, offset: u64, len: usize) -> Vec<u8> {
    if buf.is_null() {
        return Vec::new();
    }
    let b = &*buf;
    if b.mem.is_null() {
        return Vec::new();
    }
    let m = &*b.mem;
    let start = (b.offset + offset) as usize;
    let end = (start + len).min(m.bytes.len());
    if start >= m.bytes.len() {
        return Vec::new();
    }
    m.bytes[start..end].to_vec()
}

unsafe fn write_buffer(buf: *mut Buffer, offset: u64, data: &[u8]) {
    if buf.is_null() {
        return;
    }
    let b = &mut *buf;
    if b.mem.is_null() {
        return;
    }
    let m = &mut *b.mem;
    let start = (b.offset + offset) as usize;
    let n = data.len().min(m.bytes.len().saturating_sub(start));
    m.bytes[start..start + n].copy_from_slice(&data[..n]);
    b.cpu_newer = true;
}

unsafe fn copy_b2i(src: *mut Buffer, dst: *mut Image, src_off: u64, w: u32, h: u32) {
    if dst.is_null() {
        return;
    }
    let img = &mut *dst;
    let bpp = format_bpp(img.format).max(4) as usize;
    let n = (w as usize) * (h as usize) * bpp;
    let bytes = read_buffer(src, src_off, n);
    let m = bytes.len().min(img.store.len());
    img.store[..m].copy_from_slice(&bytes[..m]);
    img.defined = true;
}

unsafe fn copy_i2b(src: *mut Image, dst: *mut Buffer, dst_off: u64, w: u32, h: u32) {
    if src.is_null() {
        return;
    }
    let img = &*src;
    let bpp = format_bpp(img.format).max(4) as usize;
    let n = ((w as usize) * (h as usize) * bpp).min(img.store.len());
    write_buffer(dst, dst_off, &img.store[..n]);
}

fn shader_key(a: &str, b: &str) -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    a.hash(&mut h);
    b.hash(&mut h);
    h.finish() as usize
}

fn image_rgba(img: &Image) -> Vec<u8> {
    let bpp = format_bpp(img.format).max(4) as usize;
    let pixels = (img.width * img.height) as usize;
    let mut out = Vec::with_capacity(pixels * 4);
    for i in 0..pixels {
        let s = i * bpp;
        if s + 4 > img.store.len() {
            break;
        }
        if img.format == FORMAT_B8G8R8A8_UNORM {
            out.extend_from_slice(&[img.store[s + 2], img.store[s + 1], img.store[s], img.store[s + 3]]);
        } else {
            out.extend_from_slice(&img.store[s..s + 4]);
        }
    }
    out
}

fn store_rgba8(img: &mut Image, rgba: &[u8]) {
    let bpp = format_bpp(img.format).max(4) as usize;
    let pixels = (img.width * img.height) as usize;
    if img.store.len() < pixels * bpp {
        img.store.resize(pixels * bpp, 0);
    }
    for i in 0..pixels {
        let s = i * 4;
        if s + 4 > rgba.len() {
            break;
        }
        let d = i * bpp;
        if img.format == FORMAT_B8G8R8A8_UNORM {
            img.store[d] = rgba[s + 2];
            img.store[d + 1] = rgba[s + 1];
            img.store[d + 2] = rgba[s];
            img.store[d + 3] = rgba[s + 3];
        } else if d + 4 <= img.store.len() {
            img.store[d..d + 4].copy_from_slice(&rgba[s..s + 4]);
        }
    }
    img.defined = true;
}

unsafe fn slot_bytes(set: *mut DescriptorSet, kind: i32) -> Option<(*mut Buffer, u64, Vec<u8>)> {
    if set.is_null() {
        return None;
    }
    let set = &*set;
    let slot = set.slots.iter().find(|s| s.kind == kind && !s.buffer.is_null())?;
    let range = if slot.range == WHOLE_SIZE {
        (*slot.buffer).size.saturating_sub(slot.offset)
    } else {
        slot.range
    };
    let bytes = read_buffer(slot.buffer, slot.offset, range as usize);
    Some((slot.buffer, slot.offset, bytes))
}

unsafe fn slot_image(set: *mut DescriptorSet) -> Option<(*mut Image, bool, bool)> {
    if set.is_null() {
        return None;
    }
    let set = &*set;
    let slot = set.slots.iter().find(|s| s.kind == DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER && !s.image.is_null())?;
    Some((slot.image, slot.nearest, slot.clamp))
}

unsafe fn exec_draw(dev: *mut Device, snap: DrawSnap) {
    if snap.pipe.is_null() || snap.image.is_null() {
        return;
    }
    let dev = &mut *dev;
    let pipe = &mut *snap.pipe;
    let image = &mut *snap.image;
    let stride = pipe.stride.max(1);
    let verts = if snap.vb.is_null() {
        Vec::new()
    } else {
        read_buffer(snap.vb, snap.vb_offset, (*snap.vb).size as usize)
    };
    let mut vp = snap.vp;
    if vp.w == 0.0 || vp.h == 0.0 {
        vp.w = image.width as f32;
        vp.h = image.height as f32;
    }
    let mut sc = snap.sc;
    if sc.w == 0 || sc.h == 0 {
        sc = Rect { x: 0, y: 0, w: image.width as i32, h: image.height as i32 };
    }
    let ubo = slot_bytes(snap.set, DESCRIPTOR_TYPE_UNIFORM_BUFFER).map(|(_, _, b)| b);
    let tex_info = slot_image(snap.set);
    let mut tex_owned: Option<(u32, u32, Vec<u8>, bool, bool)> = None;
    if let Some((img, nearest, clamp)) = tex_info {
        let image_ref = &*img;
        tex_owned = Some((image_ref.width, image_ref.height, image_rgba(image_ref), nearest, clamp));
    }

    let fan_ok = pipe.topology != PRIMITIVE_TOPOLOGY_TRIANGLE_FAN
        || dev.backend.as_ref().map(|b| b.tier().fan).unwrap_or(false);
    let inst_ok = snap.instance_count <= 1 || dev.backend.as_ref().map(|b| b.tier().instancing).unwrap_or(false);
    let fill = pipe.polygon == POLYGON_MODE_FILL;
    if pipe.hw && dev.backend.is_some() && !dev.force_sw && fan_ok && inst_ok && fill {
        let emitted = pipe.emitted.clone();
        if let Some(emitted) = emitted {
            let start = (snap.first_vertex * stride) as usize;
            let need = (snap.vertex_count * stride) as usize;
            let slice = if start < verts.len() {
                let end = (start + need).min(verts.len());
                &verts[start..end]
            } else {
                &verts[..]
            };
            let tex = tex_owned.as_ref().map(|(w, h, rgba, _, _)| Tex { w: *w, h: *h, rgba });
            let desc = DrawDesc {
                key: shader_key(&emitted.vs, &emitted.fs),
                vs: &emitted.vs,
                fs: &emitted.fs,
                verts: slice,
                stride,
                count: snap.vertex_count,
                clear: snap.clear,
                width: image.width,
                height: image.height,
                ubo: ubo.as_deref(),
                ubo_slot: emitted.ubo_slot,
                tex,
                components: pipe.components,
            };
            match dev.backend.as_mut().unwrap().draw(&desc) {
                Ok(px) => {
                    store_rgba8(image, &px);
                    return;
                }
                Err(e) => {
                    eprintln!("vkcompat: hw draw failed: {e}");
                    pipe.hw = false;
                    pipe.note = note_from(format!("{e}\n{}\n{}", emitted.vs, emitted.fs));
                }
            }
        }
    } else if dev.backend.as_ref().map(|b| b.tier().fixed_function).unwrap_or(false)
        && !matches!(pipe.ff.kind, FfKind::None)
        && !dev.force_sw
        && snap.instance_count <= 1
        && fill
    {
        if let (Some(vs), Some(_fs)) = (pipe.vs.clone(), pipe.fs.clone()) {
            let (clips, uvs) = shade_clips(&vs, &verts, stride, snap.vertex_count, snap.first_vertex, &pipe.attrs, pipe.ff.uv_loc);
            let tex = tex_owned.as_ref().map(|(w, h, rgba, _, _)| Tex { w: *w, h: *h, rgba });
            let desc = FixedDesc {
                color: pipe.ff.color,
                textured: matches!(pipe.ff.kind, FfKind::Tex | FfKind::TexMul),
                clips: &clips,
                uvs: &uvs,
                clear: snap.clear,
                width: image.width,
                height: image.height,
                tex,
            };
            match dev.backend.as_mut().unwrap().draw_fixed(&desc) {
                Ok(px) => {
                    store_rgba8(image, &px);
                    return;
                }
                Err(e) => {
                    eprintln!("vkcompat: hw draw failed: {e}");
                    pipe.ff_hw = false;
                    pipe.note = note_from(e);
                }
            }
        }
    }
    if !pipe.sw {
        return;
    }
    if let (Some(vs), Some(fs)) = (pipe.vs.as_ref(), pipe.fs.as_ref()) {
        let ubo_tuple;
        let ubos: &[_] = if let Some(bytes) = ubo.as_ref() {
            ubo_tuple = [(0u32, 0u32, bytes.as_slice())];
            &ubo_tuple
        } else {
            &[]
        };
        let image_bind;
        let images: &[_] = if let Some((w, h, rgba, nearest, clamp)) = tex_owned.as_ref() {
            image_bind = [ImageBind {
                set: 0,
                binding: 0,
                w: *w,
                h: *h,
                rgba,
                nearest: *nearest,
                clamp: *clamp,
            }];
            &image_bind
        } else {
            &[]
        };
        let mut target = FragTarget {
            pixels: &mut image.store,
            width: image.width,
            height: image.height,
            format: image.format,
        };
        if let Err(e) = raster::draw_triangle(
            vs,
            fs,
            &verts,
            stride,
            snap.vertex_count,
            snap.first_vertex,
            &pipe.attrs,
            vp,
            sc,
            ubos,
            images,
            &mut target,
        ) {
            pipe.note = note_from(e);
        }
        image.defined = true;
    }
}

fn shade_clips(
    vs: &Module,
    verts: &[u8],
    stride: u32,
    count: u32,
    first: u32,
    attrs: &[(u32, u32, i32)],
    uv_loc: i32,
) -> (Vec<[f32; 4]>, Vec<[f32; 2]>) {
    let mut clips = Vec::new();
    let mut uvs = Vec::new();
    for i in 0..count {
        let index = first + i;
        let mut inputs = vec![[0.0, 0.0, 0.0, 1.0]; 8];
        for (loc, off, fmt) in attrs {
            if let Some(slot) = inputs.get_mut(*loc as usize) {
                *slot = fetch_attr(verts, stride, index, *off, *fmt);
            }
        }
        let mut io = Io {
            attrs: &inputs,
            frag_coord: [0.0; 4],
            vertex_index: index as i32,
            instance_index: 0,
            global_id: [0; 3],
            ubos: &[],
            images: &[],
            ssbo: &mut [],
            ssbo_set: 0,
            ssbo_binding: 0,
            position: [0.0; 4],
            outputs: Vec::new(),
        };
        if spirv::execute(vs, &mut io).is_err() {
            break;
        }
        clips.push(io.position);
        let uv = io
            .outputs
            .iter()
            .find(|(loc, _)| *loc as i32 == uv_loc)
            .map(|(_, v)| [v[0], v[1]])
            .unwrap_or([0.0, 0.0]);
        uvs.push(uv);
    }
    (clips, uvs)
}

fn fetch_attr(verts: &[u8], stride: u32, index: u32, offset: u32, fmt: i32) -> [f32; 4] {
    let base = index as usize * stride as usize + offset as usize;
    let mut o = [0.0, 0.0, 0.0, 1.0];
    let n = components_of(fmt) as usize;
    for i in 0..n {
        let s = base + i * 4;
        if s + 4 <= verts.len() {
            o[i] = f32::from_le_bytes(verts[s..s + 4].try_into().unwrap());
        }
    }
    o
}

unsafe fn exec_dispatch(dev: *mut Device, snap: DispatchSnap) {
    if snap.pipe.is_null() {
        return;
    }
    let dev = &mut *dev;
    let pipe = &mut *snap.pipe;
    let Some((buf, offset, mut bytes)) = slot_bytes(snap.set, DESCRIPTOR_TYPE_STORAGE_BUFFER) else {
        return;
    };
    if pipe.hw && dev.backend.is_some() && !dev.force_sw {
        if let Some(emitted) = pipe.emitted.clone() {
            let desc = gpu::DispatchDesc {
                key: shader_key(&emitted.cs, ""),
                cs: &emitted.cs,
                groups: snap.groups,
                ssbo: &bytes,
            };
            match dev.backend.as_mut().unwrap().dispatch(&desc) {
                Ok(out) => {
                    write_buffer(buf, offset, &out);
                    return;
                }
                Err(e) => {
                    eprintln!("vkcompat: hw draw failed: {e}");
                    pipe.hw = false;
                    pipe.note = note_from(format!("{e}\n{}", emitted.cs));
                }
            }
        }
    }
    if pipe.sw {
        if let Some(cs) = pipe.cs.as_ref() {
            if let Err(e) = raster::dispatch_compute(cs, snap.groups, &mut bytes, 0, 0) {
                pipe.note = note_from(e);
            } else {
                write_buffer(buf, offset, &bytes);
            }
        }
    }
}

pub unsafe fn create_surface(hwnd: *mut c_void) -> *mut Surface {
    leak(Surface { hwnd: hwnd as isize })
}
pub unsafe fn destroy_surface(surface: VkSurfaceKHR) {
    if !surface.is_null() {
        drop(Box::from_raw(surface as *mut Surface));
    }
}

pub unsafe fn create_swapchain(info: &VkSwapchainCreateInfoKHR) -> Result<*mut Swapchain, VkResult> {
    if info.surface.is_null() {
        return Err(ERROR_INITIALIZATION_FAILED);
    }
    let surface = &*(info.surface as *mut Surface);
    let count = info.minImageCount.clamp(1, 8);
    let mut images = Vec::new();
    let w = info.imageExtent.width.max(1);
    let h = info.imageExtent.height.max(1);
    let bpp = format_bpp(info.imageFormat).max(4) as usize;
    for _ in 0..count {
        images.push(Box::new(Image {
            width: w,
            height: h,
            format: info.imageFormat,
            store: vec![0u8; (w as usize) * (h as usize) * bpp],
            defined: false,
        }));
    }
    Ok(leak(Swapchain {
        images,
        hwnd: surface.hwnd,
        acquired: 0,
    }))
}

pub unsafe fn destroy_swapchain(swap: VkSwapchainKHR) {
    if !swap.is_null() {
        drop(Box::from_raw(swap as *mut Swapchain));
    }
}

pub unsafe fn swapchain_images(swap: VkSwapchainKHR, count: *mut u32, images: *mut VkImage) -> VkResult {
    if swap.is_null() || count.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let s = &*(swap as *mut Swapchain);
    let n = s.images.len() as u32;
    if images.is_null() {
        *count = n;
        return SUCCESS;
    }
    let room = *count;
    let write = room.min(n) as usize;
    for i in 0..write {
        *images.add(i) = (&*s.images[i]) as *const Image as VkImage;
    }
    *count = write as u32;
    if room < n { INCOMPLETE } else { SUCCESS }
}

pub unsafe fn acquire(swap: VkSwapchainKHR, semaphore: VkSemaphore, index: *mut u32) -> VkResult {
    if swap.is_null() || index.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let s = &mut *(swap as *mut Swapchain);
    if s.images.is_empty() {
        return ERROR_INITIALIZATION_FAILED;
    }
    *index = s.acquired % s.images.len() as u32;
    s.acquired = s.acquired.wrapping_add(1);
    if !semaphore.is_null() {
        (*(semaphore as *mut Semaphore)).signaled = true;
    }
    SUCCESS
}

pub unsafe fn present(info: &VkPresentInfoKHR) -> VkResult {
    if info.swapchainCount == 0 || info.pSwapchains.is_null() || info.pImageIndices.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let swap = *info.pSwapchains as *mut Swapchain;
    if swap.is_null() {
        return ERROR_INITIALIZATION_FAILED;
    }
    let index = *info.pImageIndices as usize;
    let s = &*swap;
    if let Some(img) = s.images.get(index) {
        gpu::present_to_window(s.hwnd, &img.store, img.width, img.height, img.format);
    }
    SUCCESS
}

pub fn create_semaphore() -> *mut Semaphore {
    leak(Semaphore { signaled: false })
}
pub unsafe fn destroy_semaphore(sem: VkSemaphore) {
    if !sem.is_null() {
        drop(Box::from_raw(sem as *mut Semaphore));
    }
}

pub fn dev_name(p: &PhysDevice) -> String {
    unsafe { CStr::from_ptr(p.props.deviceName.as_ptr()).to_string_lossy().into_owned() }
}

pub fn fill_props(p: &PhysDevice, out: *mut VkPhysicalDeviceProperties) {
    if !out.is_null() {
        unsafe { *out = p.props; }
    }
}
pub fn fill_mem(p: &PhysDevice, out: *mut VkPhysicalDeviceMemoryProperties) {
    if !out.is_null() {
        unsafe { *out = p.memory; }
    }
}
pub fn queue_props(p: &PhysDevice) -> VkQueueFamilyProperties {
    p.queue_props
}

impl PhysDevice {
    pub fn props_copy(&self) -> VkPhysicalDeviceProperties {
        self.props
    }
    pub fn mem_copy(&self) -> VkPhysicalDeviceMemoryProperties {
        self.memory
    }
}

impl Buffer {
    pub fn size_of(&self) -> u64 {
        self.size
    }
}
