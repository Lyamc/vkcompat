//! Direct3D 9 backend. Shader model 3 only. Compute and storage buffers stay on the CPU.
//! A device is advertised only after GetDeviceCaps and CreateDevice both succeed.

use super::{device_type_for_vendor, hidden_window, destroy_window, Backend, DispatchDesc, DrawDesc, PhysDesc, Tier};
use crate::emit::Dialect;
use std::collections::HashMap;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D9::*;
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::core::{PCSTR, s};

pub struct Dx9 {
    d3d: Option<IDirect3D9>,
    device: Option<IDirect3DDevice9>,
    hwnd: HWND,
    decl: Option<IDirect3DVertexDeclaration9>,
    vs: HashMap<usize, IDirect3DVertexShader9>,
    ps: HashMap<usize, IDirect3DPixelShader9>,
}

pub fn probe() -> Option<PhysDesc> {
    unsafe {
        let d3d = Direct3DCreate9(D3D_SDK_VERSION)?;
        let mut caps = D3DCAPS9::default();
        if d3d.GetDeviceCaps(D3DADAPTER_DEFAULT, D3DDEVTYPE_HAL, &mut caps).is_err() {
            return None;
        }
        // D3DVS_VERSION(3,0) = 0xFFFE0300, D3DPS_VERSION(3,0) = 0xFFFF0300.
        if caps.VertexShaderVersion < 0xFFFE_0300 || caps.PixelShaderVersion < 0xFFFF_0300 {
            return None;
        }
        let mut ident = D3DADAPTER_IDENTIFIER9::default();
        let _ = d3d.GetAdapterIdentifier(D3DADAPTER_DEFAULT, 0, &mut ident);
        let (hwnd, _) = hidden_window("VkcompatHidden", "vkcompat", 64, 64).ok()?;
        let mut pp = present_params(hwnd);
        let mut device = None;
        let flags = dx9_behavior_flags();
        let created = d3d.CreateDevice(
            D3DADAPTER_DEFAULT,
            D3DDEVTYPE_HAL,
            hwnd,
            flags,
            &mut pp,
            &mut device,
        );
        drop(device);
        destroy_window(hwnd);
        if let Err(e) = created {
            eprintln!("vkcompat: D3D9 CreateDevice probe failed: {e}");
            return None;
        }
        Some(PhysDesc {
            name: "vkcompat D3D9".into(),
            kind: super::Kind::D3d9,
            vendor: ident.VendorId,
            device: ident.DeviceId,
            device_type: device_type_for_vendor(ident.VendorId),
            adapter: 0,
            gl_tier: 0,
            fl11: false,
            gl_major: 0,
            gl_minor: 0,
        })
    }
}

pub fn open() -> Result<Dx9, String> {
    unsafe {
        let d3d = Direct3DCreate9(D3D_SDK_VERSION).ok_or("Direct3DCreate9 returned null")?;
        let (hwnd, _) = hidden_window("VkcompatHidden", "vkcompat", 64, 64)?;
        let mut pp = present_params(hwnd);
        let mut device = None;
        let flags = dx9_behavior_flags();
        d3d.CreateDevice(
            D3DADAPTER_DEFAULT,
            D3DDEVTYPE_HAL,
            hwnd,
            flags,
            &mut pp,
            &mut device,
        )
        .map_err(|e| {
            destroy_window(hwnd);
            format!("D3D9 CreateDevice {e}")
        })?;
        let device = device.ok_or("null d3d9 device")?;
        let decl = make_decl(&device)?;
        Ok(Dx9 {
            d3d: Some(d3d),
            device: Some(device),
            hwnd,
            decl: Some(decl),
            vs: HashMap::new(),
            ps: HashMap::new(),
        })
    }
}

fn present_params(hwnd: HWND) -> D3DPRESENT_PARAMETERS {
    let mut pp = D3DPRESENT_PARAMETERS::default();
    pp.Windowed = true.into();
    pp.SwapEffect = D3DSWAPEFFECT_DISCARD;
    pp.BackBufferFormat = D3DFMT_UNKNOWN;
    pp.hDeviceWindow = hwnd;
    pp.PresentationInterval = D3DPRESENT_INTERVAL_IMMEDIATE as u32;
    pp
}

fn make_decl(device: &IDirect3DDevice9) -> Result<IDirect3DVertexDeclaration9, String> {
    let elems = [
        D3DVERTEXELEMENT9 {
            Stream: 0,
            Offset: 0,
            Type: D3DDECLTYPE_FLOAT2.0 as u8,
            Method: D3DDECLMETHOD_DEFAULT.0 as u8,
            Usage: D3DDECLUSAGE_POSITION.0 as u8,
            UsageIndex: 0,
        },
        D3DVERTEXELEMENT9 {
            Stream: 0xFF,
            Offset: 0,
            Type: D3DDECLTYPE_UNUSED.0 as u8,
            Method: 0,
            Usage: 0,
            UsageIndex: 0,
        },
    ];
    unsafe {
        device
            .CreateVertexDeclaration(elems.as_ptr())
            .map_err(|e| format!("CreateVertexDeclaration {e}"))
    }
}

impl Drop for Dx9 {
    fn drop(&mut self) {
        self.vs.clear();
        self.ps.clear();
        self.decl = None;
        self.device = None;
        self.d3d = None;
        if !self.hwnd.0.is_null() {
            destroy_window(self.hwnd);
            self.hwnd = HWND::default();
        }
    }
}

impl Dx9 {
    fn device(&self) -> Result<&IDirect3DDevice9, String> {
        self.device.as_ref().ok_or_else(|| "d3d9 device is closed".into())
    }

    fn shaders(&mut self, key: usize, vs: &str, fs: &str) -> Result<(), String> {
        if self.vs.contains_key(&key) && self.ps.contains_key(&key) {
            return Ok(());
        }
        let vs_bc = compile_hlsl(vs, "vs_3_0")?;
        let ps_bc = compile_hlsl(fs, "ps_3_0")?;
        let device = self.device()?.clone();
        unsafe {
            let vso = device
                .CreateVertexShader(vs_bc.as_ptr() as *const u32)
                .map_err(|e| format!("CreateVertexShader {e}\n{vs}"))?;
            let pso = device
                .CreatePixelShader(ps_bc.as_ptr() as *const u32)
                .map_err(|e| format!("CreatePixelShader {e}\n{fs}"))?;
            self.vs.insert(key, vso);
            self.ps.insert(key, pso);
        }
        Ok(())
    }
}

impl Backend for Dx9 {
    fn tier(&self) -> Tier {
        Tier {
            dialect: Dialect::Hlsl3,
            fixed_function: false,
            compute: false,
            fan: false,
            instancing: false,
            flip_y: false,
            clip_y_up: true,
        }
    }

    fn draw(&mut self, d: &DrawDesc) -> Result<Vec<u8>, String> {
        self.shaders(d.key, d.vs, d.fs)?;
        let device = self.device()?.clone();
        let vs = self.vs.get(&d.key).ok_or("missing vs")?.clone();
        let ps = self.ps.get(&d.key).ok_or("missing ps")?.clone();
        let decl = self.decl.as_ref().ok_or("missing decl")?.clone();
        unsafe {
            let (color, rt) = render_target(&device, d.width, d.height)?;
            device
                .SetRenderTarget(0, &rt)
                .map_err(|e| format!("SetRenderTarget {e}"))?;
            let vp = D3DVIEWPORT9 {
                X: 0,
                Y: 0,
                Width: d.width.max(1),
                Height: d.height.max(1),
                MinZ: 0.0,
                MaxZ: 1.0,
            };
            let _ = device.SetViewport(&vp);
            let _ = device.SetRenderState(D3DRS_CULLMODE, D3DCULL_NONE.0 as u32);
            let _ = device.SetRenderState(D3DRS_ZENABLE, 0);
            let _ = device.SetRenderState(D3DRS_LIGHTING, 0);
            let _ = device.SetRenderState(D3DRS_ALPHABLENDENABLE, 0);
            let packed = pack_a8r8g8b8(d.clear);
            device
                .Clear(0, std::ptr::null(), D3DCLEAR_TARGET as u32, packed, 1.0, 0)
                .map_err(|e| format!("Clear {e}"))?;
            device.BeginScene().map_err(|e| format!("BeginScene {e}"))?;
            let _ = device.SetVertexDeclaration(&decl);
            let _ = device.SetVertexShader(&vs);
            let _ = device.SetPixelShader(&ps);
            if d.ubo_slot >= 0 {
                if let Some(ubo) = d.ubo {
                    let mut regs = [0f32; 4];
                    let n = ubo.len().min(16);
                    if n >= 4 {
                        regs[0] = f32::from_le_bytes(ubo[0..4].try_into().unwrap_or([0; 4]));
                    }
                    if n >= 8 {
                        regs[1] = f32::from_le_bytes(ubo[4..8].try_into().unwrap_or([0; 4]));
                    }
                    if n >= 12 {
                        regs[2] = f32::from_le_bytes(ubo[8..12].try_into().unwrap_or([0; 4]));
                    }
                    if n >= 16 {
                        regs[3] = f32::from_le_bytes(ubo[12..16].try_into().unwrap_or([0; 4]));
                    }
                    let reg = d.ubo_slot as u32;
                    let _ = device.SetVertexShaderConstantF(reg, regs.as_ptr(), 1);
                    let _ = device.SetPixelShaderConstantF(reg, regs.as_ptr(), 1);
                }
            }
            let tex = if let Some(tex) = d.tex.as_ref() {
                let t = upload_texture(&device, tex.w, tex.h, tex.rgba)?;
                let _ = device.SetTexture(0, &t);
                let _ = device.SetSamplerState(0, D3DSAMP_MINFILTER, D3DTEXF_POINT.0 as u32);
                let _ = device.SetSamplerState(0, D3DSAMP_MAGFILTER, D3DTEXF_POINT.0 as u32);
                let _ = device.SetSamplerState(0, D3DSAMP_MIPFILTER, D3DTEXF_NONE.0 as u32);
                let _ = device.SetSamplerState(0, D3DSAMP_ADDRESSU, D3DTADDRESS_CLAMP.0 as u32);
                let _ = device.SetSamplerState(0, D3DSAMP_ADDRESSV, D3DTADDRESS_CLAMP.0 as u32);
                Some(t)
            } else {
                let _ = device.SetTexture(0, None);
                None
            };
            let prims = d.count / 3;
            if prims > 0 {
                device
                    .DrawPrimitiveUP(
                        D3DPT_TRIANGLELIST,
                        prims,
                        d.verts.as_ptr() as *const _,
                        d.stride,
                    )
                    .map_err(|e| format!("DrawPrimitiveUP {e}"))?;
            }
            device.EndScene().map_err(|e| format!("EndScene {e}"))?;
            drop(tex);
            let pixels = readback(&device, &rt, d.width, d.height)?;
            drop(color);
            Ok(pixels)
        }
    }

    fn dispatch(&mut self, _d: &DispatchDesc) -> Result<Vec<u8>, String> {
        Err("compute is software".into())
    }
}

fn pack_a8r8g8b8(c: [f32; 4]) -> u32 {
    let u = |x: f32| (x * 255.0 + 0.5).clamp(0.0, 255.0) as u32;
    (u(c[3]) << 24) | (u(c[0]) << 16) | (u(c[1]) << 8) | u(c[2])
}

fn render_target(
    device: &IDirect3DDevice9,
    w: u32,
    h: u32,
) -> Result<(IDirect3DTexture9, IDirect3DSurface9), String> {
    unsafe {
        let mut tex = None;
        device
            .CreateTexture(
                w.max(1),
                h.max(1),
                1,
                D3DUSAGE_RENDERTARGET as u32,
                D3DFMT_A8R8G8B8,
                D3DPOOL_DEFAULT,
                &mut tex,
                std::ptr::null_mut(),
            )
            .map_err(|e| format!("CreateTexture RT {e}"))?;
        let tex = tex.ok_or("null rt texture")?;
        let surf = tex
            .GetSurfaceLevel(0)
            .map_err(|e| format!("GetSurfaceLevel {e}"))?;
        Ok((tex, surf))
    }
}

fn upload_texture(
    device: &IDirect3DDevice9,
    w: u32,
    h: u32,
    rgba: &[u8],
) -> Result<IDirect3DTexture9, String> {
    unsafe {
        let mut tex = None;
        device
            .CreateTexture(
                w.max(1),
                h.max(1),
                1,
                0,
                D3DFMT_A8R8G8B8,
                D3DPOOL_MANAGED,
                &mut tex,
                std::ptr::null_mut(),
            )
            .map_err(|e| format!("CreateTexture {e}"))?;
        let tex = tex.ok_or("null texture")?;
        let mut locked = D3DLOCKED_RECT::default();
        tex.LockRect(0, &mut locked, std::ptr::null(), 0)
            .map_err(|e| format!("LockRect {e}"))?;
        let pitch = locked.Pitch as usize;
        let dst = locked.pBits as *mut u8;
        for y in 0..h as usize {
            let row = dst.add(y * pitch);
            for x in 0..w as usize {
                let s = (y * w as usize + x) * 4;
                let (r, g, b, a) = if s + 4 <= rgba.len() {
                    (rgba[s], rgba[s + 1], rgba[s + 2], rgba[s + 3])
                } else {
                    (0, 0, 0, 255)
                };
                std::ptr::copy_nonoverlapping([b, g, r, a].as_ptr(), row.add(x * 4), 4);
            }
        }
        tex.UnlockRect(0).map_err(|e| format!("UnlockRect {e}"))?;
        Ok(tex)
    }
}

fn readback(device: &IDirect3DDevice9, rt: &IDirect3DSurface9, w: u32, h: u32) -> Result<Vec<u8>, String> {
    unsafe {
        let mut sys = None;
        device
            .CreateOffscreenPlainSurface(
                w.max(1),
                h.max(1),
                D3DFMT_A8R8G8B8,
                D3DPOOL_SYSTEMMEM,
                &mut sys,
                std::ptr::null_mut(),
            )
            .map_err(|e| format!("CreateOffscreenPlainSurface {e}"))?;
        let sys = sys.ok_or("null sysmem surface")?;
        device
            .GetRenderTargetData(rt, &sys)
            .map_err(|e| format!("GetRenderTargetData {e}"))?;
        let mut locked = D3DLOCKED_RECT::default();
        sys.LockRect(&mut locked, std::ptr::null(), D3DLOCK_READONLY as u32)
            .map_err(|e| format!("LockRect sys {e}"))?;
        let pitch = locked.Pitch as usize;
        let src = locked.pBits as *const u8;
        let mut out = vec![0u8; (w * h * 4) as usize];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let p = std::slice::from_raw_parts(src.add(y * pitch + x * 4), 4);
                let d = (y * w as usize + x) * 4;
                out[d] = p[2];
                out[d + 1] = p[1];
                out[d + 2] = p[0];
                out[d + 3] = p[3];
            }
        }
        let _ = sys.UnlockRect();
        Ok(out)
    }
}

fn compile_hlsl(src: &str, profile: &str) -> Result<Vec<u8>, String> {
    let target = std::ffi::CString::new(profile).map_err(|e| e.to_string())?;
    let name = std::ffi::CString::new("shader.hlsl").unwrap();
    unsafe {
        let mut code = None;
        let mut errs = None;
        let hr = D3DCompile(
            src.as_ptr() as *const _,
            src.len(),
            PCSTR(name.as_ptr() as *const u8),
            None,
            None,
            s!("main"),
            PCSTR(target.as_ptr() as *const u8),
            0,
            0,
            &mut code,
            Some(&mut errs),
        );
        if let Err(e) = hr {
            let msg = blob_string(errs.as_ref());
            return Err(format!("D3DCompile {profile}: {e}\n{msg}\n{src}"));
        }
        let blob = code.ok_or("D3DCompile returned no bytecode")?;
        let ptr = blob.GetBufferPointer();
        let len = blob.GetBufferSize();
        Ok(std::slice::from_raw_parts(ptr as *const u8, len).to_vec())
    }
}

fn dx9_behavior_flags() -> u32 {
    (D3DCREATE_HARDWARE_VERTEXPROCESSING | D3DCREATE_FPU_PRESERVE | D3DCREATE_MULTITHREADED) as u32
}

fn blob_string(blob: Option<&ID3DBlob>) -> String {
    let Some(blob) = blob else {
        return String::new();
    };
    unsafe {
        let ptr = blob.GetBufferPointer() as *const u8;
        let len = blob.GetBufferSize();
        if ptr.is_null() || len == 0 {
            return String::new();
        }
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned()
    }
}
