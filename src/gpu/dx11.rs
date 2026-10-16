//! Direct3D 11 backend. Feature level 11 runs compute. Feature level 10 does not.
//! One device per hardware adapter. Software adapters are skipped.

use super::{device_type_for_vendor, Backend, DispatchDesc, DrawDesc, PhysDesc, Tier};
use crate::emit::Dialect;
use std::collections::HashMap;
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Foundation::HMODULE;
use windows::core::{Interface, PCSTR, s};

pub struct Dx11 {
    device: ID3D11Device,
    ctx: ID3D11DeviceContext,
    fl11: bool,
    pipes: HashMap<usize, GraphicsPipe>,
    comps: HashMap<usize, ID3D11ComputeShader>,
    rast: Option<ID3D11RasterizerState>,
    samp: Option<ID3D11SamplerState>,
}

struct GraphicsPipe {
    vs: ID3D11VertexShader,
    ps: ID3D11PixelShader,
    layout: ID3D11InputLayout,
}

struct ColorTarget {
    tex: ID3D11Texture2D,
    rtv: ID3D11RenderTargetView,
    w: u32,
    h: u32,
}

pub fn enumerate() -> Vec<PhysDesc> {
    let mut out = Vec::new();
    unsafe {
        let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("vkcompat: CreateDXGIFactory1 failed: {e}");
                return out;
            }
        };
        let mut i = 0u32;
        loop {
            let adapter = match factory.EnumAdapters1(i) {
                Ok(a) => a,
                Err(_) => break,
            };
            let desc = match adapter.GetDesc1() {
                Ok(d) => d,
                Err(_) => {
                    i += 1;
                    continue;
                }
            };
            let software = dxgi_software(&desc);
            if software {
                i += 1;
                continue;
            }
            match probe_adapter(&adapter) {
                Some(fl11) => {
                    let name = if fl11 { "vkcompat D3D11" } else { "vkcompat D3D11 FL10" };
                    out.push(PhysDesc {
                        name: name.into(),
                        kind: super::Kind::D3d11,
                        vendor: desc.VendorId,
                        device: desc.DeviceId,
                        device_type: device_type_for_vendor(desc.VendorId),
                        adapter: i,
                        gl_tier: 0,
                        fl11,
                        gl_major: 0,
                        gl_minor: 0,
                    });
                }
                None => {}
            }
            i += 1;
        }
    }
    out
}

fn dxgi_software(desc: &DXGI_ADAPTER_DESC1) -> bool {
    // DXGI_ADAPTER_FLAG_SOFTWARE is 2. Flags is a u32-sized newtype or a raw u32.
    let bits = flags_bits(desc.Flags);
    (bits & 2) != 0
}

fn flags_bits<T>(v: T) -> u32 {
    let raw = &v as *const T as *const u32;
    unsafe { *raw }
}

fn probe_adapter(adapter: &IDXGIAdapter1) -> Option<bool> {
    unsafe {
        let mut device = None;
        let mut context = None;
        let mut level = D3D_FEATURE_LEVEL_10_0;
        let levels = [D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_10_0];
        if D3D11CreateDevice(
            adapter,
            D3D_DRIVER_TYPE_UNKNOWN,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            Some(&mut level),
            Some(&mut context),
        )
        .is_err()
        {
            return None;
        }
        let fl11 = level.0 >= D3D_FEATURE_LEVEL_11_0.0;
        Some(fl11)
    }
}

pub fn open(adapter_index: u32) -> Result<Dx11, String> {
    unsafe {
        let factory: IDXGIFactory1 = CreateDXGIFactory1().map_err(|e| format!("dxgi factory {e}"))?;
        let adapter = factory
            .EnumAdapters1(adapter_index)
            .map_err(|e| format!("adapter {adapter_index}: {e}"))?;
        let mut device = None;
        let mut context = None;
        let mut level = D3D_FEATURE_LEVEL_10_0;
        let levels = [D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_10_0];
        D3D11CreateDevice(
            &adapter,
            D3D_DRIVER_TYPE_UNKNOWN,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            Some(&mut level),
            Some(&mut context),
        )
        .map_err(|e| format!("D3D11CreateDevice {e}"))?;
        let device = device.ok_or("null d3d11 device")?;
        let ctx = context.ok_or("null d3d11 context")?;
        let fl11 = level.0 >= D3D_FEATURE_LEVEL_11_0.0;
        Ok(Dx11 {
            device,
            ctx,
            fl11,
            pipes: HashMap::new(),
            comps: HashMap::new(),
            rast: None,
            samp: None,
        })
    }
}

impl Dx11 {
    fn ensure_state(&mut self) -> Result<(), String> {
        if self.rast.is_none() {
            unsafe {
                let mut rd = D3D11_RASTERIZER_DESC::default();
                rd.FillMode = D3D11_FILL_SOLID;
                rd.CullMode = D3D11_CULL_NONE;
                rd.DepthClipEnable = true.into();
                let mut rs = None;
                self.device
                    .CreateRasterizerState(&rd, Some(&mut rs))
                    .map_err(|e| format!("rasterizer {e}"))?;
                self.rast = rs;
            }
        }
        if self.samp.is_none() {
            unsafe {
                let mut sd = D3D11_SAMPLER_DESC::default();
                sd.Filter = D3D11_FILTER_MIN_MAG_MIP_POINT;
                sd.AddressU = D3D11_TEXTURE_ADDRESS_CLAMP;
                sd.AddressV = D3D11_TEXTURE_ADDRESS_CLAMP;
                sd.AddressW = D3D11_TEXTURE_ADDRESS_CLAMP;
                sd.ComparisonFunc = D3D11_COMPARISON_NEVER;
                sd.MinLOD = 0.0;
                sd.MaxLOD = f32::MAX;
                let mut s = None;
                self.device
                    .CreateSamplerState(&sd, Some(&mut s))
                    .map_err(|e| format!("sampler {e}"))?;
                self.samp = s;
            }
        }
        Ok(())
    }

    fn graphics(&mut self, key: usize, vs: &str, fs: &str, components: u8) -> Result<(), String> {
        if self.pipes.contains_key(&key) {
            return Ok(());
        }
        let vs_profile = if self.fl11 { "vs_5_0" } else { "vs_4_0" };
        let ps_profile = if self.fl11 { "ps_5_0" } else { "ps_4_0" };
        let vs_bc = compile_hlsl(vs, vs_profile)?;
        let ps_bc = compile_hlsl(fs, ps_profile)?;
        unsafe {
            let mut vso = None;
            self.device
                .CreateVertexShader(&vs_bc, None, Some(&mut vso))
                .map_err(|e| format!("CreateVertexShader {e}"))?;
            let mut pso = None;
            self.device
                .CreatePixelShader(&ps_bc, None, Some(&mut pso))
                .map_err(|e| format!("CreatePixelShader {e}"))?;
            let fmt = if components <= 2 {
                DXGI_FORMAT_R32G32_FLOAT
            } else {
                DXGI_FORMAT_R32G32B32A32_FLOAT
            };
            let elem = [D3D11_INPUT_ELEMENT_DESC {
                SemanticName: s!("TEXCOORD"),
                SemanticIndex: 0,
                Format: fmt,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            }];
            let mut layout = None;
            self.device
                .CreateInputLayout(&elem, &vs_bc, Some(&mut layout))
                .map_err(|e| format!("CreateInputLayout {e}\n{vs}"))?;
            self.pipes.insert(
                key,
                GraphicsPipe {
                    vs: vso.ok_or("null vs")?,
                    ps: pso.ok_or("null ps")?,
                    layout: layout.ok_or("null layout")?,
                },
            );
        }
        Ok(())
    }
}

impl Backend for Dx11 {
    fn tier(&self) -> Tier {
        Tier {
            dialect: Dialect::Hlsl5,
            fixed_function: false,
            compute: self.fl11,
            fan: false,
            instancing: true,
            flip_y: false,
            clip_y_up: true,
        }
    }

    fn draw(&mut self, d: &DrawDesc) -> Result<Vec<u8>, String> {
        self.ensure_state()?;
        self.graphics(d.key, d.vs, d.fs, d.components)?;
        let target = color_target(&self.device, d.width, d.height)?;
        unsafe {
            let pipe = self.pipes.get(&d.key).ok_or("missing pipeline")?;
            let vb = upload_buffer(&self.device, d.verts, D3D11_BIND_VERTEX_BUFFER.0 as u32, false, 0)?;
            let ctx = &self.ctx;
            ctx.IASetInputLayout(&pipe.layout);
            ctx.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            let bufs = [Some(vb)];
            let stride = d.stride;
            let offset = 0u32;
            ctx.IASetVertexBuffers(0, 1, Some(bufs.as_ptr()), Some(&stride), Some(&offset));
            ctx.VSSetShader(&pipe.vs, None);
            ctx.PSSetShader(&pipe.ps, None);
            if let Some(rs) = &self.rast {
                ctx.RSSetState(rs);
            }
            let vp = D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: d.width as f32,
                Height: d.height as f32,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };
            ctx.RSSetViewports(Some(&[vp]));
            let rtvs = [Some(target.rtv.clone())];
            ctx.OMSetRenderTargets(Some(&rtvs), None);
            ctx.ClearRenderTargetView(&target.rtv, &d.clear);
            bind_ubo(ctx, &self.device, d.ubo, d.ubo_slot)?;
            bind_tex(ctx, &self.device, d.tex.as_ref(), self.samp.as_ref())?;
            ctx.Draw(d.count, 0);
            let null_srv: [Option<ID3D11ShaderResourceView>; 1] = [None];
            ctx.PSSetShaderResources(0, Some(&null_srv));
            ctx.Flush();
        }
        read_texture(&self.device, &self.ctx, &target.tex, d.width, d.height)
    }

    fn dispatch(&mut self, d: &DispatchDesc) -> Result<Vec<u8>, String> {
        if !self.fl11 {
            return Err("compute requires feature level 11".into());
        }
        if !self.comps.contains_key(&d.key) {
            let bc = compile_hlsl(d.cs, "cs_5_0")?;
            unsafe {
                let mut cs = None;
                self.device
                    .CreateComputeShader(&bc, None, Some(&mut cs))
                    .map_err(|e| format!("CreateComputeShader {e}\n{}", d.cs))?;
                self.comps.insert(d.key, cs.ok_or("null cs")?);
            }
        }
        let n_bytes = d.ssbo.len().max(16);
        let elems = (n_bytes / 16) as u32;
        unsafe {
            let buf = structured_buffer(&self.device, d.ssbo)?;
            let resource: ID3D11Resource = buf.cast().map_err(|e| e.to_string())?;
            let uav_desc = D3D11_UNORDERED_ACCESS_VIEW_DESC {
                Format: DXGI_FORMAT_UNKNOWN,
                ViewDimension: D3D11_UAV_DIMENSION_BUFFER,
                Anonymous: D3D11_UNORDERED_ACCESS_VIEW_DESC_0 {
                    Buffer: D3D11_BUFFER_UAV {
                        FirstElement: 0,
                        NumElements: elems.max(1),
                        Flags: 0,
                    },
                },
            };
            let mut uav = None;
            self.device
                .CreateUnorderedAccessView(&resource, Some(&uav_desc), Some(&mut uav))
                .map_err(|e| format!("uav {e}"))?;
            let uav = uav.ok_or("null uav")?;
            let cs = self.comps.get(&d.key).ok_or("missing cs")?;
            self.ctx.CSSetShader(cs, None);
            let uavs = [Some(uav.clone())];
            let counts = [u32::MAX];
            self.ctx.CSSetUnorderedAccessViews(0, 1, Some(uavs.as_ptr()), Some(counts.as_ptr()));
            self.ctx.Dispatch(d.groups[0], d.groups[1], d.groups[2]);
            let none: [Option<ID3D11UnorderedAccessView>; 1] = [None];
            self.ctx.CSSetUnorderedAccessViews(0, 1, Some(none.as_ptr()), None);
            self.ctx.Flush();
            read_buffer(&self.device, &self.ctx, &buf, n_bytes)
        }
    }
}

fn compile_hlsl(src: &str, profile: &str) -> Result<Vec<u8>, String> {
    let target = std::ffi::CString::new(profile).map_err(|e| e.to_string())?;
    let name = std::ffi::CString::new("shader.hlsl").unwrap();
    unsafe {
        let mut code: Option<ID3DBlob> = None;
        let mut errs: Option<ID3DBlob> = None;
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

fn upload_buffer(
    device: &ID3D11Device,
    bytes: &[u8],
    bind: u32,
    structured: bool,
    stride: u32,
) -> Result<ID3D11Buffer, String> {
    unsafe {
        let desc = D3D11_BUFFER_DESC {
            ByteWidth: bytes.len().max(16) as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: bind,
            CPUAccessFlags: 0,
            MiscFlags: if structured { D3D11_RESOURCE_MISC_BUFFER_STRUCTURED.0 as u32 } else { 0 },
            StructureByteStride: if structured { stride } else { 0 },
        };
        let init = D3D11_SUBRESOURCE_DATA {
            pSysMem: bytes.as_ptr() as *const _,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };
        let mut buf = None;
        device
            .CreateBuffer(&desc, Some(&init), Some(&mut buf))
            .map_err(|e| format!("CreateBuffer {e}"))?;
        buf.ok_or_else(|| "null buffer".into())
    }
}

fn structured_buffer(device: &ID3D11Device, bytes: &[u8]) -> Result<ID3D11Buffer, String> {
    let mut owned = bytes.to_vec();
    if owned.len() < 16 {
        owned.resize(16, 0);
    }
    // Structured buffers want a multiple of the stride.
    let rem = owned.len() % 16;
    if rem != 0 {
        owned.resize(owned.len() + (16 - rem), 0);
    }
    upload_buffer(
        device,
        &owned,
        (D3D11_BIND_UNORDERED_ACCESS.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
        true,
        16,
    )
}

fn color_target(device: &ID3D11Device, w: u32, h: u32) -> Result<ColorTarget, String> {
    unsafe {
        let desc = D3D11_TEXTURE2D_DESC {
            Width: w.max(1),
            Height: h.max(1),
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let mut tex = None;
        device
            .CreateTexture2D(&desc, None, Some(&mut tex))
            .map_err(|e| format!("color tex {e}"))?;
        let tex = tex.ok_or("null color tex")?;
        let resource: ID3D11Resource = tex.cast().map_err(|e| e.to_string())?;
        let mut rtv = None;
        device
            .CreateRenderTargetView(&resource, None, Some(&mut rtv))
            .map_err(|e| format!("rtv {e}"))?;
        Ok(ColorTarget {
            tex,
            rtv: rtv.ok_or("null rtv")?,
            w,
            h,
        })
    }
}

fn bind_ubo(ctx: &ID3D11DeviceContext, device: &ID3D11Device, ubo: Option<&[u8]>, slot: i32) -> Result<(), String> {
    if slot < 0 {
        return Ok(());
    }
    let src = ubo.unwrap_or(&[0u8; 16]);
    let mut bytes = src.to_vec();
    if bytes.len() < 16 {
        bytes.resize(16, 0);
    }
    let buf = upload_buffer(device, &bytes, D3D11_BIND_CONSTANT_BUFFER.0 as u32, false, 0)?;
    unsafe {
        let cbs = [Some(buf)];
        ctx.PSSetConstantBuffers(slot as u32, Some(&cbs));
        ctx.VSSetConstantBuffers(slot as u32, Some(&cbs));
    }
    Ok(())
}

fn bind_tex(
    ctx: &ID3D11DeviceContext,
    device: &ID3D11Device,
    tex: Option<&super::Tex>,
    samp: Option<&ID3D11SamplerState>,
) -> Result<(), String> {
    unsafe {
        let none: [Option<ID3D11ShaderResourceView>; 1] = [None];
        ctx.PSSetShaderResources(0, Some(&none));
    }
    let Some(tex) = tex else {
        return Ok(());
    };
    unsafe {
        let desc = D3D11_TEXTURE2D_DESC {
            Width: tex.w.max(1),
            Height: tex.h.max(1),
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let init = D3D11_SUBRESOURCE_DATA {
            pSysMem: tex.rgba.as_ptr() as *const _,
            SysMemPitch: tex.w.max(1) * 4,
            SysMemSlicePitch: 0,
        };
        let mut image = None;
        device
            .CreateTexture2D(&desc, Some(&init), Some(&mut image))
            .map_err(|e| format!("tex {e}"))?;
        let image = image.ok_or("null tex")?;
        let resource: ID3D11Resource = image.cast().map_err(|e| e.to_string())?;
        let mut srv = None;
        device
            .CreateShaderResourceView(&resource, None, Some(&mut srv))
            .map_err(|e| format!("srv {e}"))?;
        let srvs = [srv];
        ctx.PSSetShaderResources(0, Some(&srvs));
        if let Some(samp) = samp {
            let samplers = [Some(samp.clone())];
            ctx.PSSetSamplers(0, Some(&samplers));
        }
    }
    Ok(())
}

fn read_texture(
    device: &ID3D11Device,
    ctx: &ID3D11DeviceContext,
    tex: &ID3D11Texture2D,
    w: u32,
    h: u32,
) -> Result<Vec<u8>, String> {
    unsafe {
        let desc = D3D11_TEXTURE2D_DESC {
            Width: w.max(1),
            Height: h.max(1),
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut staging = None;
        device
            .CreateTexture2D(&desc, None, Some(&mut staging))
            .map_err(|e| format!("staging {e}"))?;
        let staging = staging.ok_or("null staging")?;
        let src: ID3D11Resource = tex.cast().map_err(|e| e.to_string())?;
        let dst: ID3D11Resource = staging.cast().map_err(|e| e.to_string())?;
        ctx.CopyResource(&dst, &src);
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        ctx.Map(&dst, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
            .map_err(|e| format!("map {e}"))?;
        let pitch = mapped.RowPitch as usize;
        let src_p = mapped.pData as *const u8;
        let mut out = vec![0u8; (w * h * 4) as usize];
        for y in 0..h as usize {
            let row = w as usize * 4;
            std::ptr::copy_nonoverlapping(src_p.add(y * pitch), out.as_mut_ptr().add(y * row), row);
        }
        ctx.Unmap(&dst, 0);
        Ok(out)
    }
}

fn read_buffer(device: &ID3D11Device, ctx: &ID3D11DeviceContext, buf: &ID3D11Buffer, n: usize) -> Result<Vec<u8>, String> {
    unsafe {
        let desc = D3D11_BUFFER_DESC {
            ByteWidth: n.max(16) as u32,
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
            StructureByteStride: 0,
        };
        let mut staging = None;
        device
            .CreateBuffer(&desc, None, Some(&mut staging))
            .map_err(|e| format!("stage buf {e}"))?;
        let staging = staging.ok_or("null stage buf")?;
        let src: ID3D11Resource = buf.cast().map_err(|e| e.to_string())?;
        let dst: ID3D11Resource = staging.cast().map_err(|e| e.to_string())?;
        ctx.CopyResource(&dst, &src);
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        ctx.Map(&dst, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
            .map_err(|e| format!("map buf {e}"))?;
        let bytes = std::slice::from_raw_parts(mapped.pData as *const u8, n).to_vec();
        ctx.Unmap(&dst, 0);
        Ok(bytes)
    }
}
