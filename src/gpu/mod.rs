//! Host backends. On Windows, DirectX is preferred over OpenGL. The CPU path has no backend.
//! Linux and macOS enumerate OpenGL, then the CPU device. Direct3D is not compiled there.

#[cfg(windows)]
mod dx11;
#[cfg(windows)]
mod dx9;
mod gl;

use crate::emit::Dialect;
#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, GetDC, ReleaseDC, SRCCOPY, StretchDIBits,
};
#[cfg(windows)]
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, RegisterClassW, CS_OWNDC,
    WINDOW_EX_STYLE, WNDCLASSW, WS_POPUP,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    D3d11,
    D3d9,
    Gl,
    Cpu,
}

#[derive(Clone, Debug)]
pub struct PhysDesc {
    pub name: String,
    pub kind: Kind,
    pub vendor: u32,
    pub device: u32,
    pub device_type: i32,
    pub adapter: u32,
    pub gl_tier: u32,
    pub fl11: bool,
    pub gl_major: u32,
    pub gl_minor: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Tier {
    pub dialect: Dialect,
    pub fixed_function: bool,
    pub compute: bool,
    pub fan: bool,
    pub instancing: bool,
    pub flip_y: bool,
    pub clip_y_up: bool,
}

pub struct Tex<'a> {
    pub w: u32,
    pub h: u32,
    pub rgba: &'a [u8],
}

pub struct DrawDesc<'a> {
    pub key: usize,
    pub vs: &'a str,
    pub fs: &'a str,
    pub verts: &'a [u8],
    pub stride: u32,
    pub count: u32,
    pub clear: [f32; 4],
    pub width: u32,
    pub height: u32,
    pub ubo: Option<&'a [u8]>,
    pub ubo_slot: i32,
    pub tex: Option<Tex<'a>>,
    pub components: u8,
}

pub struct FixedDesc<'a> {
    pub color: [f32; 4],
    pub textured: bool,
    pub clips: &'a [[f32; 4]],
    pub uvs: &'a [[f32; 2]],
    pub clear: [f32; 4],
    pub width: u32,
    pub height: u32,
    pub tex: Option<Tex<'a>>,
}

pub struct DispatchDesc<'a> {
    pub key: usize,
    pub cs: &'a str,
    pub groups: [u32; 3],
    pub ssbo: &'a [u8],
}

pub trait Backend {
    fn tier(&self) -> Tier;
    fn draw(&mut self, d: &DrawDesc) -> Result<Vec<u8>, String>;
    fn draw_fixed(&mut self, _d: &FixedDesc) -> Result<Vec<u8>, String> {
        Err("fixed function is not available".into())
    }
    fn dispatch(&mut self, _d: &DispatchDesc) -> Result<Vec<u8>, String> {
        Err("compute is software".into())
    }
}

fn env_is(name: &str, expect: &str) -> bool {
    std::env::var(name).map(|v| v == expect).unwrap_or(false)
}

fn env_val(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

fn cpu_desc() -> PhysDesc {
    PhysDesc {
        name: "vkcompat CPU".into(),
        kind: Kind::Cpu,
        vendor: 0x10001,
        device: 1,
        device_type: crate::vk::PHYSICAL_DEVICE_TYPE_CPU,
        adapter: 0,
        gl_tier: 0,
        fl11: false,
        gl_major: 0,
        gl_minor: 0,
    }
}

pub fn enumerate() -> Vec<PhysDesc> {
    let only = env_val("VKCOMPAT_ONLY");
    let mut out = Vec::new();
    #[cfg(windows)]
    {
        let want_dx11 = only.is_empty() || only == "d3d11";
        let want_d3d9 = only == "d3d9" || (only.is_empty() && out.is_empty());
        let want_gl = only == "gl";
        if want_dx11 {
            out.extend(dx11::enumerate());
        }
        if only == "d3d9" || (only.is_empty() && out.is_empty()) {
            if let Some(d) = dx9::probe() {
                out.push(d);
            }
        }
        if want_gl || (only.is_empty() && out.is_empty()) {
            let _ = want_d3d9;
            out.extend(gl::enumerate());
        }
        if only.is_empty() || only == "cpu" {
            out.push(cpu_desc());
        }
    }
    #[cfg(not(windows))]
    {
        if only.is_empty() || only == "gl" {
            out.extend(gl::enumerate());
        }
        if only.is_empty() || only == "cpu" {
            out.push(cpu_desc());
        }
    }
    out
}

pub fn open(desc: &PhysDesc) -> Result<Box<dyn Backend>, String> {
    match desc.kind {
        #[cfg(windows)]
        Kind::D3d11 => dx11::open(desc.adapter).map(|b| Box::new(b) as Box<dyn Backend>),
        #[cfg(windows)]
        Kind::D3d9 => dx9::open().map(|b| Box::new(b) as Box<dyn Backend>),
        #[cfg(not(windows))]
        Kind::D3d11 | Kind::D3d9 => Err("Direct3D is not available on this platform".into()),
        Kind::Gl => gl::open(desc.gl_tier).map(|b| Box::new(b) as Box<dyn Backend>),
        Kind::Cpu => Err("cpu".into()),
    }
}

pub fn device_type_for_vendor(vendor: u32) -> i32 {
    match vendor {
        0x10de | 0x1002 => crate::vk::PHYSICAL_DEVICE_TYPE_DISCRETE_GPU,
        0x8086 => crate::vk::PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU,
        _ => crate::vk::PHYSICAL_DEVICE_TYPE_OTHER,
    }
}

#[cfg(windows)]
unsafe extern "system" fn hidden_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

#[cfg(windows)]
pub(crate) fn hidden_window(class_name: &str, title: &str, w: i32, h: i32) -> Result<(HWND, *mut c_void), String> {
    unsafe {
        let inst = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW {e}"))?;
        let class_buf = class_wide(class_name);
        let title_buf = class_wide(title);
        let wc = WNDCLASSW {
            style: CS_OWNDC,
            lpfnWndProc: Some(hidden_wnd_proc),
            hInstance: inst.into(),
            lpszClassName: windows::core::PCWSTR(class_buf.as_ptr()),
            ..Default::default()
        };
        let _ = RegisterClassW(&wc);
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            windows::core::PCWSTR(class_buf.as_ptr()),
            windows::core::PCWSTR(title_buf.as_ptr()),
            WS_POPUP,
            0,
            0,
            w,
            h,
            None,
            None,
            Some(inst.into()),
            None,
        )
        .map_err(|e| format!("CreateWindowExW {e}"))?;
        std::mem::forget(class_buf);
        std::mem::forget(title_buf);
        Ok((hwnd, inst.0))
    }
}

#[cfg(windows)]
fn class_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
pub(crate) fn destroy_window(hwnd: HWND) {
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

pub fn present_to_window(hwnd: isize, pixels: &[u8], w: u32, h: u32, format: i32) {
    #[cfg(windows)]
    gdi_present(hwnd, pixels, w, h, format);
    #[cfg(not(windows))]
    {
        let _ = (hwnd, pixels, w, h, format);
    }
}

#[cfg(windows)]
fn gdi_present(hwnd: isize, pixels: &[u8], w: u32, h: u32, format: i32) {
    if hwnd == 0 || w == 0 || h == 0 || pixels.is_empty() {
        return;
    }
    let mut bgra = Vec::with_capacity((w * h * 4) as usize);
    let bpp = crate::vk::format_bpp(format).max(4) as usize;
    for i in 0..(w * h) as usize {
        let s = i * bpp;
        if s + 4 > pixels.len() {
            break;
        }
        let (b, g, r, a) = if format == crate::vk::FORMAT_B8G8R8A8_UNORM {
            (pixels[s], pixels[s + 1], pixels[s + 2], pixels[s + 3])
        } else if format == crate::vk::FORMAT_R32G32B32A32_SFLOAT {
            let c = |o: usize| {
                let f = f32::from_le_bytes(pixels[s + o..s + o + 4].try_into().unwrap_or([0; 4]));
                (f * 255.0 + 0.5).clamp(0.0, 255.0) as u8
            };
            (c(8), c(4), c(0), c(12))
        } else {
            (pixels[s + 2], pixels[s + 1], pixels[s], pixels[s + 3])
        };
        bgra.extend_from_slice(&[b, g, r, a]);
    }
    unsafe {
        let window = HWND(hwnd as *mut c_void);
        let hdc = GetDC(Some(window));
        if hdc.0.is_null() {
            return;
        }
        let mut rc = RECT::default();
        let _ = GetClientRect(window, &mut rc);
        let dw = (rc.right - rc.left).max(1);
        let dh = (rc.bottom - rc.top).max(1);
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w as i32,
                biHeight: -(h as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        StretchDIBits(
            hdc,
            0,
            0,
            dw,
            dh,
            0,
            0,
            w as i32,
            h as i32,
            Some(bgra.as_ptr() as *const c_void),
            &bmi,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
        let _ = ReleaseDC(Some(window), hdc);
    }
}

pub fn log_enabled() -> bool {
    env_is("VKCOMPAT_LOG", "1") || env_is("GFX_LOG", "1") || env_is("VKDX_LOG", "1") || env_is("VKGL_LOG", "1")
}

pub fn logf(msg: &str) {
    if log_enabled() {
        eprintln!("vkcompat: {msg}");
    }
}
