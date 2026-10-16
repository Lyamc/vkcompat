//! OpenGL 1/2/3 backend. Tier 3 is GLSL 330, tier 2 is GLSL 120, tier 1 is fixed function.
//! No compute and no GL 4 features. A 4.x context created from a 3.3 request still runs as its tier.

#[path = "gl_native.rs"]
mod gl_native;

use super::{device_type_for_vendor, Backend, DispatchDesc, DrawDesc, FixedDesc, PhysDesc, Tex, Tier};
use crate::emit::Dialect;
use gl_native::GlNative;
use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};

pub(super) const GL_VERSION: u32 = 0x1F02;
const GL_VENDOR: u32 = 0x1F00;
const GL_DEPTH_TEST: u32 = 0x0B71;
const GL_CULL_FACE: u32 = 0x0B44;
const GL_BLEND: u32 = 0x0BE2;
const GL_LIGHTING: u32 = 0x0B50;
const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
const GL_TRIANGLES: u32 = 0x0004;
const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_RGBA: u32 = 0x1908;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
const GL_FLOAT: u32 = 0x1406;
const GL_NEAREST: i32 = 0x2600;
const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_TEXTURE_ENV: u32 = 0x2300;
const GL_TEXTURE_ENV_MODE: u32 = 0x2200;
const GL_MODELVIEW: u32 = 0x1700;
const GL_PROJECTION: u32 = 0x1701;
const GL_PACK_ALIGNMENT: u32 = 0x0D05;
const GL_UNPACK_ALIGNMENT: u32 = 0x0CF5;
const GL_SCISSOR_TEST: u32 = 0x0C11;

const GL_FRAGMENT_SHADER: u32 = 0x8B30;
const GL_VERTEX_SHADER: u32 = 0x8B31;
const GL_COMPILE_STATUS: i32 = 0x8B81;
const GL_LINK_STATUS: i32 = 0x8B82;
const GL_INFO_LOG_LENGTH: i32 = 0x8B84;
const GL_ARRAY_BUFFER: u32 = 0x8892;
const GL_STATIC_DRAW: u32 = 0x88E4;
const GL_FRAMEBUFFER: u32 = 0x8D40;
const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
const GL_FRAMEBUFFER_COMPLETE: u32 = 0x8CD5;
const GL_RGBA8: i32 = 0x8058;
const GL_CLAMP_TO_EDGE: i32 = 0x812F;
const GL_TEXTURE0: u32 = 0x84C0;
const GL_UNIFORM_BUFFER: u32 = 0x8A11;
const GL_UNIFORM_BUFFER_OFFSET_ALIGNMENT: i32 = 0x8A34;
const GL_INVALID_INDEX: u32 = 0xFFFF_FFFF;
const GL_REPLACE: i32 = 0x1E01;

type GlCreateShader = unsafe extern "system" fn(u32) -> u32;
type GlShaderSource = unsafe extern "system" fn(u32, i32, *const *const i8, *const i32);
type GlCompile = unsafe extern "system" fn(u32);
type GlGetShaderiv = unsafe extern "system" fn(u32, i32, *mut i32);
type GlGetInfoLog = unsafe extern "system" fn(u32, i32, *mut i32, *mut i8);
type GlCreateProgram = unsafe extern "system" fn() -> u32;
type GlAttach = unsafe extern "system" fn(u32, u32);
type GlLink = unsafe extern "system" fn(u32);
type GlUse = unsafe extern "system" fn(u32);
type GlDeleteShader = unsafe extern "system" fn(u32);
type GlDeleteProgram = unsafe extern "system" fn(u32);
type GlBindAttrib = unsafe extern "system" fn(u32, u32, *const i8);
type GlBindFrag = unsafe extern "system" fn(u32, u32, *const i8);
type GlGetUniform = unsafe extern "system" fn(u32, *const i8) -> i32;
type GlUniform4fv = unsafe extern "system" fn(i32, i32, *const f32);
type GlUniform1i = unsafe extern "system" fn(i32, i32);
type GlGetBlock = unsafe extern "system" fn(u32, *const i8) -> u32;
type GlBlockBinding = unsafe extern "system" fn(u32, u32, u32);
type GlGenBuf = unsafe extern "system" fn(i32, *mut u32);
type GlBindBuf = unsafe extern "system" fn(u32, u32);
type GlBufData = unsafe extern "system" fn(u32, isize, *const c_void, u32);
type GlBindBase = unsafe extern "system" fn(u32, u32, u32);
type GlEnableAttrib = unsafe extern "system" fn(u32);
type GlDisableAttrib = unsafe extern "system" fn(u32);
type GlAttribPtr = unsafe extern "system" fn(u32, i32, u32, u8, i32, *const c_void);
type GlDrawArrays = unsafe extern "system" fn(u32, i32, i32);
type GlGenFb = unsafe extern "system" fn(i32, *mut u32);
type GlBindFb = unsafe extern "system" fn(u32, u32);
type GlFbTex = unsafe extern "system" fn(u32, u32, u32, u32, i32);
type GlCheckFb = unsafe extern "system" fn(u32) -> u32;
type GlActiveTex = unsafe extern "system" fn(u32);
type GlGetIntegerv = unsafe extern "system" fn(i32, *mut i32);
type GlGenVao = unsafe extern "system" fn(i32, *mut u32);
type GlBindVao = unsafe extern "system" fn(u32);
type GlDeleteTextures = unsafe extern "system" fn(i32, *const u32);

type GlPixelStore = unsafe extern "system" fn(u32, i32);
type GlGetString = unsafe extern "system" fn(u32) -> *const u8;
type GlGenTex = unsafe extern "system" fn(i32, *mut u32);
type GlBindTex = unsafe extern "system" fn(u32, u32);
type GlTexParam = unsafe extern "system" fn(u32, u32, i32);
type GlTexImage = unsafe extern "system" fn(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
type GlViewport = unsafe extern "system" fn(i32, i32, i32, i32);
type GlToggle = unsafe extern "system" fn(u32);
type GlClearColor = unsafe extern "system" fn(f32, f32, f32, f32);
type GlClear = unsafe extern "system" fn(u32);
type GlFinish = unsafe extern "system" fn();
type GlReadPixels = unsafe extern "system" fn(i32, i32, i32, i32, u32, u32, *mut c_void);
type GlMatrixMode = unsafe extern "system" fn(u32);
type GlLoadIdentity = unsafe extern "system" fn();
type GlTexEnvi = unsafe extern "system" fn(u32, u32, i32);
type GlColor4f = unsafe extern "system" fn(f32, f32, f32, f32);
type GlBegin = unsafe extern "system" fn(u32);
type GlTexCoord = unsafe extern "system" fn(f32, f32);
type GlVertex4 = unsafe extern "system" fn(f32, f32, f32, f32);
type GlEnd = unsafe extern "system" fn();

#[derive(Clone, Copy)]
struct CoreGl {
    pixel_storei: GlPixelStore,
    get_string: GlGetString,
    gen_textures: GlGenTex,
    bind_texture: GlBindTex,
    tex_parameteri: GlTexParam,
    tex_image_2d: GlTexImage,
    viewport: GlViewport,
    disable: GlToggle,
    clear_color: GlClearColor,
    clear: GlClear,
    finish: GlFinish,
    read_pixels: GlReadPixels,
    enable: Option<GlToggle>,
    matrix_mode: Option<GlMatrixMode>,
    load_identity: Option<GlLoadIdentity>,
    tex_envi: Option<GlTexEnvi>,
    color4f: Option<GlColor4f>,
    begin: Option<GlBegin>,
    tex_coord2f: Option<GlTexCoord>,
    vertex4f: Option<GlVertex4>,
    end: Option<GlEnd>,
}

struct Procs {
    create_shader: GlCreateShader,
    shader_source: GlShaderSource,
    compile: GlCompile,
    get_shaderiv: GlGetShaderiv,
    get_shader_log: GlGetInfoLog,
    create_program: GlCreateProgram,
    attach: GlAttach,
    link: GlLink,
    use_program: GlUse,
    delete_shader: GlDeleteShader,
    get_programiv: GlGetShaderiv,
    get_program_log: GlGetInfoLog,
    bind_attrib: GlBindAttrib,
    bind_frag: Option<GlBindFrag>,
    get_uniform: GlGetUniform,
    uniform4fv: GlUniform4fv,
    uniform1i: GlUniform1i,
    get_block: Option<GlGetBlock>,
    block_binding: Option<GlBlockBinding>,
    gen_buffers: GlGenBuf,
    bind_buffer: GlBindBuf,
    buffer_data: GlBufData,
    bind_buffer_base: Option<GlBindBase>,
    enable_attrib: GlEnableAttrib,
    disable_attrib: GlDisableAttrib,
    attrib_ptr: GlAttribPtr,
    draw_arrays: GlDrawArrays,
    gen_fb: Option<GlGenFb>,
    bind_fb: Option<GlBindFb>,
    fb_tex: Option<GlFbTex>,
    check_fb: Option<GlCheckFb>,
    active_tex: Option<GlActiveTex>,
    get_integerv: GlGetIntegerv,
    gen_vao: Option<GlGenVao>,
    bind_vao: Option<GlBindVao>,
    delete_textures: Option<GlDeleteTextures>,
}

struct Prog {
    id: u32,
    ubo_loc: i32,
}

pub struct Gl {
    native: GlNative,
    tier_n: u32,
    core: CoreGl,
    procs: Option<Procs>,
    programs: HashMap<usize, Prog>,
    vao: u32,
}

pub fn enumerate() -> Vec<PhysDesc> {
    let probed = match probe_version() {
        Some(v) => v,
        None => return Vec::new(),
    };
    let best = if probed.0 > 3 || (probed.0 == 3 && probed.1 >= 3) {
        3
    } else if probed.0 > 2 || (probed.0 == 2 && probed.1 >= 0) {
        2
    } else {
        1
    };
    let mut tiers = Vec::new();
    if let Some(force) = forced_tier() {
        if force <= best {
            tiers.push(force);
        }
    } else if all_tiers() {
        for t in [3u32, 2, 1] {
            if t <= best {
                tiers.push(t);
            }
        }
    } else {
        tiers.push(best);
    }
    let mut out = Vec::new();
    for tier in tiers {
        if let Some(desc) = describe_tier(tier) {
            out.push(desc);
        }
    }
    out
}

fn all_tiers() -> bool {
    std::env::var("VKCOMPAT_ALL_TIERS").map(|v| v == "1").unwrap_or(false)
        || std::env::var("VKGL_ALL_TIERS").map(|v| v == "1").unwrap_or(false)
}

fn forced_tier() -> Option<u32> {
    for name in ["VKCOMPAT_TIER", "VKGL_TIER"] {
        if let Ok(v) = std::env::var(name) {
            if let Ok(n) = v.parse::<u32>() {
                if (1..=3).contains(&n) {
                    return Some(n);
                }
            }
        }
    }
    None
}

fn probe_version() -> Option<(u32, u32, u32)> {
    let gl = Gl::create(1).ok()?;
    let v = gl.version();
    let vendor = gl.vendor_id();
    drop(gl);
    Some((v.0, v.1, vendor))
}

fn describe_tier(tier: u32) -> Option<PhysDesc> {
    let gl = Gl::create(tier).ok()?;
    let (maj, min) = gl.version();
    let vendor = gl.vendor_id();
    drop(gl);
    Some(PhysDesc {
        name: format!("vkcompat OpenGL {maj}.{min} tier {tier}"),
        kind: super::Kind::Gl,
        vendor,
        device: 0,
        device_type: device_type_for_vendor(vendor),
        adapter: 0,
        gl_tier: tier,
        fl11: false,
        gl_major: maj,
        gl_minor: min,
    })
}

pub fn open(tier: u32) -> Result<Gl, String> {
    Gl::create(tier)
}

impl Gl {
    fn create(tier: u32) -> Result<Gl, String> {
        let native = GlNative::create(tier)?;
        let core = match load_core() {
            Ok(c) => c,
            Err(e) => return Err(e),
        };
        unsafe {
            (core.pixel_storei)(GL_UNPACK_ALIGNMENT, 1);
            (core.pixel_storei)(GL_PACK_ALIGNMENT, 1);
        }
        let want_procs = tier >= 2 || cfg!(target_os = "macos");
        let procs = if want_procs {
            match load_procs(if tier >= 2 { tier } else { 2 }) {
                Ok(p) => Some(p),
                Err(e) => {
                    if tier >= 2 {
                        return Err(e);
                    }
                    None
                }
            }
        } else {
            None
        };
        if tier >= 2 {
            let p = procs.as_ref().unwrap();
            if p.gen_fb.is_none() || p.create_shader as usize == 0 {
                return Err("OpenGL shader entry points are missing".into());
            }
        }
        let mut vao = 0u32;
        if let Some(p) = procs.as_ref() {
            if let Some(gen) = p.gen_vao {
                unsafe { gen(1, &mut vao) };
            }
        }
        Ok(Gl {
            native,
            tier_n: tier,
            core,
            procs,
            programs: HashMap::new(),
            vao,
        })
    }

    fn current(&self) {
        self.native.make_current();
    }

    fn version(&self) -> (u32, u32) {
        self.current();
        unsafe { parse_version((self.core.get_string)(GL_VERSION)) }
    }

    fn vendor_id(&self) -> u32 {
        self.current();
        unsafe {
            let p = (self.core.get_string)(GL_VENDOR);
            if p.is_null() {
                return 0;
            }
            let s = CStr::from_ptr(p as *const i8).to_string_lossy().to_lowercase();
            if s.contains("nvidia") {
                0x10DE
            } else if s.contains("amd") || s.contains("ati") {
                0x1002
            } else if s.contains("intel") {
                0x8086
            } else {
                0
            }
        }
    }

    fn ensure_target(&mut self, w: u32, h: u32) -> Result<Target, String> {
        self.current();
        let core = self.core;
        let fbo = {
            let Some(p) = self.procs.as_ref() else {
                return self.window_target(w, h);
            };
            match (p.gen_fb, p.bind_fb, p.fb_tex, p.check_fb) {
                (Some(gen_fb), Some(bind_fb), Some(fb_tex), Some(check)) => Some((gen_fb, bind_fb, fb_tex, check)),
                _ => None,
            }
        };
        let Some((gen_fb, bind_fb, fb_tex, check)) = fbo else {
            return self.window_target(w, h);
        };
        unsafe {
            let mut fbo_id = 0u32;
            let mut tex = 0u32;
            gen_fb(1, &mut fbo_id);
            bind_fb(GL_FRAMEBUFFER, fbo_id);
            (core.gen_textures)(1, &mut tex);
            (core.bind_texture)(GL_TEXTURE_2D, tex);
            (core.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
            (core.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
            let internal = if self.tier_n >= 3 { GL_RGBA8 } else { GL_RGBA as i32 };
            (core.tex_image_2d)(
                GL_TEXTURE_2D,
                0,
                internal,
                w as i32,
                h as i32,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null(),
            );
            fb_tex(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, tex, 0);
            if check(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE && internal != GL_RGBA8 {
                (core.tex_image_2d)(
                    GL_TEXTURE_2D,
                    0,
                    GL_RGBA8,
                    w as i32,
                    h as i32,
                    0,
                    GL_RGBA,
                    GL_UNSIGNED_BYTE,
                    std::ptr::null(),
                );
            }
            if check(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE {
                if self.tier_n <= 1 {
                    return self.window_target(w, h);
                }
                return Err("framebuffer is incomplete".into());
            }
            Ok(Target { fbo: fbo_id, tex, window: false })
        }
    }

    fn window_target(&mut self, w: u32, h: u32) -> Result<Target, String> {
        self.native.resize(w as i32, h as i32);
        if let Some(p) = self.procs.as_ref() {
            if let Some(bind) = p.bind_fb {
                unsafe { bind(GL_FRAMEBUFFER, 0) };
            }
        }
        Ok(Target { fbo: 0, tex: 0, window: true })
    }

    fn program(&mut self, key: usize, vs: &str, fs: &str, ubo_in_fs: bool) -> Result<u32, String> {
        if let Some(p) = self.programs.get(&key) {
            return Ok(p.id);
        }
        let procs = self.procs.as_ref().ok_or("no GL shader procs")?;
        unsafe {
            let vs_id = compile_one(procs, GL_VERTEX_SHADER, vs)?;
            let fs_id = compile_one(procs, GL_FRAGMENT_SHADER, fs)?;
            let prog = (procs.create_program)();
            if self.tier_n < 3 {
                let name = CString::new("a0").unwrap();
                (procs.bind_attrib)(prog, 0, name.as_ptr());
            }
            if let Some(bind_frag) = procs.bind_frag {
                let name = CString::new("color0_out").unwrap();
                bind_frag(prog, 0, name.as_ptr());
            }
            (procs.attach)(prog, vs_id);
            (procs.attach)(prog, fs_id);
            (procs.link)(prog);
            let mut status = 0i32;
            (procs.get_programiv)(prog, GL_LINK_STATUS, &mut status);
            if status == 0 {
                let log = program_log(procs, prog);
                return Err(format!("link failed\n{log}\nVS:\n{vs}\nFS:\n{fs}"));
            }
            (procs.delete_shader)(vs_id);
            (procs.delete_shader)(fs_id);
            if self.tier_n >= 3 {
                if let (Some(get_block), Some(bind)) = (procs.get_block, procs.block_binding) {
                    let name = CString::new("C1").unwrap();
                    let index = get_block(prog, name.as_ptr());
                    if index != GL_INVALID_INDEX {
                        bind(prog, index, 1);
                    }
                }
            }
            let ubo_loc = if self.tier_n < 3 && ubo_in_fs {
                let name = CString::new("U1").unwrap();
                (procs.get_uniform)(prog, name.as_ptr())
            } else {
                -1
            };
            self.programs.insert(key, Prog { id: prog, ubo_loc });
            Ok(prog)
        }
    }
}

impl Drop for Gl {
    fn drop(&mut self) {
        self.native.make_current();
        if let Some(p) = self.procs.as_ref() {
            for prog in self.programs.values() {
                if let Some(del) = p.delete_program_fn() {
                    unsafe { del(prog.id) };
                }
            }
        }
        self.programs.clear();
    }
}

impl Procs {
    fn delete_program_fn(&self) -> Option<GlDeleteProgram> {
        None
    }
}

struct Target {
    fbo: u32,
    tex: u32,
    window: bool,
}

impl Backend for Gl {
    fn tier(&self) -> Tier {
        let dialect = match self.tier_n {
            3 => Dialect::Glsl330,
            2 => Dialect::Glsl120,
            _ => Dialect::Fixed,
        };
        Tier {
            dialect,
            fixed_function: self.tier_n == 1,
            compute: false,
            fan: true,
            instancing: self.tier_n >= 3,
            flip_y: true,
            clip_y_up: false,
        }
    }

    fn draw(&mut self, d: &DrawDesc) -> Result<Vec<u8>, String> {
        self.current();
        let prog = self.program(d.key, d.vs, d.fs, d.ubo_slot >= 0)?;
        let target = self.ensure_target(d.width, d.height)?;
        let procs = self.procs.as_ref().ok_or("no procs")?.clone_fns();
        let core = self.core;
        unsafe {
            if let Some(bind) = procs.bind_fb {
                bind(GL_FRAMEBUFFER, target.fbo);
            }
            if let Some(bind_vao) = procs.bind_vao {
                bind_vao(self.vao);
            }
            (core.viewport)(0, 0, d.width as i32, d.height as i32);
            (core.disable)(GL_SCISSOR_TEST);
            (core.clear_color)(d.clear[0], d.clear[1], d.clear[2], d.clear[3]);
            (core.clear)(GL_COLOR_BUFFER_BIT);
            (core.disable)(GL_DEPTH_TEST);
            (core.disable)(GL_CULL_FACE);
            (core.disable)(GL_BLEND);
            (procs.use_program)(prog);
            for i in 0..8 {
                (procs.disable_attrib)(i);
            }
            let mut vbo = 0u32;
            (procs.gen_buffers)(1, &mut vbo);
            (procs.bind_buffer)(GL_ARRAY_BUFFER, vbo);
            (procs.buffer_data)(
                GL_ARRAY_BUFFER,
                d.verts.len() as isize,
                d.verts.as_ptr() as *const _,
                GL_STATIC_DRAW,
            );
            let comps = d.components.max(1) as i32;
            (procs.enable_attrib)(0);
            (procs.attrib_ptr)(0, comps, GL_FLOAT, 0, d.stride as i32, std::ptr::null());
            if d.ubo_slot >= 0 {
                if let Some(bytes) = d.ubo {
                    bind_ubo(self, prog, d.ubo_slot, bytes)?;
                }
            }
            if let Some(tex) = d.tex.as_ref() {
                bind_gl_tex(self, prog, tex)?;
            }
            (procs.draw_arrays)(GL_TRIANGLES, 0, d.count as i32);
            (core.finish)();
            let pixels = read_color(&core, d.width, d.height);
            if let Some(bind) = procs.bind_fb {
                bind(GL_FRAMEBUFFER, 0);
            }
            let _ = target;
            Ok(pixels)
        }
    }

    fn draw_fixed(&mut self, d: &FixedDesc) -> Result<Vec<u8>, String> {
        self.current();
        if d.clips.len() < 3 {
            return Err("fixed function has no vertices".into());
        }
        if d.textured && d.tex.is_none() {
            return Err("fixed function texture is missing".into());
        }
        let begin = self.core.begin.ok_or("fixed function entry points are missing")?;
        let end = self.core.end.ok_or("fixed function entry points are missing")?;
        let vertex4f = self.core.vertex4f.ok_or("fixed function entry points are missing")?;
        let tex_coord2f = self.core.tex_coord2f.ok_or("fixed function entry points are missing")?;
        let color4f = self.core.color4f.ok_or("fixed function entry points are missing")?;
        let matrix_mode = self.core.matrix_mode.ok_or("fixed function entry points are missing")?;
        let load_identity = self.core.load_identity.ok_or("fixed function entry points are missing")?;
        let tex_envi = self.core.tex_envi.ok_or("fixed function entry points are missing")?;
        let enable = self.core.enable.ok_or("fixed function entry points are missing")?;
        let target = self.ensure_target(d.width, d.height)?;
        let core = self.core;
        unsafe {
            if let Some(p) = self.procs.as_ref() {
                if let Some(bind) = p.bind_fb {
                    bind(GL_FRAMEBUFFER, if target.window { 0 } else { target.fbo });
                }
                if let Some(bind_vao) = p.bind_vao {
                    bind_vao(0);
                }
                (p.use_program)(0);
                for i in 0..8 {
                    (p.disable_attrib)(i);
                }
            }
            (core.viewport)(0, 0, d.width as i32, d.height as i32);
            (core.disable)(GL_DEPTH_TEST);
            (core.disable)(GL_CULL_FACE);
            (core.disable)(GL_BLEND);
            (core.disable)(GL_LIGHTING);
            matrix_mode(GL_PROJECTION);
            load_identity();
            matrix_mode(GL_MODELVIEW);
            load_identity();
            (core.clear_color)(d.clear[0], d.clear[1], d.clear[2], d.clear[3]);
            (core.clear)(GL_COLOR_BUFFER_BIT);
            let mut tex_id = 0u32;
            if d.textured {
                if let Some(tex) = d.tex.as_ref() {
                    tex_id = upload_ff_tex(&core, tex);
                    enable(GL_TEXTURE_2D);
                    (core.bind_texture)(GL_TEXTURE_2D, tex_id);
                    tex_envi(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_REPLACE);
                }
            } else {
                (core.disable)(GL_TEXTURE_2D);
                color4f(d.color[0], d.color[1], d.color[2], d.color[3]);
            }
            begin(GL_TRIANGLES);
            let n = d.clips.len() / 3 * 3;
            for i in 0..n {
                let c = d.clips[i];
                let uv = d.uvs.get(i).copied().unwrap_or([0.0, 0.0]);
                if d.textured {
                    tex_coord2f(uv[0], 1.0 - uv[1]);
                }
                vertex4f(c[0], -c[1], c[2] * 2.0 - c[3], c[3]);
            }
            end();
            (core.finish)();
            if tex_id != 0 {
                (core.bind_texture)(GL_TEXTURE_2D, 0);
            }
            Ok(read_color(&core, d.width, d.height))
        }
    }

    fn dispatch(&mut self, _d: &DispatchDesc) -> Result<Vec<u8>, String> {
        Err("compute is software".into())
    }
}

fn bind_ubo(gl: &Gl, prog: u32, slot: i32, bytes: &[u8]) -> Result<(), String> {
    let procs = gl.procs.as_ref().ok_or("no procs")?;
    unsafe {
        if gl.tier_n >= 3 {
            let bind_base = procs.bind_buffer_base.ok_or("glBindBufferBase missing")?;
            let mut align = 256i32;
            (procs.get_integerv)(GL_UNIFORM_BUFFER_OFFSET_ALIGNMENT, &mut align);
            let align = (align.max(16) as usize).max(16);
            let mut padded = vec![0u8; align.max(bytes.len())];
            let n = bytes.len().min(padded.len());
            padded[..n].copy_from_slice(&bytes[..n]);
            let mut buf = 0u32;
            (procs.gen_buffers)(1, &mut buf);
            (procs.bind_buffer)(GL_UNIFORM_BUFFER, buf);
            (procs.buffer_data)(
                GL_UNIFORM_BUFFER,
                padded.len() as isize,
                padded.as_ptr() as *const _,
                GL_STATIC_DRAW,
            );
            bind_base(GL_UNIFORM_BUFFER, slot.max(0) as u32, buf);
        } else {
            let prog_info = gl.programs.values().find(|p| p.id == prog);
            let loc = prog_info.map(|p| p.ubo_loc).unwrap_or(-1);
            if loc >= 0 {
                let mut regs = [0f32; 4];
                for i in 0..4 {
                    let s = i * 4;
                    if s + 4 <= bytes.len() {
                        regs[i] = f32::from_le_bytes(bytes[s..s + 4].try_into().unwrap());
                    }
                }
                (procs.uniform4fv)(loc, 1, regs.as_ptr());
            }
        }
    }
    Ok(())
}

fn bind_gl_tex(gl: &Gl, prog: u32, tex: &Tex) -> Result<(), String> {
    let procs = gl.procs.as_ref().ok_or("no procs")?;
    let core = gl.core;
    unsafe {
        if let Some(active) = procs.active_tex {
            active(GL_TEXTURE0);
        }
        let id = upload_ff_tex(&core, tex);
        (core.bind_texture)(GL_TEXTURE_2D, id);
        let name = CString::new("tex0").unwrap();
        let loc = (procs.get_uniform)(prog, name.as_ptr());
        if loc >= 0 {
            (procs.uniform1i)(loc, 0);
        }
    }
    Ok(())
}

fn upload_ff_tex(core: &CoreGl, tex: &Tex) -> u32 {
    unsafe {
        let mut id = 0u32;
        (core.gen_textures)(1, &mut id);
        (core.bind_texture)(GL_TEXTURE_2D, id);
        (core.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
        (core.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
        (core.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
        (core.tex_parameteri)(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        let w = tex.w.max(1);
        let h = tex.h.max(1);
        let row = (w * 4) as usize;
        let mut flipped = vec![0u8; (w * h * 4) as usize];
        for y in 0..h as usize {
            let src = y * row;
            let dst = (h as usize - 1 - y) * row;
            let n = row.min(tex.rgba.len().saturating_sub(src));
            if n > 0 && src + n <= tex.rgba.len() && dst + n <= flipped.len() {
                flipped[dst..dst + n].copy_from_slice(&tex.rgba[src..src + n]);
            }
        }
        (core.tex_image_2d)(
            GL_TEXTURE_2D,
            0,
            GL_RGBA as i32,
            w as i32,
            h as i32,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            flipped.as_ptr() as *const _,
        );
        id
    }
}

fn read_color(core: &CoreGl, w: u32, h: u32) -> Vec<u8> {
    unsafe {
        let mut raw = vec![0u8; (w * h * 4) as usize];
        (core.read_pixels)(
            0,
            0,
            w as i32,
            h as i32,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            raw.as_mut_ptr() as *mut _,
        );
        let row = (w * 4) as usize;
        let mut out = vec![0u8; raw.len()];
        for y in 0..h as usize {
            let src = y * row;
            let dst = (h as usize - 1 - y) * row;
            if src + row <= raw.len() && dst + row <= out.len() {
                out[dst..dst + row].copy_from_slice(&raw[src..src + row]);
            }
        }
        out
    }
}

fn compile_one(procs: &Procs, kind: u32, src: &str) -> Result<u32, String> {
    unsafe {
        let id = (procs.create_shader)(kind);
        let c = CString::new(src).map_err(|e| e.to_string())?;
        let ptr = c.as_ptr();
        let len = src.len() as i32;
        (procs.shader_source)(id, 1, &ptr, &len);
        (procs.compile)(id);
        let mut status = 0i32;
        (procs.get_shaderiv)(id, GL_COMPILE_STATUS, &mut status);
        if status == 0 {
            let mut log_len = 0i32;
            (procs.get_shaderiv)(id, GL_INFO_LOG_LENGTH, &mut log_len);
            let mut buf = vec![0u8; log_len.max(1) as usize];
            (procs.get_shader_log)(id, buf.len() as i32, std::ptr::null_mut(), buf.as_mut_ptr() as *mut i8);
            let log = String::from_utf8_lossy(&buf).into_owned();
            return Err(format!("compile failed\n{log}\n{src}"));
        }
        Ok(id)
    }
}

fn program_log(procs: &Procs, prog: u32) -> String {
    unsafe {
        let mut log_len = 0i32;
        (procs.get_programiv)(prog, GL_INFO_LOG_LENGTH, &mut log_len);
        let mut buf = vec![0u8; log_len.max(1) as usize];
        (procs.get_program_log)(prog, buf.len() as i32, std::ptr::null_mut(), buf.as_mut_ptr() as *mut i8);
        String::from_utf8_lossy(&buf).into_owned()
    }
}

pub(super) fn parse_version(p: *const u8) -> (u32, u32) {
    if p.is_null() {
        return (1, 0);
    }
    unsafe {
        let s = CStr::from_ptr(p as *const i8).to_string_lossy();
        let s = s.trim_start_matches("OpenGL ES ");
        let mut parts = s.split(|c: char| !c.is_ascii_digit());
        let maj = parts.next().unwrap_or("1").parse().unwrap_or(1);
        let min = parts.next().unwrap_or("0").parse().unwrap_or(0);
        (maj, min)
    }
}

fn load_proc<T: Copy>(name: &str) -> Option<T> {
    gl_native::load_fn(name)
}

fn load_core() -> Result<CoreGl, String> {
    fn must<T: Copy>(name: &str) -> Result<T, String> {
        load_proc(name).ok_or_else(|| format!("{name} is missing"))
    }
    Ok(CoreGl {
        pixel_storei: must("glPixelStorei")?,
        get_string: must("glGetString")?,
        gen_textures: must("glGenTextures")?,
        bind_texture: must("glBindTexture")?,
        tex_parameteri: must("glTexParameteri")?,
        tex_image_2d: must("glTexImage2D")?,
        viewport: must("glViewport")?,
        disable: must("glDisable")?,
        clear_color: must("glClearColor")?,
        clear: must("glClear")?,
        finish: must("glFinish")?,
        read_pixels: must("glReadPixels")?,
        enable: load_proc("glEnable"),
        matrix_mode: load_proc("glMatrixMode"),
        load_identity: load_proc("glLoadIdentity"),
        tex_envi: load_proc("glTexEnvi"),
        color4f: load_proc("glColor4f"),
        begin: load_proc("glBegin"),
        tex_coord2f: load_proc("glTexCoord2f"),
        vertex4f: load_proc("glVertex4f"),
        end: load_proc("glEnd"),
    })
}

fn load_procs(tier: u32) -> Result<Procs, String> {
    fn must<T: Copy>(name: &str) -> Result<T, String> {
        load_proc(name).ok_or_else(|| format!("{name} is missing"))
    }
    let procs = Procs {
        create_shader: must("glCreateShader")?,
        shader_source: must("glShaderSource")?,
        compile: must("glCompileShader")?,
        get_shaderiv: must("glGetShaderiv")?,
        get_shader_log: must("glGetShaderInfoLog")?,
        create_program: must("glCreateProgram")?,
        attach: must("glAttachShader")?,
        link: must("glLinkProgram")?,
        use_program: must("glUseProgram")?,
        delete_shader: must("glDeleteShader")?,
        get_programiv: must("glGetProgramiv")?,
        get_program_log: must("glGetProgramInfoLog")?,
        bind_attrib: must("glBindAttribLocation")?,
        bind_frag: load_proc("glBindFragDataLocation"),
        get_uniform: must("glGetUniformLocation")?,
        uniform4fv: must("glUniform4fv")?,
        uniform1i: must("glUniform1i")?,
        get_block: if tier >= 3 { Some(must("glGetUniformBlockIndex")?) } else { None },
        block_binding: if tier >= 3 { Some(must("glUniformBlockBinding")?) } else { None },
        gen_buffers: must("glGenBuffers")?,
        bind_buffer: must("glBindBuffer")?,
        buffer_data: must("glBufferData")?,
        bind_buffer_base: if tier >= 3 { Some(must("glBindBufferBase")?) } else { None },
        enable_attrib: must("glEnableVertexAttribArray")?,
        disable_attrib: must("glDisableVertexAttribArray")?,
        attrib_ptr: must("glVertexAttribPointer")?,
        draw_arrays: must("glDrawArrays")?,
        gen_fb: load_proc("glGenFramebuffers").or_else(|| load_proc("glGenFramebuffersEXT")),
        bind_fb: load_proc("glBindFramebuffer").or_else(|| load_proc("glBindFramebufferEXT")),
        fb_tex: load_proc("glFramebufferTexture2D").or_else(|| load_proc("glFramebufferTexture2DEXT")),
        check_fb: load_proc("glCheckFramebufferStatus").or_else(|| load_proc("glCheckFramebufferStatusEXT")),
        active_tex: load_proc("glActiveTexture"),
        get_integerv: must("glGetIntegerv")?,
        gen_vao: load_proc("glGenVertexArrays"),
        bind_vao: load_proc("glBindVertexArray"),
        delete_textures: load_proc("glDeleteTextures"),
    };
    if tier >= 2 && (procs.gen_fb.is_none() || procs.bind_fb.is_none()) {
        return Err("framebuffer entry points are missing".into());
    }
    Ok(procs)
}

impl Procs {
    fn clone_fns(&self) -> ProcsFns {
        ProcsFns {
            use_program: self.use_program,
            disable_attrib: self.disable_attrib,
            gen_buffers: self.gen_buffers,
            bind_buffer: self.bind_buffer,
            buffer_data: self.buffer_data,
            enable_attrib: self.enable_attrib,
            attrib_ptr: self.attrib_ptr,
            draw_arrays: self.draw_arrays,
            bind_fb: self.bind_fb,
            bind_vao: self.bind_vao,
        }
    }
}

struct ProcsFns {
    use_program: GlUse,
    disable_attrib: GlDisableAttrib,
    gen_buffers: GlGenBuf,
    bind_buffer: GlBindBuf,
    buffer_data: GlBufData,
    enable_attrib: GlEnableAttrib,
    attrib_ptr: GlAttribPtr,
    draw_arrays: GlDrawArrays,
    bind_fb: Option<GlBindFb>,
    bind_vao: Option<GlBindVao>,
}
