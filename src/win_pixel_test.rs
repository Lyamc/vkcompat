use std::ffi::{c_void, CString};
use std::ptr;
use vkcompat::entry::*;
use vkcompat::vk::*;
use windows::Win32::Foundation::{FreeLibrary, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WINDOW_EX_STYLE, WNDCLASSW, WS_POPUP,
};
use windows::core::PCWSTR;

static mut FAILS: i32 = 0;

fn fail(msg: &str) {
    eprintln!("FAIL {msg}");
    unsafe { FAILS += 1; }
}

fn vk_ok(r: VkResult, what: &str) -> bool {
    if r == SUCCESS {
        true
    } else {
        eprintln!("FAIL {what} ({r})");
        unsafe { FAILS += 1; }
        false
    }
}

fn near(a: u8, b: u8) -> bool {
    (a as i32 - b as i32).abs() <= 1
}

fn show_note(pipe: VkPipeline) {
    let n = unsafe { gfx_pipeline_note(pipe) };
    if n.is_null() {
        return;
    }
    let s = unsafe { std::ffi::CStr::from_ptr(n) }.to_string_lossy();
    let s = if s.len() > 2000 { &s[..2000] } else { &s };
    if !s.is_empty() {
        eprintln!("  pipeline note:\n{s}");
    }
}

struct Asm {
    w: Vec<u32>,
    next: u32,
}
impl Asm {
    fn new() -> Self { Self { w: Vec::new(), next: 1 } }
    fn id(&mut self) -> u32 { let i = self.next; self.next += 1; i }
    fn op(&mut self, opcode: u16, ops: &[u32]) {
        self.w.push(((1 + ops.len() as u32) << 16) | opcode as u32);
        self.w.extend_from_slice(ops);
    }
    fn lit(s: &str) -> Vec<u32> {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0);
        while bytes.len() % 4 != 0 { bytes.push(0); }
        bytes.chunks(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
    }
    fn entry(&mut self, model: u32, func: u32, iface: &[u32]) {
        let mut ops = vec![model, func];
        ops.extend(Self::lit("main"));
        ops.extend_from_slice(iface);
        self.op(15, &ops);
    }
    fn finish(self) -> Vec<u32> {
        let mut m = vec![0x0723_0203, 0x0001_0000, 0, self.next, 0];
        m.extend(self.w);
        m
    }
}
fn header(a: &mut Asm) {
    a.op(17, &[1]);
    a.op(14, &[0, 0]);
}
fn fbits(f: f32) -> u32 { f.to_bits() }

fn asm_vs_pos() -> Vec<u32> {
    let mut a = Asm::new();
    let fn_ = a.id(); let vin = a.id(); let vout = a.id();
    header(&mut a);
    a.entry(0, fn_, &[vin, vout]);
    a.op(71, &[vin, 30, 0]);
    a.op(71, &[vout, 11, 0]);
    let tvoid = a.id(); let tfloat = a.id(); let v2 = a.id(); let v4 = a.id();
    let p_in = a.id(); let p_out = a.id(); let tfn = a.id(); let c0 = a.id(); let c1 = a.id();
    a.op(19, &[tvoid]); a.op(22, &[tfloat, 32]); a.op(23, &[v2, tfloat, 2]); a.op(23, &[v4, tfloat, 4]);
    a.op(32, &[p_in, 1, v2]); a.op(32, &[p_out, 3, v4]); a.op(33, &[tfn, tvoid]);
    a.op(43, &[tfloat, c0, fbits(0.0)]); a.op(43, &[tfloat, c1, fbits(1.0)]);
    a.op(59, &[p_in, vin, 1]); a.op(59, &[p_out, vout, 3]);
    let entry = a.id(); let loaded = a.id(); let pos = a.id();
    a.op(54, &[tvoid, fn_, 0, tfn]); a.op(248, &[entry]);
    a.op(61, &[v2, loaded, vin]); a.op(80, &[v4, pos, loaded, c0, c1]); a.op(62, &[vout, pos]);
    a.op(253, &[]); a.op(56, &[]);
    a.finish()
}
fn asm_vs_uv() -> Vec<u32> {
    let mut a = Asm::new();
    let fn_ = a.id(); let vin = a.id(); let vpos = a.id(); let vuv = a.id();
    header(&mut a);
    a.entry(0, fn_, &[vin, vpos, vuv]);
    a.op(71, &[vin, 30, 0]); a.op(71, &[vpos, 11, 0]); a.op(71, &[vuv, 30, 0]);
    let tvoid = a.id(); let tfloat = a.id(); let v2 = a.id(); let v4 = a.id();
    let p_in = a.id(); let p_out = a.id(); let tfn = a.id();
    let c0 = a.id(); let c1 = a.id(); let c03 = a.id(); let uv = a.id();
    a.op(19, &[tvoid]); a.op(22, &[tfloat, 32]); a.op(23, &[v2, tfloat, 2]); a.op(23, &[v4, tfloat, 4]);
    a.op(32, &[p_in, 1, v2]); a.op(32, &[p_out, 3, v4]); a.op(33, &[tfn, tvoid]);
    a.op(43, &[tfloat, c0, fbits(0.0)]); a.op(43, &[tfloat, c1, fbits(1.0)]);
    a.op(43, &[tfloat, c03, fbits(0.3)]); a.op(44, &[v2, uv, c03, c03]);
    a.op(59, &[p_in, vin, 1]); a.op(59, &[p_out, vpos, 3]); a.op(59, &[p_out, vuv, 3]);
    let entry = a.id(); let loaded = a.id(); let pos = a.id(); let uv4 = a.id();
    a.op(54, &[tvoid, fn_, 0, tfn]); a.op(248, &[entry]);
    a.op(61, &[v2, loaded, vin]); a.op(80, &[v4, pos, loaded, c0, c1]); a.op(62, &[vpos, pos]);
    a.op(80, &[v4, uv4, uv, c0, c1]); a.op(62, &[vuv, uv4]);
    a.op(253, &[]); a.op(56, &[]);
    a.finish()
}
fn asm_fs_const(r: f32, g: f32, b: f32, a: f32) -> Vec<u32> {
    let mut s = Asm::new();
    let fn_ = s.id(); let cout = s.id();
    header(&mut s);
    s.entry(4, fn_, &[cout]);
    s.op(16, &[fn_, 7]);
    s.op(71, &[cout, 30, 0]);
    let tvoid = s.id(); let tfloat = s.id(); let v4 = s.id(); let p_out = s.id(); let tfn = s.id();
    let cr = s.id(); let cg = s.id(); let cb = s.id(); let ca = s.id(); let col = s.id();
    s.op(19, &[tvoid]); s.op(22, &[tfloat, 32]); s.op(23, &[v4, tfloat, 4]);
    s.op(32, &[p_out, 3, v4]); s.op(33, &[tfn, tvoid]);
    s.op(43, &[tfloat, cr, fbits(r)]); s.op(43, &[tfloat, cg, fbits(g)]);
    s.op(43, &[tfloat, cb, fbits(b)]); s.op(43, &[tfloat, ca, fbits(a)]);
    s.op(44, &[v4, col, cr, cg, cb, ca]);
    s.op(59, &[p_out, cout, 3]);
    let entry = s.id();
    s.op(54, &[tvoid, fn_, 0, tfn]); s.op(248, &[entry]);
    s.op(62, &[cout, col]); s.op(253, &[]); s.op(56, &[]);
    s.finish()
}
fn asm_fs_ubo() -> Vec<u32> {
    let mut a = Asm::new();
    let fn_ = a.id(); let cout = a.id(); let ubo = a.id();
    header(&mut a);
    a.entry(4, fn_, &[cout, ubo]);
    a.op(16, &[fn_, 7]);
    a.op(71, &[cout, 30, 0]);
    let tvoid = a.id(); let tfloat = a.id(); let v4 = a.id(); let tint = a.id(); let blk = a.id();
    let p_out = a.id(); let p_uni = a.id(); let p_mem = a.id(); let tfn = a.id(); let i0 = a.id();
    a.op(71, &[blk, 2]);
    a.op(72, &[blk, 0, 35, 0]);
    a.op(71, &[ubo, 34, 0]);
    a.op(71, &[ubo, 33, 0]);
    a.op(19, &[tvoid]); a.op(22, &[tfloat, 32]); a.op(23, &[v4, tfloat, 4]);
    a.op(21, &[tint, 32, 1]); a.op(30, &[blk, v4]);
    a.op(32, &[p_out, 3, v4]); a.op(32, &[p_uni, 2, blk]); a.op(32, &[p_mem, 2, v4]);
    a.op(33, &[tfn, tvoid]); a.op(43, &[tint, i0, 0]);
    a.op(59, &[p_out, cout, 3]); a.op(59, &[p_uni, ubo, 2]);
    let entry = a.id(); let chain = a.id(); let col = a.id();
    a.op(54, &[tvoid, fn_, 0, tfn]); a.op(248, &[entry]);
    a.op(65, &[p_mem, chain, ubo, i0]); a.op(61, &[v4, col, chain]); a.op(62, &[cout, col]);
    a.op(253, &[]); a.op(56, &[]);
    a.finish()
}
fn asm_fs_tex() -> Vec<u32> {
    let mut a = Asm::new();
    let fn_ = a.id(); let uv_in = a.id(); let tex = a.id(); let cout = a.id();
    header(&mut a);
    a.entry(4, fn_, &[uv_in, tex, cout]);
    a.op(16, &[fn_, 7]);
    a.op(71, &[uv_in, 30, 0]); a.op(71, &[tex, 34, 0]); a.op(71, &[tex, 33, 0]); a.op(71, &[cout, 30, 0]);
    let tvoid = a.id(); let tfloat = a.id(); let v4 = a.id(); let img = a.id(); let simg = a.id();
    let p_in = a.id(); let p_uni = a.id(); let p_out = a.id(); let tfn = a.id();
    a.op(19, &[tvoid]); a.op(22, &[tfloat, 32]); a.op(23, &[v4, tfloat, 4]);
    a.op(25, &[img, tfloat, 1, 0, 0, 0, 1, 0]);
    a.op(27, &[simg, img]);
    a.op(32, &[p_in, 1, v4]); a.op(32, &[p_uni, 0, simg]); a.op(32, &[p_out, 3, v4]);
    a.op(33, &[tfn, tvoid]);
    a.op(59, &[p_in, uv_in, 1]); a.op(59, &[p_uni, tex, 0]); a.op(59, &[p_out, cout, 3]);
    let entry = a.id(); let uv = a.id(); let handle = a.id(); let sampled = a.id();
    a.op(54, &[tvoid, fn_, 0, tfn]); a.op(248, &[entry]);
    a.op(61, &[v4, uv, uv_in]); a.op(61, &[simg, handle, tex]); a.op(87, &[v4, sampled, handle, uv]);
    a.op(62, &[cout, sampled]); a.op(253, &[]); a.op(56, &[]);
    a.finish()
}
fn asm_cs_red() -> Vec<u32> {
    let mut a = Asm::new();
    let fn_ = a.id(); let gid = a.id(); let ssbo = a.id();
    header(&mut a);
    a.entry(5, fn_, &[gid, ssbo]);
    a.op(16, &[fn_, 17, 1, 1, 1]);
    a.op(71, &[gid, 11, 28]);
    let tvoid = a.id(); let tuint = a.id(); let uv3 = a.id(); let tfloat = a.id(); let v4 = a.id();
    let rt = a.id(); let blk = a.id(); let p_gid = a.id(); let p_ssbo = a.id(); let p_elt = a.id();
    let tfn = a.id(); let u0 = a.id(); let f1 = a.id(); let f0 = a.id(); let red = a.id();
    a.op(71, &[rt, 6, 16]); a.op(71, &[blk, 2]); a.op(72, &[blk, 0, 35, 0]);
    a.op(71, &[ssbo, 34, 0]); a.op(71, &[ssbo, 33, 0]);
    a.op(19, &[tvoid]); a.op(21, &[tuint, 32, 0]); a.op(23, &[uv3, tuint, 3]);
    a.op(22, &[tfloat, 32]); a.op(23, &[v4, tfloat, 4]);
    a.op(29, &[rt, v4]); a.op(30, &[blk, rt]);
    a.op(32, &[p_gid, 1, uv3]); a.op(32, &[p_ssbo, 12, blk]); a.op(32, &[p_elt, 12, v4]);
    a.op(33, &[tfn, tvoid]);
    a.op(43, &[tuint, u0, 0]); a.op(43, &[tfloat, f1, fbits(1.0)]); a.op(43, &[tfloat, f0, fbits(0.0)]);
    a.op(44, &[v4, red, f1, f0, f0, f1]);
    a.op(59, &[p_gid, gid, 1]); a.op(59, &[p_ssbo, ssbo, 12]);
    let entry = a.id(); let g = a.id(); let x = a.id(); let ptr_ = a.id();
    a.op(54, &[tvoid, fn_, 0, tfn]); a.op(248, &[entry]);
    a.op(61, &[uv3, g, gid]); a.op(81, &[tuint, x, g, 0]); a.op(65, &[p_elt, ptr_, ssbo, u0, x]);
    a.op(62, &[ptr_, red]); a.op(253, &[]); a.op(56, &[]);
    a.finish()
}
fn asm_fs_loop() -> Vec<u32> {
    let mut a = Asm::new();
    let fn_ = a.id(); let cout = a.id();
    header(&mut a);
    a.entry(4, fn_, &[cout]);
    a.op(16, &[fn_, 7]);
    a.op(71, &[cout, 30, 0]);
    let tvoid = a.id(); let tfloat = a.id(); let tint = a.id(); let tbool = a.id(); let v4 = a.id();
    let p_out = a.id(); let tfn = a.id();
    let i0 = a.id(); let i1 = a.id(); let i4 = a.id(); let f0 = a.id(); let f025 = a.id(); let f1 = a.id();
    a.op(19, &[tvoid]); a.op(22, &[tfloat, 32]); a.op(21, &[tint, 32, 1]); a.op(20, &[tbool]);
    a.op(23, &[v4, tfloat, 4]); a.op(32, &[p_out, 3, v4]); a.op(33, &[tfn, tvoid]);
    a.op(43, &[tint, i0, 0]); a.op(43, &[tint, i1, 1]); a.op(43, &[tint, i4, 4]);
    a.op(43, &[tfloat, f0, fbits(0.0)]); a.op(43, &[tfloat, f025, fbits(0.25)]); a.op(43, &[tfloat, f1, fbits(1.0)]);
    a.op(59, &[p_out, cout, 3]);
    let b_entry = a.id(); let b_loop = a.id(); let b_body = a.id(); let b_cont = a.id(); let b_merge = a.id();
    let phi_i = a.id(); let phi_s = a.id(); let cond = a.id(); let snext = a.id(); let inext = a.id(); let col = a.id();
    a.op(54, &[tvoid, fn_, 0, tfn]);
    a.op(248, &[b_entry]); a.op(249, &[b_loop]);
    a.op(248, &[b_loop]);
    a.op(245, &[tint, phi_i, i0, b_entry, inext, b_cont]);
    a.op(245, &[tfloat, phi_s, f0, b_entry, snext, b_cont]);
    a.op(246, &[b_merge, b_cont, 0]);
    a.op(177, &[tbool, cond, phi_i, i4]);
    a.op(250, &[cond, b_body, b_merge]);
    a.op(248, &[b_body]); a.op(249, &[b_cont]);
    a.op(248, &[b_cont]);
    a.op(129, &[tfloat, snext, phi_s, f025]);
    a.op(128, &[tint, inext, phi_i, i1]);
    a.op(249, &[b_loop]);
    a.op(248, &[b_merge]);
    a.op(80, &[v4, col, phi_s, f0, f0, f1]);
    a.op(62, &[cout, col]); a.op(253, &[]); a.op(56, &[]);
    a.finish()
}

struct Buf { dev: VkDevice, buffer: VkBuffer, mem: VkDeviceMemory }
impl Drop for Buf {
    fn drop(&mut self) {
        unsafe {
            if !self.buffer.is_null() { vkDestroyBuffer(self.dev, self.buffer, ptr::null()); }
            if !self.mem.is_null() { vkFreeMemory(self.dev, self.mem, ptr::null()); }
        }
    }
}
impl Buf {
    fn create(dev: VkDevice, bytes: u64, usage: u32) -> Option<Self> {
        unsafe {
            let mut bi: VkBufferCreateInfo = zeroed();
            bi.sType = STRUCTURE_TYPE_BUFFER_CREATE_INFO;
            bi.size = bytes;
            bi.usage = usage;
            bi.sharingMode = SHARING_MODE_EXCLUSIVE;
            let mut buffer = ptr::null_mut();
            if !vk_ok(vkCreateBuffer(dev, &bi, ptr::null(), &mut buffer), "vkCreateBuffer") { return None; }
            let mut req: VkMemoryRequirements = zeroed();
            vkGetBufferMemoryRequirements(dev, buffer, &mut req);
            let mut ai: VkMemoryAllocateInfo = zeroed();
            ai.sType = STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
            ai.allocationSize = req.size;
            ai.memoryTypeIndex = 0;
            let mut mem = ptr::null_mut();
            if !vk_ok(vkAllocateMemory(dev, &ai, ptr::null(), &mut mem), "vkAllocateMemory") { return None; }
            if !vk_ok(vkBindBufferMemory(dev, buffer, mem, 0), "vkBindBufferMemory") { return None; }
            Some(Buf { dev, buffer, mem })
        }
    }
    fn write(&self, data: &[u8]) -> bool {
        unsafe {
            let mut p = ptr::null_mut();
            if !vk_ok(vkMapMemory(self.dev, self.mem, 0, WHOLE_SIZE, 0, &mut p), "vkMapMemory") { return false; }
            ptr::copy_nonoverlapping(data.as_ptr(), p as *mut u8, data.len());
            vkUnmapMemory(self.dev, self.mem);
            true
        }
    }
    fn read(&self, data: &mut [u8]) -> bool {
        unsafe {
            let mut p = ptr::null_mut();
            if !vk_ok(vkMapMemory(self.dev, self.mem, 0, WHOLE_SIZE, 0, &mut p), "vkMapMemory") { return false; }
            ptr::copy_nonoverlapping(p as *const u8, data.as_mut_ptr(), data.len());
            vkUnmapMemory(self.dev, self.mem);
            true
        }
    }
}

struct Img { dev: VkDevice, image: VkImage, mem: VkDeviceMemory, view: VkImageView }
impl Drop for Img {
    fn drop(&mut self) {
        unsafe {
            if !self.view.is_null() { vkDestroyImageView(self.dev, self.view, ptr::null()); }
            if !self.image.is_null() { vkDestroyImage(self.dev, self.image, ptr::null()); }
            if !self.mem.is_null() { vkFreeMemory(self.dev, self.mem, ptr::null()); }
        }
    }
}
impl Img {
    fn create(dev: VkDevice, w: u32, h: u32, fmt: i32, usage: u32) -> Option<Self> {
        unsafe {
            let mut ii: VkImageCreateInfo = zeroed();
            ii.sType = STRUCTURE_TYPE_IMAGE_CREATE_INFO;
            ii.imageType = IMAGE_TYPE_2D;
            ii.format = fmt;
            ii.extent = VkExtent3D { width: w, height: h, depth: 1 };
            ii.mipLevels = 1;
            ii.arrayLayers = 1;
            ii.samples = SAMPLE_COUNT_1_BIT;
            ii.tiling = IMAGE_TILING_OPTIMAL;
            ii.usage = usage;
            ii.sharingMode = SHARING_MODE_EXCLUSIVE;
            let mut image = ptr::null_mut();
            if !vk_ok(vkCreateImage(dev, &ii, ptr::null(), &mut image), "vkCreateImage") { return None; }
            let mut req: VkMemoryRequirements = zeroed();
            vkGetImageMemoryRequirements(dev, image, &mut req);
            let mut ai: VkMemoryAllocateInfo = zeroed();
            ai.sType = STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
            ai.allocationSize = req.size;
            ai.memoryTypeIndex = 0;
            let mut mem = ptr::null_mut();
            if !vk_ok(vkAllocateMemory(dev, &ai, ptr::null(), &mut mem), "vkAllocateMemory(image)") { return None; }
            if !vk_ok(vkBindImageMemory(dev, image, mem, 0), "vkBindImageMemory") { return None; }
            let mut vi: VkImageViewCreateInfo = zeroed();
            vi.sType = STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
            vi.image = image;
            vi.viewType = IMAGE_VIEW_TYPE_2D;
            vi.format = fmt;
            vi.subresourceRange.aspectMask = IMAGE_ASPECT_COLOR_BIT;
            vi.subresourceRange.levelCount = 1;
            vi.subresourceRange.layerCount = 1;
            let mut view = ptr::null_mut();
            if !vk_ok(vkCreateImageView(dev, &vi, ptr::null(), &mut view), "vkCreateImageView") { return None; }
            Some(Img { dev, image, mem, view })
        }
    }
}

struct Dev {
    phys: VkPhysicalDevice,
    device: VkDevice,
    queue: VkQueue,
    pool: VkCommandPool,
    cmd: VkCommandBuffer,
    rp: VkRenderPass,
    name: String,
    want_color_hw: bool,
    want_ubo_hw: bool,
    want_tex_hw: bool,
    want_cs_hw: bool,
}
impl Drop for Dev {
    fn drop(&mut self) { self.close(); }
}
impl Dev {
    fn close(&mut self) {
        unsafe {
            if !self.device.is_null() && !self.rp.is_null() { vkDestroyRenderPass(self.device, self.rp, ptr::null()); }
            self.rp = ptr::null_mut();
            if !self.device.is_null() && !self.pool.is_null() { vkDestroyCommandPool(self.device, self.pool, ptr::null()); }
            self.pool = ptr::null_mut();
            self.cmd = ptr::null_mut();
            if !self.device.is_null() { vkDestroyDevice(self.device, ptr::null()); }
            self.device = ptr::null_mut();
        }
    }
}

fn dev_name(phys: VkPhysicalDevice) -> String {
    unsafe {
        let mut props: VkPhysicalDeviceProperties = zeroed();
        vkGetPhysicalDeviceProperties(phys, &mut props);
        let bytes = props.deviceName.iter().take_while(|c| **c != 0).map(|c| *c as u8).collect::<Vec<_>>();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

fn open_dev(phys: VkPhysicalDevice) -> Option<Dev> {
    let name = dev_name(phys);
    let cpu = name.contains("CPU");
    let d3d11 = name.contains("D3D11");
    let fl10 = name.contains("FL10");
    let d3d9 = name.contains("D3D9");
    let tier3 = name.contains("tier 3");
    let tier2 = name.contains("tier 2");
    let want_color_hw = !cpu;
    let want_ubo_hw = (d3d11 && !fl10) || d3d9 || tier3 || tier2;
    let want_tex_hw = want_ubo_hw;
    let want_cs_hw = d3d11 && !fl10;
    unsafe {
        let mut feats: VkPhysicalDeviceFeatures = zeroed();
        vkGetPhysicalDeviceFeatures(phys, &mut feats);
        let mut mem: VkPhysicalDeviceMemoryProperties = zeroed();
        vkGetPhysicalDeviceMemoryProperties(phys, &mut mem);
        if mem.memoryTypeCount < 1 || mem.memoryHeapCount < 1 {
            fail("memory properties");
            return None;
        }
        let mut qcount = 0u32;
        vkGetPhysicalDeviceQueueFamilyProperties(phys, &mut qcount, ptr::null_mut());
        if qcount < 1 { fail("no queue families"); return None; }
        let mut qprops: VkQueueFamilyProperties = zeroed();
        let mut one = 1u32;
        vkGetPhysicalDeviceQueueFamilyProperties(phys, &mut one, &mut qprops);
        let need = QUEUE_GRAPHICS_BIT | QUEUE_COMPUTE_BIT | QUEUE_TRANSFER_BIT;
        if qprops.queueFlags & need != need { fail("queue family flags"); return None; }
        let e0 = CString::new(EXT_SWAPCHAIN).unwrap();
        let e1 = CString::new(EXT_DYNAMIC_RENDERING).unwrap();
        let exts = [e0.as_ptr(), e1.as_ptr()];
        let prio = 1.0f32;
        let mut qi: VkDeviceQueueCreateInfo = zeroed();
        qi.sType = STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
        qi.queueCount = 1;
        qi.pQueuePriorities = &prio;
        let mut di: VkDeviceCreateInfo = zeroed();
        di.sType = STRUCTURE_TYPE_DEVICE_CREATE_INFO;
        di.queueCreateInfoCount = 1;
        di.pQueueCreateInfos = &qi;
        di.enabledExtensionCount = 2;
        di.ppEnabledExtensionNames = exts.as_ptr();
        let mut device = ptr::null_mut();
        let cr = vkCreateDevice(phys, &di, ptr::null(), &mut device);
        if cr != SUCCESS {
            println!("SKIP {name} (device create {cr})");
            if cpu { fail("CPU device failed to open"); }
            return None;
        }
        let mut queue = ptr::null_mut();
        vkGetDeviceQueue(device, 0, 0, &mut queue);
        let mut pi: VkCommandPoolCreateInfo = zeroed();
        pi.sType = STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
        let mut pool = ptr::null_mut();
        if !vk_ok(vkCreateCommandPool(device, &pi, ptr::null(), &mut pool), "vkCreateCommandPool") { return None; }
        let mut ai: VkCommandBufferAllocateInfo = zeroed();
        ai.sType = STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
        ai.commandPool = pool;
        ai.level = COMMAND_BUFFER_LEVEL_PRIMARY;
        ai.commandBufferCount = 1;
        let mut cmd = ptr::null_mut();
        if !vk_ok(vkAllocateCommandBuffers(device, &ai, &mut cmd), "vkAllocateCommandBuffers") { return None; }
        let mut ri: VkRenderPassCreateInfo = zeroed();
        ri.sType = STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO;
        ri.attachmentCount = 1;
        ri.subpassCount = 1;
        let mut rp = ptr::null_mut();
        if !vk_ok(vkCreateRenderPass(device, &ri, ptr::null(), &mut rp), "vkCreateRenderPass") { return None; }
        Some(Dev { phys, device, queue, pool, cmd, rp, name, want_color_hw, want_ubo_hw, want_tex_hw, want_cs_hw })
    }
}

fn begin_cmd(d: &Dev) -> bool {
    unsafe {
        let mut bi: VkCommandBufferBeginInfo = zeroed();
        bi.sType = STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
        vk_ok(vkBeginCommandBuffer(d.cmd, &bi), "vkBeginCommandBuffer")
    }
}
fn submit(d: &Dev) -> bool {
    unsafe {
        if !vk_ok(vkEndCommandBuffer(d.cmd), "vkEndCommandBuffer") { return false; }
        let mut si: VkSubmitInfo = zeroed();
        si.sType = STRUCTURE_TYPE_SUBMIT_INFO;
        si.commandBufferCount = 1;
        si.pCommandBuffers = &d.cmd;
        vk_ok(vkQueueSubmit(d.queue, 1, &si, ptr::null_mut()), "vkQueueSubmit")
    }
}
fn make_shader(device: VkDevice, words: &[u32]) -> VkShaderModule {
    unsafe {
        let mut si: VkShaderModuleCreateInfo = zeroed();
        si.sType = STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
        si.codeSize = words.len() * 4;
        si.pCode = words.as_ptr();
        let mut out = ptr::null_mut();
        if !vk_ok(vkCreateShaderModule(device, &si, ptr::null(), &mut out), "vkCreateShaderModule") {
            return ptr::null_mut();
        }
        out
    }
}
fn make_graphics(d: &Dev, vs_words: &[u32], fs_words: &[u32], layout: VkPipelineLayout) -> VkPipeline {
    unsafe {
        let vs = make_shader(d.device, vs_words);
        let fs = make_shader(d.device, fs_words);
        if vs.is_null() || fs.is_null() { return ptr::null_mut(); }
        let name = CString::new("main").unwrap();
        let mut stages: [VkPipelineShaderStageCreateInfo; 2] = [zeroed(), zeroed()];
        stages[0].sType = STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
        stages[0].stage = SHADER_STAGE_VERTEX_BIT;
        stages[0].module = vs;
        stages[0].pName = name.as_ptr();
        stages[1].sType = STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
        stages[1].stage = SHADER_STAGE_FRAGMENT_BIT;
        stages[1].module = fs;
        stages[1].pName = name.as_ptr();
        let bind = VkVertexInputBindingDescription { binding: 0, stride: 8, inputRate: 0 };
        let attr = VkVertexInputAttributeDescription { location: 0, binding: 0, format: FORMAT_R32G32_SFLOAT, offset: 0 };
        let mut vi: VkPipelineVertexInputStateCreateInfo = zeroed();
        vi.sType = STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
        vi.vertexBindingDescriptionCount = 1;
        vi.pVertexBindingDescriptions = &bind;
        vi.vertexAttributeDescriptionCount = 1;
        vi.pVertexAttributeDescriptions = &attr;
        let mut ia: VkPipelineInputAssemblyStateCreateInfo = zeroed();
        ia.sType = STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
        ia.topology = PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;
        let mut rs: VkPipelineRasterizationStateCreateInfo = zeroed();
        rs.sType = STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
        rs.polygonMode = POLYGON_MODE_FILL;
        rs.cullMode = CULL_MODE_NONE;
        rs.frontFace = FRONT_FACE_COUNTER_CLOCKWISE;
        rs.lineWidth = 1.0;
        let mut gi: VkGraphicsPipelineCreateInfo = zeroed();
        gi.sType = STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
        gi.stageCount = 2;
        gi.pStages = stages.as_ptr();
        gi.pVertexInputState = &vi;
        gi.pInputAssemblyState = &ia;
        gi.pRasterizationState = &rs;
        gi.layout = layout;
        gi.renderPass = d.rp;
        let mut out = ptr::null_mut();
        let ok = vk_ok(vkCreateGraphicsPipelines(d.device, ptr::null_mut(), 1, &gi, ptr::null(), &mut out), "vkCreateGraphicsPipelines");
        vkDestroyShaderModule(d.device, vs, ptr::null());
        vkDestroyShaderModule(d.device, fs, ptr::null());
        if ok { out } else { ptr::null_mut() }
    }
}
fn empty_layout(device: VkDevice) -> VkPipelineLayout {
    unsafe {
        let mut li: VkPipelineLayoutCreateInfo = zeroed();
        li.sType = STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
        let mut layout = ptr::null_mut();
        if vk_ok(vkCreatePipelineLayout(device, &li, ptr::null(), &mut layout), "vkCreatePipelineLayout") { layout } else { ptr::null_mut() }
    }
}

struct Frame { color: Img, fb: VkFramebuffer, readback: Buf, dev: VkDevice }
impl Drop for Frame {
    fn drop(&mut self) {
        unsafe { if !self.fb.is_null() { vkDestroyFramebuffer(self.dev, self.fb, ptr::null()); } }
    }
}
impl Frame {
    fn create(d: &Dev) -> Option<Self> {
        let color = Img::create(d.device, 64, 64, FORMAT_R8G8B8A8_UNORM, IMAGE_USAGE_COLOR_ATTACHMENT_BIT | IMAGE_USAGE_TRANSFER_SRC_BIT)?;
        let readback = Buf::create(d.device, 64 * 64 * 4, BUFFER_USAGE_TRANSFER_DST_BIT)?;
        unsafe {
            let mut fi: VkFramebufferCreateInfo = zeroed();
            fi.sType = STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
            fi.renderPass = d.rp;
            fi.attachmentCount = 1;
            fi.pAttachments = &color.view;
            fi.width = 64;
            fi.height = 64;
            fi.layers = 1;
            let mut fb = ptr::null_mut();
            if !vk_ok(vkCreateFramebuffer(d.device, &fi, ptr::null(), &mut fb), "vkCreateFramebuffer") { return None; }
            Some(Frame { color, fb, readback, dev: d.device })
        }
    }
}

fn bind_draw_state(d: &Dev, pipe: VkPipeline, vb: VkBuffer, layout: VkPipelineLayout, set: VkDescriptorSet) {
    unsafe {
        vkCmdBindPipeline(d.cmd, PIPELINE_BIND_POINT_GRAPHICS, pipe);
        let off = 0u64;
        vkCmdBindVertexBuffers(d.cmd, 0, 1, &vb, &off);
        if !set.is_null() {
            vkCmdBindDescriptorSets(d.cmd, PIPELINE_BIND_POINT_GRAPHICS, layout, 0, 1, &set, 0, ptr::null());
        }
        let mut vp: VkViewport = zeroed();
        vp.width = 64.0;
        vp.height = 64.0;
        vp.maxDepth = 1.0;
        vkCmdSetViewport(d.cmd, 0, 1, &vp);
        let mut sc: VkRect2D = zeroed();
        sc.extent = VkExtent2D { width: 64, height: 64 };
        vkCmdSetScissor(d.cmd, 0, 1, &sc);
    }
}

fn draw_and_read(d: &Dev, frame: &Frame, pipe: VkPipeline, vb: VkBuffer, layout: VkPipelineLayout, set: VkDescriptorSet, er: u8, eg: u8, eb: u8, label: &str, want_hw: bool) -> bool {
    if !begin_cmd(d) { return false; }
    bind_draw_state(d, pipe, vb, layout, set);
    unsafe {
        let mut clear: VkClearValue = zeroed();
        clear.float32 = [0.0, 0.0, 1.0, 1.0];
        let mut rp: VkRenderPassBeginInfo = zeroed();
        rp.sType = STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
        rp.renderPass = d.rp;
        rp.framebuffer = frame.fb;
        rp.renderArea.extent = VkExtent2D { width: 64, height: 64 };
        rp.clearValueCount = 1;
        rp.pClearValues = &clear;
        vkCmdBeginRenderPass(d.cmd, &rp, SUBPASS_CONTENTS_INLINE);
        vkCmdDraw(d.cmd, 3, 1, 0, 0);
        vkCmdEndRenderPass(d.cmd);
        vkCmdPipelineBarrier(d.cmd, 0, 0, 0, 0, ptr::null(), 0, ptr::null(), 0, ptr::null());
        let mut region: VkBufferImageCopy = zeroed();
        region.imageSubresource.aspectMask = IMAGE_ASPECT_COLOR_BIT;
        region.imageSubresource.layerCount = 1;
        region.imageExtent = VkExtent3D { width: 64, height: 64, depth: 1 };
        vkCmdCopyImageToBuffer(d.cmd, frame.color.image, IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL, frame.readback.buffer, 1, &region);
    }
    if !submit(d) { return false; }
    let mut pix = vec![0u8; 64 * 64 * 4];
    if !frame.readback.read(&mut pix) { return false; }
    let c = &pix[(32 * 64 + 32) * 4..];
    let o = &pix[..];
    let center = near(c[0], er) && near(c[1], eg) && near(c[2], eb) && near(c[3], 255);
    let corner = near(o[0], 0) && near(o[1], 0) && near(o[2], 255) && near(o[3], 255);
    let hw = unsafe { gfx_pipeline_uses_hw(pipe) };
    let hw_ok = if want_hw { hw == 1 } else { true };
    if !center || !corner || !hw_ok {
        eprintln!("FAIL {} {label} center={},{},{},{} corner={},{},{},{} hw={hw} (want hw {})",
            d.name, c[0], c[1], c[2], c[3], o[0], o[1], o[2], o[3], if want_hw { 1 } else { 0 });
        show_note(pipe);
        unsafe { FAILS += 1; }
        false
    } else {
        println!("  {label} hw={hw} ok");
        true
    }
}

fn run_color(d: &Dev, frame: &Frame, vb: VkBuffer) -> bool {
    let layout = empty_layout(d.device);
    let vs = asm_vs_pos();
    let fs = asm_fs_const(1.0, 0.0, 0.0, 1.0);
    let pipe = make_graphics(d, &vs, &fs, layout);
    let mut ok = !pipe.is_null();
    if ok { ok = draw_and_read(d, frame, pipe, vb, layout, ptr::null_mut(), 255, 0, 0, "color", d.want_color_hw); }
    unsafe {
        if !pipe.is_null() { vkDestroyPipeline(d.device, pipe, ptr::null()); }
        vkDestroyPipelineLayout(d.device, layout, ptr::null());
    }
    ok
}
fn run_loop(d: &Dev, frame: &Frame, vb: VkBuffer) -> bool {
    let layout = empty_layout(d.device);
    let vs = asm_vs_pos();
    let fs = asm_fs_loop();
    let pipe = make_graphics(d, &vs, &fs, layout);
    let mut ok = !pipe.is_null();
    if ok {
        let hw = unsafe { gfx_pipeline_uses_hw(pipe) };
        if hw != 0 {
            eprintln!("FAIL {} loop ran on hardware", d.name);
            show_note(pipe);
            unsafe { FAILS += 1; }
            ok = false;
        } else {
            ok = draw_and_read(d, frame, pipe, vb, layout, ptr::null_mut(), 255, 0, 0, "loop", false);
        }
    }
    unsafe {
        if !pipe.is_null() { vkDestroyPipeline(d.device, pipe, ptr::null()); }
        vkDestroyPipelineLayout(d.device, layout, ptr::null());
    }
    ok
}

fn one_set(d: &Dev, ty: i32, stage: u32) -> Option<(VkDescriptorSetLayout, VkPipelineLayout, VkDescriptorPool, VkDescriptorSet)> {
    unsafe {
        let lb = VkDescriptorSetLayoutBinding { binding: 0, descriptorType: ty, descriptorCount: 1, stageFlags: stage, pImmutableSamplers: ptr::null() };
        let mut sl: VkDescriptorSetLayoutCreateInfo = zeroed();
        sl.sType = STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
        sl.bindingCount = 1;
        sl.pBindings = &lb;
        let mut set_layout = ptr::null_mut();
        if !vk_ok(vkCreateDescriptorSetLayout(d.device, &sl, ptr::null(), &mut set_layout), "set layout") { return None; }
        let mut li: VkPipelineLayoutCreateInfo = zeroed();
        li.sType = STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
        li.setLayoutCount = 1;
        li.pSetLayouts = &set_layout;
        let mut layout = ptr::null_mut();
        if !vk_ok(vkCreatePipelineLayout(d.device, &li, ptr::null(), &mut layout), "layout") { return None; }
        let mut pi: VkDescriptorPoolCreateInfo = zeroed();
        pi.sType = STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
        pi.maxSets = 1;
        let mut pool = ptr::null_mut();
        if !vk_ok(vkCreateDescriptorPool(d.device, &pi, ptr::null(), &mut pool), "pool") { return None; }
        let mut ai: VkDescriptorSetAllocateInfo = zeroed();
        ai.sType = STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
        ai.descriptorPool = pool;
        ai.descriptorSetCount = 1;
        ai.pSetLayouts = &set_layout;
        let mut set = ptr::null_mut();
        if !vk_ok(vkAllocateDescriptorSets(d.device, &ai, &mut set), "alloc set") { return None; }
        Some((set_layout, layout, pool, set))
    }
}

fn run_ubo(d: &Dev, frame: &Frame, vb: VkBuffer) -> bool {
    let (set_layout, layout, pool, set) = match one_set(d, DESCRIPTOR_TYPE_UNIFORM_BUFFER, SHADER_STAGE_FRAGMENT_BIT) {
        Some(v) => v,
        None => return false,
    };
    let green = [0f32, 1.0, 0.0, 1.0];
    let bytes = green.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<_>>();
    let ubo = match Buf::create(d.device, bytes.len() as u64, BUFFER_USAGE_UNIFORM_BUFFER_BIT) {
        Some(b) => b,
        None => return false,
    };
    if !ubo.write(&bytes) { return false; }
    unsafe {
        let info = VkDescriptorBufferInfo { buffer: ubo.buffer, offset: 0, range: bytes.len() as u64 };
        let mut wr: VkWriteDescriptorSet = zeroed();
        wr.sType = STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        wr.dstSet = set;
        wr.descriptorCount = 1;
        wr.descriptorType = DESCRIPTOR_TYPE_UNIFORM_BUFFER;
        wr.pBufferInfo = &info;
        vkUpdateDescriptorSets(d.device, 1, &wr, 0, ptr::null());
        let vs = asm_vs_pos();
        let fs = asm_fs_ubo();
        let pipe = make_graphics(d, &vs, &fs, layout);
        let mut ok = !pipe.is_null();
        if ok { ok = draw_and_read(d, frame, pipe, vb, layout, set, 0, 255, 0, "ubo", d.want_ubo_hw); }
        if !pipe.is_null() { vkDestroyPipeline(d.device, pipe, ptr::null()); }
        vkDestroyDescriptorPool(d.device, pool, ptr::null());
        vkDestroyPipelineLayout(d.device, layout, ptr::null());
        vkDestroyDescriptorSetLayout(d.device, set_layout, ptr::null());
        ok
    }
}

fn run_tex(d: &Dev, frame: &Frame, vb: VkBuffer) -> bool {
    let (set_layout, layout, pool, set) = match one_set(d, DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER, SHADER_STAGE_FRAGMENT_BIT) {
        Some(v) => v,
        None => return false,
    };
    let mut texels = vec![0u8, 0, 255, 255].repeat(16);
    let i = (1 * 4 + 1) * 4;
    texels[i] = 255; texels[i + 1] = 0; texels[i + 2] = 0; texels[i + 3] = 255;
    let stage = match Buf::create(d.device, texels.len() as u64, BUFFER_USAGE_TRANSFER_SRC_BIT) {
        Some(b) => b, None => return false,
    };
    if !stage.write(&texels) { return false; }
    let tex = match Img::create(d.device, 4, 4, FORMAT_R8G8B8A8_UNORM, IMAGE_USAGE_SAMPLED_BIT | IMAGE_USAGE_TRANSFER_DST_BIT) {
        Some(t) => t, None => return false,
    };
    unsafe {
        let mut sci: VkSamplerCreateInfo = zeroed();
        sci.sType = STRUCTURE_TYPE_SAMPLER_CREATE_INFO;
        sci.magFilter = FILTER_NEAREST;
        sci.minFilter = FILTER_NEAREST;
        sci.addressModeU = SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
        sci.addressModeV = SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
        sci.addressModeW = SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE;
        let mut sampler = ptr::null_mut();
        if !vk_ok(vkCreateSampler(d.device, &sci, ptr::null(), &mut sampler), "sampler") { return false; }
        let ii = VkDescriptorImageInfo { sampler, imageView: tex.view, imageLayout: IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL };
        let mut wr: VkWriteDescriptorSet = zeroed();
        wr.sType = STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        wr.dstSet = set;
        wr.descriptorCount = 1;
        wr.descriptorType = DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
        wr.pImageInfo = &ii;
        vkUpdateDescriptorSets(d.device, 1, &wr, 0, ptr::null());
        let vs = asm_vs_uv();
        let fs = asm_fs_tex();
        let pipe = make_graphics(d, &vs, &fs, layout);
        let mut ok = !pipe.is_null() && begin_cmd(d);
        if ok {
            let mut region: VkBufferImageCopy = zeroed();
            region.imageSubresource.aspectMask = IMAGE_ASPECT_COLOR_BIT;
            region.imageSubresource.layerCount = 1;
            region.imageExtent = VkExtent3D { width: 4, height: 4, depth: 1 };
            vkCmdCopyBufferToImage(d.cmd, stage.buffer, tex.image, IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 1, &region);
            bind_draw_state(d, pipe, vb, layout, set);
            let mut clear: VkClearValue = zeroed();
            clear.float32 = [0.0, 0.0, 1.0, 1.0];
            let mut rp: VkRenderPassBeginInfo = zeroed();
            rp.sType = STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
            rp.renderPass = d.rp;
            rp.framebuffer = frame.fb;
            rp.renderArea.extent = VkExtent2D { width: 64, height: 64 };
            rp.clearValueCount = 1;
            rp.pClearValues = &clear;
            vkCmdBeginRenderPass(d.cmd, &rp, SUBPASS_CONTENTS_INLINE);
            vkCmdDraw(d.cmd, 3, 1, 0, 0);
            vkCmdEndRenderPass(d.cmd);
            let mut back: VkBufferImageCopy = zeroed();
            back.imageSubresource.aspectMask = IMAGE_ASPECT_COLOR_BIT;
            back.imageSubresource.layerCount = 1;
            back.imageExtent = VkExtent3D { width: 64, height: 64, depth: 1 };
            vkCmdCopyImageToBuffer(d.cmd, frame.color.image, IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL, frame.readback.buffer, 1, &back);
            ok = submit(d);
        }
        if ok {
            let mut pix = vec![0u8; 64 * 64 * 4];
            ok = frame.readback.read(&mut pix);
            if ok {
                let c = &pix[(32 * 64 + 32) * 4..];
                let center = near(c[0], 255) && near(c[1], 0) && near(c[2], 0) && near(c[3], 255);
                let corner = near(pix[0], 0) && near(pix[1], 0) && near(pix[2], 255) && near(pix[3], 255);
                let hw = gfx_pipeline_uses_hw(pipe);
                let hw_ok = if d.want_tex_hw { hw == 1 } else { true };
                if !center || !corner || !hw_ok {
                    eprintln!("FAIL {} texture center={},{},{},{} corner={},{},{},{} hw={hw} (want hw {})",
                        d.name, c[0], c[1], c[2], c[3], pix[0], pix[1], pix[2], pix[3], if d.want_tex_hw { 1 } else { 0 });
                    show_note(pipe);
                    FAILS += 1;
                    ok = false;
                } else {
                    println!("  texture hw={hw} ok");
                }
            }
        }
        if !pipe.is_null() { vkDestroyPipeline(d.device, pipe, ptr::null()); }
        vkDestroySampler(d.device, sampler, ptr::null());
        vkDestroyDescriptorPool(d.device, pool, ptr::null());
        vkDestroyPipelineLayout(d.device, layout, ptr::null());
        vkDestroyDescriptorSetLayout(d.device, set_layout, ptr::null());
        ok
    }
}

fn run_compute(d: &Dev) -> bool {
    let (set_layout, layout, pool, set) = match one_set(d, DESCRIPTOR_TYPE_STORAGE_BUFFER, SHADER_STAGE_COMPUTE_BIT) {
        Some(v) => v, None => return false,
    };
    let zeros = vec![0u8; 64];
    let ssbo = match Buf::create(d.device, 64, BUFFER_USAGE_STORAGE_BUFFER_BIT) {
        Some(b) => b, None => return false,
    };
    if !ssbo.write(&zeros) { return false; }
    unsafe {
        let info = VkDescriptorBufferInfo { buffer: ssbo.buffer, offset: 0, range: 64 };
        let mut wr: VkWriteDescriptorSet = zeroed();
        wr.sType = STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        wr.dstSet = set;
        wr.descriptorCount = 1;
        wr.descriptorType = DESCRIPTOR_TYPE_STORAGE_BUFFER;
        wr.pBufferInfo = &info;
        vkUpdateDescriptorSets(d.device, 1, &wr, 0, ptr::null());
        let words = asm_cs_red();
        let module = make_shader(d.device, &words);
        let name = CString::new("main").unwrap();
        let mut ci: VkComputePipelineCreateInfo = zeroed();
        ci.sType = STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
        ci.stage.sType = STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
        ci.stage.stage = SHADER_STAGE_COMPUTE_BIT;
        ci.stage.module = module;
        ci.stage.pName = name.as_ptr();
        ci.layout = layout;
        let mut pipe = ptr::null_mut();
        let mut ok = vk_ok(vkCreateComputePipelines(d.device, ptr::null_mut(), 1, &ci, ptr::null(), &mut pipe), "compute pipeline");
        vkDestroyShaderModule(d.device, module, ptr::null());
        if ok { ok = begin_cmd(d); }
        if ok {
            vkCmdBindPipeline(d.cmd, PIPELINE_BIND_POINT_COMPUTE, pipe);
            vkCmdBindDescriptorSets(d.cmd, PIPELINE_BIND_POINT_COMPUTE, layout, 0, 1, &set, 0, ptr::null());
            vkCmdDispatch(d.cmd, 4, 1, 1);
            ok = submit(d);
        }
        if ok {
            let mut raw = [0u8; 64];
            ok = ssbo.read(&mut raw);
            let mut out = [0f32; 16];
            for i in 0..16 {
                out[i] = f32::from_le_bytes(raw[i * 4..i * 4 + 4].try_into().unwrap());
            }
            let hw = if pipe.is_null() { 0 } else { gfx_pipeline_uses_hw(pipe) };
            let mut pixels = ok;
            if ok {
                for i in 0..4 {
                    if out[i * 4] < 0.99 || out[i * 4 + 1] > 0.01 || out[i * 4 + 2] > 0.01 || out[i * 4 + 3] < 0.99 {
                        pixels = false;
                    }
                }
            }
            let hw_ok = if d.want_cs_hw { hw == 1 } else { hw == 0 };
            if !pixels || !hw_ok {
                eprintln!("FAIL {} compute hw={hw} (want {}) px0={},{},{},{}", d.name, if d.want_cs_hw { 1 } else { 0 }, out[0], out[1], out[2], out[3]);
                if !pipe.is_null() { show_note(pipe); }
                FAILS += 1;
                ok = false;
            } else {
                println!("  compute hw={hw} ok");
            }
        }
        if !pipe.is_null() { vkDestroyPipeline(d.device, pipe, ptr::null()); }
        vkDestroyDescriptorPool(d.device, pool, ptr::null());
        vkDestroyPipelineLayout(d.device, layout, ptr::null());
        vkDestroyDescriptorSetLayout(d.device, set_layout, ptr::null());
        ok
    }
}

fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

unsafe extern "system" fn test_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn run_present(d: &Dev) -> bool {
    unsafe {
        let class = wide("VkcompatTestWindow");
        let title = wide("vkcompat");
        let inst = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap_or_default();
        let mut wc = WNDCLASSW::default();
        wc.lpfnWndProc = Some(test_wnd_proc);
        wc.hInstance = inst.into();
        wc.lpszClassName = PCWSTR(class.as_ptr());
        let _ = RegisterClassW(&wc);
        let hwnd = match CreateWindowExW(WINDOW_EX_STYLE(0), PCWSTR(class.as_ptr()), PCWSTR(title.as_ptr()), WS_POPUP, 0, 0, 64, 64, None, None, Some(inst.into()), None) {
            Ok(h) => h,
            Err(_) => { fail("CreateWindow"); return false; }
        };
        std::mem::forget(class);
        let mut si: VkWin32SurfaceCreateInfoKHR = zeroed();
        si.sType = STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR;
        si.hwnd = hwnd.0;
        let mut surface = ptr::null_mut();
        let mut ok = vk_ok(vkCreateWin32SurfaceKHR(ptr::null_mut(), &si, ptr::null(), &mut surface), "vkCreateWin32SurfaceKHR");
        let mut support = 0u32;
        if !surface.is_null() {
            let _ = vkGetPhysicalDeviceSurfaceSupportKHR(d.phys, 0, surface, &mut support);
        }
        let mut sci: VkSwapchainCreateInfoKHR = zeroed();
        sci.sType = STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR;
        sci.surface = surface;
        sci.minImageCount = 2;
        sci.imageFormat = FORMAT_B8G8R8A8_UNORM;
        sci.imageColorSpace = COLOR_SPACE_SRGB_NONLINEAR_KHR;
        sci.imageExtent = VkExtent2D { width: 64, height: 64 };
        sci.imageArrayLayers = 1;
        sci.imageUsage = IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
        sci.imageSharingMode = SHARING_MODE_EXCLUSIVE;
        sci.preTransform = SURFACE_TRANSFORM_IDENTITY_BIT_KHR;
        sci.compositeAlpha = COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
        sci.presentMode = PRESENT_MODE_FIFO_KHR;
        sci.clipped = 1;
        let mut swap = ptr::null_mut();
        if ok { ok = vk_ok(vkCreateSwapchainKHR(d.device, &sci, ptr::null(), &mut swap), "vkCreateSwapchainKHR"); }
        let mut nimg = 0u32;
        if ok { ok = vk_ok(vkGetSwapchainImagesKHR(d.device, swap, &mut nimg, ptr::null_mut()), "image count"); }
        let mut images = vec![ptr::null_mut(); nimg as usize];
        if ok && nimg > 0 { ok = vk_ok(vkGetSwapchainImagesKHR(d.device, swap, &mut nimg, images.as_mut_ptr()), "images"); }
        let mut semi: VkSemaphoreCreateInfo = zeroed();
        semi.sType = STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;
        let mut sem = ptr::null_mut();
        if ok { ok = vk_ok(vkCreateSemaphore(d.device, &semi, ptr::null(), &mut sem), "semaphore"); }
        let mut index = 0u32;
        if ok { ok = vk_ok(vkAcquireNextImageKHR(d.device, swap, 0, sem, ptr::null_mut(), &mut index), "acquire"); }
        let mut view = ptr::null_mut();
        let mut fb = ptr::null_mut();
        if ok && (index as usize) < images.len() {
            let mut vi: VkImageViewCreateInfo = zeroed();
            vi.sType = STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
            vi.image = images[index as usize];
            vi.viewType = IMAGE_VIEW_TYPE_2D;
            vi.format = FORMAT_B8G8R8A8_UNORM;
            vi.subresourceRange.aspectMask = IMAGE_ASPECT_COLOR_BIT;
            vi.subresourceRange.levelCount = 1;
            vi.subresourceRange.layerCount = 1;
            ok = vk_ok(vkCreateImageView(d.device, &vi, ptr::null(), &mut view), "swap view");
            if ok {
                let mut fi: VkFramebufferCreateInfo = zeroed();
                fi.sType = STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
                fi.renderPass = d.rp;
                fi.attachmentCount = 1;
                fi.pAttachments = &view;
                fi.width = 64;
                fi.height = 64;
                fi.layers = 1;
                ok = vk_ok(vkCreateFramebuffer(d.device, &fi, ptr::null(), &mut fb), "swap fb");
            }
            if ok && begin_cmd(d) {
                let mut clear: VkClearValue = zeroed();
                clear.float32 = [1.0, 0.0, 0.0, 1.0];
                let mut rp: VkRenderPassBeginInfo = zeroed();
                rp.sType = STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
                rp.renderPass = d.rp;
                rp.framebuffer = fb;
                rp.renderArea.extent = VkExtent2D { width: 64, height: 64 };
                rp.clearValueCount = 1;
                rp.pClearValues = &clear;
                vkCmdBeginRenderPass(d.cmd, &rp, SUBPASS_CONTENTS_INLINE);
                vkCmdEndRenderPass(d.cmd);
                ok = submit(d);
            }
            let mut pr: VkPresentInfoKHR = zeroed();
            pr.sType = STRUCTURE_TYPE_PRESENT_INFO_KHR;
            pr.waitSemaphoreCount = 1;
            pr.pWaitSemaphores = &sem;
            pr.swapchainCount = 1;
            pr.pSwapchains = &swap;
            pr.pImageIndices = &index;
            if ok { ok = vk_ok(vkQueuePresentKHR(d.queue, &pr), "vkQueuePresentKHR"); }
        }
        if !fb.is_null() { vkDestroyFramebuffer(d.device, fb, ptr::null()); }
        if !view.is_null() { vkDestroyImageView(d.device, view, ptr::null()); }
        if !sem.is_null() { vkDestroySemaphore(d.device, sem, ptr::null()); }
        if !swap.is_null() { vkDestroySwapchainKHR(d.device, swap, ptr::null()); }
        if !surface.is_null() { vkDestroySurfaceKHR(ptr::null_mut(), surface, ptr::null()); }
        let _ = DestroyWindow(hwnd);
        if ok { println!("  present ok"); }
        ok
    }
}

fn check_properties2(phys: VkPhysicalDevice) -> bool {
    unsafe {
        let mut p2: VkPhysicalDeviceProperties2 = zeroed();
        p2.sType = STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2;
        vkGetPhysicalDeviceProperties2(phys, &mut p2);
        vkGetPhysicalDeviceProperties2(phys, &mut p2);
        let mut f2: VkPhysicalDeviceFeatures2 = zeroed();
        f2.sType = STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2;
        vkGetPhysicalDeviceFeatures2(phys, &mut f2);
        let mut m2: VkPhysicalDeviceMemoryProperties2 = zeroed();
        m2.sType = STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2;
        vkGetPhysicalDeviceMemoryProperties2(phys, &mut m2);
        if p2.properties.limits.maxPushConstantsSize < 128 {
            fail("maxPushConstantsSize");
            return false;
        }
        true
    }
}

fn check_icd_exports() -> bool {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => { fail("current_exe"); return false; }
    };
    let dll = exe.parent().unwrap().join("vkcompat.dll");
    let wide = wide(&dll.to_string_lossy());
    unsafe {
        let lib = match LoadLibraryW(PCWSTR(wide.as_ptr())) {
            Ok(h) => h,
            Err(e) => { eprintln!("FAIL LoadLibrary vkcompat.dll ({e})"); FAILS += 1; return false; }
        };
        let neg = GetProcAddress(lib, windows::core::s!("vk_icdNegotiateLoaderICDInterfaceVersion"));
        let gipa = GetProcAddress(lib, windows::core::s!("vk_icdGetInstanceProcAddr"));
        let gpda = GetProcAddress(lib, windows::core::s!("vk_icdGetPhysicalDeviceProcAddr"));
        let mut ok = neg.is_some() && gipa.is_some() && gpda.is_some();
        if !ok {
            fail("ICD exports missing");
        } else {
            type Neg = unsafe extern "system" fn(*mut u32) -> i32;
            type Gipa = unsafe extern "system" fn(*mut c_void, *const i8) -> PfnVoidFunction;
            let neg: Neg = std::mem::transmute(neg.unwrap());
            let get: Gipa = std::mem::transmute(gipa.unwrap());
            let mut v = 0u32;
            if neg(&mut v) != SUCCESS || v != 5 { fail("negotiate 0"); ok = false; }
            v = 9;
            if neg(&mut v) != SUCCESS || v != 5 { fail("negotiate 9"); ok = false; }
            v = 3;
            if neg(&mut v) != SUCCESS || v != 3 { fail("negotiate 3"); ok = false; }
            let name = CString::new("vkCreateInstance").unwrap();
            if get(ptr::null_mut(), name.as_ptr()).is_none() {
                fail("vk_icdGetInstanceProcAddr(NULL, vkCreateInstance)");
                ok = false;
            }
            let name = CString::new("vkCreateDevice").unwrap();
            if get(ptr::null_mut(), name.as_ptr()).is_some() {
                fail("global proc addr returned a device function");
                ok = false;
            }
        }
        let _ = FreeLibrary(lib);
        if ok { println!("icd exports ok"); }
        ok
    }
}

fn main() {
    check_icd_exports();
    let app_name = CString::new("vkcompat_test").unwrap();
    let e0 = CString::new(EXT_SURFACE).unwrap();
    let e1 = CString::new(EXT_WIN32_SURFACE).unwrap();
    let e2 = CString::new(EXT_PROPS2).unwrap();
    let exts = [e0.as_ptr(), e1.as_ptr(), e2.as_ptr()];
    unsafe {
        let mut app: VkApplicationInfo = zeroed();
        app.sType = STRUCTURE_TYPE_APPLICATION_INFO;
        app.pApplicationName = app_name.as_ptr();
        app.apiVersion = API_VERSION_1_0;
        let mut ii: VkInstanceCreateInfo = zeroed();
        ii.sType = STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
        ii.pApplicationInfo = &app;
        ii.enabledExtensionCount = 3;
        ii.ppEnabledExtensionNames = exts.as_ptr();
        let mut instance = ptr::null_mut();
        if !vk_ok(vkCreateInstance(&ii, ptr::null(), &mut instance), "vkCreateInstance") {
            std::process::exit(1);
        }
        let mut ndev = 0u32;
        if !vk_ok(vkEnumeratePhysicalDevices(instance, &mut ndev, ptr::null_mut()), "enumerate count") || ndev == 0 {
            fail("no physical devices");
            vkDestroyInstance(instance, ptr::null());
            std::process::exit(1);
        }
        let mut devs = vec![ptr::null_mut(); ndev as usize];
        if !vk_ok(vkEnumeratePhysicalDevices(instance, &mut ndev, devs.as_mut_ptr()), "enumerate") {
            std::process::exit(1);
        }
        println!("{ndev} physical device(s)");
        check_properties2(devs[0]);
        let verts = [0f32, -0.5, 0.5, 0.5, -0.5, 0.5];
        let vbytes = verts.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<_>>();
        for phys in devs {
            let Some(d) = open_dev(phys) else { continue; };
            println!("{}", d.name);
            let Some(frame) = Frame::create(&d) else { fail("target setup"); continue; };
            let Some(vb) = Buf::create(d.device, vbytes.len() as u64, BUFFER_USAGE_VERTEX_BUFFER_BIT) else { fail("vb"); continue; };
            if !vb.write(&vbytes) { fail("vb write"); continue; }
            let _ = run_color(&d, &frame, vb.buffer);
            let _ = run_loop(&d, &frame, vb.buffer);
            let _ = run_ubo(&d, &frame, vb.buffer);
            let _ = run_tex(&d, &frame, vb.buffer);
            let _ = run_compute(&d);
            let _ = run_present(&d);
            let _ = HWND::default();
        }
        vkDestroyInstance(instance, ptr::null());
    }
    let fails = unsafe { FAILS };
    if fails != 0 {
        eprintln!("{fails} failure(s)");
        std::process::exit(1);
    }
}
