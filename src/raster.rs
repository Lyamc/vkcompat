//! CPU triangle rasterizer. Clip space matches Vulkan: x right, y down, z 0..1.

use crate::spirv::{self, ImageBind, Io, Module};
use crate::vk::{self, format_bpp, FORMAT_B8G8R8A8_UNORM};

#[derive(Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub zmin: f32,
    pub zmax: f32,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

struct Vert {
    clip: [f32; 4],
    screen: [f32; 2],
    z: f32,
    vary: Vec<[f32; 4]>,
}

pub struct FragTarget<'a> {
    pub pixels: &'a mut [u8],
    pub width: u32,
    pub height: u32,
    pub format: i32,
}

pub fn draw_triangle(
    vs: &Module,
    fs: &Module,
    verts: &[u8],
    stride: u32,
    vertex_count: u32,
    first_vertex: u32,
    attrs: &[(u32, u32, i32)],
    viewport: Viewport,
    scissor: Rect,
    ubos: &[(u32, u32, &[u8])],
    images: &[ImageBind],
    target: &mut FragTarget,
) -> Result<(), String> {
    if vertex_count < 3 || stride == 0 {
        return Ok(());
    }
    let tri_count = vertex_count / 3;
    for t in 0..tri_count {
        let mut shaded = Vec::new();
        for k in 0..3 {
            let index = first_vertex + t * 3 + k;
            let mut inputs = vec![[0.0, 0.0, 0.0, 1.0]; 8];
            for (loc, off, fmt) in attrs {
                if let Some(slot) = inputs.get_mut(*loc as usize) {
                    *slot = fetch(verts, stride, index, *off, *fmt);
                }
            }
            let mut outputs = Vec::new();
            let mut position = [0.0, 0.0, 0.0, 1.0];
            {
                let mut io = Io {
                    attrs: &inputs,
                    frag_coord: [0.0; 4],
                    vertex_index: index as i32,
                    instance_index: 0,
                    global_id: [0; 3],
                    ubos,
                    images,
                    ssbo: &mut [],
                    ssbo_set: 0,
                    ssbo_binding: 0,
                    position: [0.0; 4],
                    outputs: Vec::new(),
                };
                spirv::execute(vs, &mut io)?;
                position = io.position;
                outputs = io.outputs;
            }
            let (sx, sy, sz) = to_window(position, viewport);
            let mut vary = vec![[0.0, 0.0, 0.0, 1.0]; 8];
            for (loc, v) in outputs {
                if (loc as usize) < vary.len() {
                    vary[loc as usize] = v;
                }
            }
            shaded.push(Vert {
                clip: position,
                screen: [sx, sy],
                z: sz,
                vary,
            });
        }
        fill_tri(&shaded, fs, viewport, scissor, ubos, images, target)?;
    }
    Ok(())
}

fn fetch(verts: &[u8], stride: u32, index: u32, offset: u32, fmt: i32) -> [f32; 4] {
    let base = index as usize * stride as usize + offset as usize;
    let mut o = [0.0, 0.0, 0.0, 1.0];
    let n = match fmt {
        vk::FORMAT_R32G32_SFLOAT => 2,
        vk::FORMAT_R32G32B32_SFLOAT => 3,
        vk::FORMAT_R32G32B32A32_SFLOAT => 4,
        vk::FORMAT_R32_SFLOAT => 1,
        _ => 0,
    };
    for i in 0..n {
        let s = base + i * 4;
        if s + 4 <= verts.len() {
            o[i] = f32::from_le_bytes(verts[s..s + 4].try_into().unwrap());
        }
    }
    o
}

fn to_window(clip: [f32; 4], vp: Viewport) -> (f32, f32, f32) {
    let w = if clip[3].abs() < 1e-20 { 1.0 } else { clip[3] };
    let ndc_x = clip[0] / w;
    let ndc_y = clip[1] / w;
    let ndc_z = clip[2] / w;
    let x = vp.x + (ndc_x + 1.0) * 0.5 * vp.w;
    let y = vp.y + (ndc_y + 1.0) * 0.5 * vp.h;
    let z = vp.zmin + ndc_z * (vp.zmax - vp.zmin);
    (x, y, z)
}

fn orient(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
}

fn fill_tri(
    v: &[Vert],
    fs: &Module,
    _viewport: Viewport,
    scissor: Rect,
    ubos: &[(u32, u32, &[u8])],
    images: &[ImageBind],
    target: &mut FragTarget,
) -> Result<(), String> {
    if v.len() < 3 {
        return Ok(());
    }
    let s0 = v[0].screen;
    let s1 = v[1].screen;
    let s2 = v[2].screen;
    let area = orient(s0, s1, s2);
    if area.abs() < 1e-8 {
        return Ok(());
    }
    let min_x = s0[0].min(s1[0]).min(s2[0]).floor() as i32;
    let max_x = s0[0].max(s1[0]).max(s2[0]).ceil() as i32;
    let min_y = s0[1].min(s1[1]).min(s2[1]).floor() as i32;
    let max_y = s0[1].max(s1[1]).max(s2[1]).ceil() as i32;
    let sx0 = scissor.x.max(0);
    let sy0 = scissor.y.max(0);
    let sx1 = (scissor.x + scissor.w).min(target.width as i32);
    let sy1 = (scissor.y + scissor.h).min(target.height as i32);
    let bpp = format_bpp(target.format).max(4) as usize;
    for py in min_y.max(sy0)..max_y.min(sy1) {
        for px in min_x.max(sx0)..max_x.min(sx1) {
            let p = [px as f32 + 0.5, py as f32 + 0.5];
            let mut w0 = orient(s1, s2, p);
            let mut w1 = orient(s2, s0, p);
            let mut w2 = orient(s0, s1, p);
            if area < 0.0 {
                w0 = -w0;
                w1 = -w1;
                w2 = -w2;
            }
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let inv_area = 1.0 / (w0 + w1 + w2);
            let b0 = w0 * inv_area;
            let b1 = w1 * inv_area;
            let b2 = w2 * inv_area;
            let mut vary = vec![[0.0, 0.0, 0.0, 1.0]; 8];
            for loc in 0..8 {
                for c in 0..4 {
                    vary[loc][c] = b0 * v[0].vary[loc][c] + b1 * v[1].vary[loc][c] + b2 * v[2].vary[loc][c];
                }
            }
            let z = b0 * v[0].z + b1 * v[1].z + b2 * v[2].z;
            let clip_w = b0 * v[0].clip[3] + b1 * v[1].clip[3] + b2 * v[2].clip[3];
            let mut io = Io {
                attrs: &vary,
                frag_coord: [p[0], p[1], z, if clip_w.abs() < 1e-20 { 1.0 } else { 1.0 / clip_w }],
                vertex_index: 0,
                instance_index: 0,
                global_id: [0; 3],
                ubos,
                images,
                ssbo: &mut [],
                ssbo_set: 0,
                ssbo_binding: 0,
                position: [0.0; 4],
                outputs: Vec::new(),
            };
            spirv::execute(fs, &mut io)?;
            let mut color = [0.0, 0.0, 0.0, 1.0];
            for (loc, c) in io.outputs {
                if loc == 0 {
                    color = c;
                }
            }
            write_pixel(target, px as u32, py as u32, color, bpp);
        }
    }
    Ok(())
}

pub fn clear(target: &mut FragTarget, rgba: [f32; 4]) {
    let bpp = format_bpp(target.format) as usize;
    if bpp == 0 {
        return;
    }
    let px = pack(target.format, rgba);
    let n = (target.width * target.height) as usize;
    for i in 0..n {
        let s = i * bpp;
        if s + px.len() <= target.pixels.len() {
            target.pixels[s..s + px.len()].copy_from_slice(&px);
        }
    }
}

pub fn pack(fmt: i32, rgba: [f32; 4]) -> Vec<u8> {
    let u = |c: f32| (c * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
    let r = u(rgba[0]);
    let g = u(rgba[1]);
    let b = u(rgba[2]);
    let a = u(rgba[3]);
    if fmt == FORMAT_B8G8R8A8_UNORM {
        vec![b, g, r, a]
    } else if fmt == vk::FORMAT_R32G32B32A32_SFLOAT {
        let mut o = Vec::with_capacity(16);
        for c in rgba {
            o.extend_from_slice(&c.to_le_bytes());
        }
        o
    } else {
        vec![r, g, b, a]
    }
}

fn write_pixel(target: &mut FragTarget, x: u32, y: u32, rgba: [f32; 4], bpp: usize) {
    if x >= target.width || y >= target.height {
        return;
    }
    let packed = pack(target.format, rgba);
    let s = (y * target.width + x) as usize * bpp;
    let n = packed.len().min(bpp);
    if s + n <= target.pixels.len() {
        target.pixels[s..s + n].copy_from_slice(&packed[..n]);
    }
}

pub fn dispatch_compute(
    cs: &Module,
    groups: [u32; 3],
    ssbo: &mut [u8],
    set: u32,
    binding: u32,
) -> Result<(), String> {
    let local = cs.local_size;
    for gz in 0..groups[2] {
        for gy in 0..groups[1] {
            for gx in 0..groups[0] {
                for lz in 0..local[2] {
                    for ly in 0..local[1] {
                        for lx in 0..local[0] {
                            let gid = [gx * local[0] + lx, gy * local[1] + ly, gz * local[2] + lz];
                            let mut io = Io {
                                attrs: &[],
                                frag_coord: [0.0; 4],
                                vertex_index: 0,
                                instance_index: 0,
                                global_id: gid,
                                ubos: &[],
                                images: &[],
                                ssbo,
                                ssbo_set: set,
                                ssbo_binding: binding,
                                position: [0.0; 4],
                                outputs: Vec::new(),
                            };
                            spirv::execute(cs, &mut io)?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
