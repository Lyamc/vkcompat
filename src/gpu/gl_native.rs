//! Platform OpenGL context. Shader draw state stays in `gl.rs`.
//! Windows uses WGL. Linux uses GLX. macOS uses CGL. Libraries are loaded at runtime.

#[cfg(not(windows))]
mod dl {
    use std::ffi::{c_char, c_void, CString};

    unsafe extern "C" {
        fn dlopen(filename: *const c_char, flags: i32) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }

    pub fn open(names: &[&str]) -> *mut c_void {
        for name in names {
            let Ok(c) = CString::new(*name) else { continue };
            let h = unsafe { dlopen(c.as_ptr(), 1) };
            if !h.is_null() {
                return h;
            }
        }
        std::ptr::null_mut()
    }

    pub fn addr(handle: *mut c_void, name: &str) -> Option<usize> {
        if handle.is_null() {
            return None;
        }
        let c = CString::new(name).ok()?;
        let p = unsafe { dlsym(handle, c.as_ptr()) };
        let n = p as usize;
        if n <= 3 { None } else { Some(n) }
    }

    pub fn sym<T: Copy>(handle: *mut c_void, name: &str) -> Result<T, String> {
        let n = addr(handle, name).ok_or_else(|| format!("{name} is missing"))?;
        unsafe {
            let ptr = n as *const ();
            Ok(std::mem::transmute_copy(&ptr))
        }
    }

    #[allow(dead_code)]
    pub fn sym_opt<T: Copy>(handle: *mut c_void, name: &str) -> Option<T> {
        let n = addr(handle, name)?;
        unsafe {
            let ptr = n as *const ();
            Some(std::mem::transmute_copy(&ptr))
        }
    }
}

pub(super) fn load_fn<T: Copy>(name: &str) -> Option<T> {
    let addr = plat::lookup(name)?;
    if addr <= 3 {
        return None;
    }
    unsafe {
        let ptr = addr as *const ();
        Some(std::mem::transmute_copy(&ptr))
    }
}

pub(super) fn current_version() -> (u32, u32) {
    type GetString = unsafe extern "system" fn(u32) -> *const u8;
    match load_fn::<GetString>("glGetString") {
        Some(get) => unsafe { super::parse_version(get(super::GL_VERSION)) },
        None => (1, 0),
    }
}

#[cfg(windows)]
mod plat {
    use std::ffi::CString;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC, HDC};
    use windows::Win32::Graphics::OpenGL::{
        wglCreateContext, wglDeleteContext, wglGetProcAddress, wglMakeCurrent, ChoosePixelFormat,
        SetPixelFormat, HGLRC, PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA,
        PIXELFORMATDESCRIPTOR,
    };
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};
    use windows::core::{s, PCSTR};

    const WGL_CONTEXT_MAJOR: i32 = 0x2091;
    const WGL_CONTEXT_MINOR: i32 = 0x2092;
    const WGL_CONTEXT_PROFILE_MASK: i32 = 0x9126;
    const WGL_CONTEXT_COMPATIBILITY: i32 = 0x0002;

    type WglCreateAttribs = unsafe extern "system" fn(HDC, HGLRC, *const i32) -> HGLRC;

    pub(crate) struct GlNative {
        hwnd: HWND,
        hdc: HDC,
        hglrc: HGLRC,
    }

    pub(super) fn lookup(name: &str) -> Option<usize> {
        let c = CString::new(name).ok()?;
        unsafe {
            let mut addr = wglGetProcAddress(PCSTR(c.as_ptr() as *const u8)).map(|f| f as usize).unwrap_or(0);
            if addr <= 3 {
                let lib = GetModuleHandleA(s!("opengl32.dll")).ok()?;
                addr = GetProcAddress(lib, PCSTR(c.as_ptr() as *const u8)).map(|f| f as usize).unwrap_or(0);
            }
            if addr <= 3 { None } else { Some(addr) }
        }
    }

    impl GlNative {
        pub(crate) fn create(tier: u32) -> Result<GlNative, String> {
            unsafe {
                let (hwnd, _) = crate::gpu::hidden_window("VkcompatHidden", "vkcompat", 64, 64)?;
                let hdc = GetDC(Some(hwnd));
                if hdc.0.is_null() {
                    crate::gpu::destroy_window(hwnd);
                    return Err("GetDC failed".into());
                }
                if let Err(e) = set_pixel_format(hdc) {
                    let _ = ReleaseDC(Some(hwnd), hdc);
                    crate::gpu::destroy_window(hwnd);
                    return Err(e);
                }
                let legacy = match wglCreateContext(hdc) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = ReleaseDC(Some(hwnd), hdc);
                        crate::gpu::destroy_window(hwnd);
                        return Err(format!("wglCreateContext {e}"));
                    }
                };
                if let Err(e) = wglMakeCurrent(hdc, legacy) {
                    release(hwnd, hdc, legacy);
                    return Err(format!("wglMakeCurrent {e}"));
                }
                let mut hglrc = legacy;
                if tier >= 2 {
                    if let Some(attribs) = super::load_fn::<WglCreateAttribs>("wglCreateContextAttribsARB") {
                        let (maj, min) = if tier >= 3 { (3, 3) } else { (2, 1) };
                        let list = [
                            WGL_CONTEXT_MAJOR,
                            maj,
                            WGL_CONTEXT_MINOR,
                            min,
                            WGL_CONTEXT_PROFILE_MASK,
                            WGL_CONTEXT_COMPATIBILITY,
                            0,
                        ];
                        let created = attribs(hdc, HGLRC::default(), list.as_ptr());
                        if !created.0.is_null() {
                            let _ = wglMakeCurrent(hdc, created);
                            let _ = wglDeleteContext(legacy);
                            hglrc = created;
                        } else if tier >= 3 {
                            let (vmaj, vmin) = super::current_version();
                            if vmaj < 3 || (vmaj == 3 && vmin < 3) {
                                release(hwnd, hdc, legacy);
                                return Err("OpenGL 3.3 context was not created".into());
                            }
                        }
                    } else if tier >= 3 {
                        let (vmaj, vmin) = super::current_version();
                        if vmaj < 3 || (vmaj == 3 && vmin < 3) {
                            release(hwnd, hdc, legacy);
                            return Err("wglCreateContextAttribsARB is missing".into());
                        }
                    }
                }
                let _ = wglMakeCurrent(hdc, hglrc);
                Ok(GlNative { hwnd, hdc, hglrc })
            }
        }

        pub(crate) fn make_current(&self) {
            unsafe {
                let _ = wglMakeCurrent(self.hdc, self.hglrc);
            }
        }

        pub(crate) fn resize(&mut self, w: i32, h: i32) {
            unsafe {
                let _ = SetWindowPos(self.hwnd, None, 0, 0, w, h, SWP_NOZORDER | SWP_NOACTIVATE);
            }
        }
    }

    impl Drop for GlNative {
        fn drop(&mut self) {
            unsafe { release(self.hwnd, self.hdc, self.hglrc) };
            self.hglrc = HGLRC::default();
            self.hdc = HDC::default();
            self.hwnd = HWND::default();
        }
    }

    unsafe fn release(hwnd: HWND, hdc: HDC, ctx: HGLRC) {
        if !ctx.0.is_null() {
            let _ = wglMakeCurrent(HDC::default(), HGLRC::default());
            let _ = wglDeleteContext(ctx);
        }
        if !hdc.0.is_null() {
            let _ = ReleaseDC(Some(hwnd), hdc);
        }
        if !hwnd.0.is_null() {
            crate::gpu::destroy_window(hwnd);
        }
    }

    fn set_pixel_format(hdc: HDC) -> Result<(), String> {
        unsafe {
            let mut pfd = PIXELFORMATDESCRIPTOR::default();
            pfd.nSize = std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16;
            pfd.nVersion = 1;
            pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
            pfd.iPixelType = PFD_TYPE_RGBA;
            pfd.cColorBits = 32;
            pfd.cDepthBits = 24;
            pfd.cStencilBits = 8;
            let fmt = ChoosePixelFormat(hdc, &pfd);
            if fmt == 0 {
                return Err("ChoosePixelFormat failed".into());
            }
            SetPixelFormat(hdc, fmt, &pfd).map_err(|e| format!("SetPixelFormat {e}"))?;
            Ok(())
        }
    }

}

#[cfg(target_os = "linux")]
mod plat {
    use std::ffi::{c_void, CString};

    type XOpen = unsafe extern "system" fn(*const u8) -> *mut c_void;
    type XClose = unsafe extern "system" fn(*mut c_void) -> i32;
    type XFreeFn = unsafe extern "system" fn(*mut c_void) -> i32;
    type XScreen = unsafe extern "system" fn(*mut c_void) -> i32;
    type ChooseFb = unsafe extern "system" fn(*mut c_void, i32, *const i32, *mut i32) -> *mut *mut c_void;
    type CreatePbuf = unsafe extern "system" fn(*mut c_void, *mut c_void, *const i32) -> usize;
    type DestroyPbuf = unsafe extern "system" fn(*mut c_void, usize);
    type CreateCtx = unsafe extern "system" fn(*mut c_void, *mut c_void, i32, *mut c_void, i32) -> *mut c_void;
    type DestroyCtx = unsafe extern "system" fn(*mut c_void, *mut c_void);
    type MakeCurrent = unsafe extern "system" fn(*mut c_void, usize, usize, *mut c_void) -> i32;
    type CreateAttribs = unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void, i32, *const i32) -> *mut c_void;

    const GLX_RGBA_BIT: i32 = 0x1;
    const GLX_PBUFFER_BIT: i32 = 0x4;
    const GLX_DOUBLEBUFFER: i32 = 5;
    const GLX_RED_SIZE: i32 = 8;
    const GLX_GREEN_SIZE: i32 = 9;
    const GLX_BLUE_SIZE: i32 = 10;
    const GLX_ALPHA_SIZE: i32 = 11;
    const GLX_DEPTH_SIZE: i32 = 12;
    const GLX_STENCIL_SIZE: i32 = 13;
    const GLX_DRAWABLE_TYPE: i32 = 0x8010;
    const GLX_RENDER_TYPE: i32 = 0x8011;
    const GLX_RGBA_TYPE: i32 = 0x8014;
    const GLX_PBUFFER_HEIGHT: i32 = 0x8040;
    const GLX_PBUFFER_WIDTH: i32 = 0x8041;
    const GLX_CONTEXT_MAJOR: i32 = 0x2091;
    const GLX_CONTEXT_MINOR: i32 = 0x2092;
    const GLX_CONTEXT_PROFILE_MASK: i32 = 0x9126;
    const GLX_CONTEXT_COMPATIBILITY: i32 = 0x2;

    pub(crate) struct GlNative {
        display: *mut c_void,
        fb: *mut c_void,
        pbuffer: usize,
        context: *mut c_void,
        close_display: XClose,
        destroy_context: DestroyCtx,
        destroy_pbuffer: DestroyPbuf,
        create_pbuffer: CreatePbuf,
        make_current_fn: MakeCurrent,
        width: i32,
        height: i32,
    }

    fn gl_lib() -> *mut c_void {
        static CELL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        *CELL.get_or_init(|| super::dl::open(&["libGL.so.1", "libGL.so"]) as usize) as *mut c_void
    }

    pub(super) fn lookup(name: &str) -> Option<usize> {
        let c = CString::new(name).ok()?;
        unsafe {
            if let Some(get) = glx_get_proc() {
                let p = get(c.as_ptr() as *const u8) as usize;
                if p > 3 {
                    return Some(p);
                }
            }
        }
        super::dl::addr(gl_lib(), name)
    }

    fn glx_get_proc() -> Option<unsafe extern "system" fn(*const u8) -> *mut c_void> {
        static CELL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        let addr = *CELL.get_or_init(|| {
            let gl = gl_lib();
            super::dl::addr(gl, "glXGetProcAddressARB")
                .or_else(|| super::dl::addr(gl, "glXGetProcAddress"))
                .unwrap_or(0)
        });
        if addr <= 3 {
            None
        } else {
            unsafe {
                let ptr = addr as *const ();
                Some(std::mem::transmute_copy(&ptr))
            }
        }
    }

    impl GlNative {
        pub(crate) fn create(tier: u32) -> Result<GlNative, String> {
            let x11 = super::dl::open(&["libX11.so.6", "libX11.so"]);
            let gl = gl_lib();
            if x11.is_null() {
                return Err("libX11 is missing".into());
            }
            if gl.is_null() {
                return Err("libGL is missing".into());
            }
            let xopen: XOpen = super::dl::sym(x11, "XOpenDisplay")?;
            let close_display: XClose = super::dl::sym(x11, "XCloseDisplay")?;
            let xfree: XFreeFn = super::dl::sym(x11, "XFree")?;
            let xscreen: XScreen = super::dl::sym(x11, "XDefaultScreen")?;
            let choose: ChooseFb = super::dl::sym(gl, "glXChooseFBConfig")?;
            let create_pbuffer: CreatePbuf = super::dl::sym(gl, "glXCreatePbuffer")?;
            let destroy_pbuffer: DestroyPbuf = super::dl::sym(gl, "glXDestroyPbuffer")?;
            let create_ctx: CreateCtx = super::dl::sym(gl, "glXCreateNewContext")?;
            let destroy_context: DestroyCtx = super::dl::sym(gl, "glXDestroyContext")?;
            let make_current_fn: MakeCurrent = super::dl::sym(gl, "glXMakeContextCurrent")?;
            unsafe {
                let display = xopen(std::ptr::null());
                if display.is_null() {
                    return Err("XOpenDisplay failed".into());
                }
                let screen = xscreen(display);
                let attrs = [
                    GLX_DRAWABLE_TYPE,
                    GLX_PBUFFER_BIT,
                    GLX_RENDER_TYPE,
                    GLX_RGBA_BIT,
                    GLX_DOUBLEBUFFER,
                    1,
                    GLX_RED_SIZE,
                    8,
                    GLX_GREEN_SIZE,
                    8,
                    GLX_BLUE_SIZE,
                    8,
                    GLX_ALPHA_SIZE,
                    8,
                    GLX_DEPTH_SIZE,
                    24,
                    GLX_STENCIL_SIZE,
                    8,
                    0,
                ];
                let mut n = 0i32;
                let list = choose(display, screen, attrs.as_ptr(), &mut n);
                if list.is_null() || n <= 0 {
                    close_display(display);
                    return Err("glXChooseFBConfig failed".into());
                }
                let fb = *list;
                xfree(list as *mut c_void);
                let pbuffer = make_pbuffer(create_pbuffer, display, fb, 64, 64);
                if pbuffer == 0 {
                    close_display(display);
                    return Err("glXCreatePbuffer failed".into());
                }
                let legacy = create_ctx(display, fb, GLX_RGBA_TYPE, std::ptr::null_mut(), 1);
                if legacy.is_null() {
                    destroy_pbuffer(display, pbuffer);
                    close_display(display);
                    return Err("glXCreateNewContext failed".into());
                }
                if make_current_fn(display, pbuffer, pbuffer, legacy) == 0 {
                    destroy_context(display, legacy);
                    destroy_pbuffer(display, pbuffer);
                    close_display(display);
                    return Err("glXMakeContextCurrent failed".into());
                }
                let mut context = legacy;
                if tier >= 2 {
                    let attribs: Option<CreateAttribs> = super::dl::sym_opt(gl, "glXCreateContextAttribsARB")
                        .or_else(|| super::load_fn("glXCreateContextAttribsARB"));
                    if let Some(attribs) = attribs {
                        let (maj, min) = if tier >= 3 { (3, 3) } else { (2, 1) };
                        let spec = [
                            GLX_CONTEXT_MAJOR,
                            maj,
                            GLX_CONTEXT_MINOR,
                            min,
                            GLX_CONTEXT_PROFILE_MASK,
                            GLX_CONTEXT_COMPATIBILITY,
                            0,
                        ];
                        let created = attribs(display, fb, std::ptr::null_mut(), 1, spec.as_ptr());
                        if !created.is_null() {
                            let _ = make_current_fn(display, pbuffer, pbuffer, created);
                            destroy_context(display, legacy);
                            context = created;
                        } else if tier >= 3 {
                            let (vmaj, vmin) = super::current_version();
                            if vmaj < 3 || (vmaj == 3 && vmin < 3) {
                                destroy_context(display, legacy);
                                destroy_pbuffer(display, pbuffer);
                                close_display(display);
                                return Err("OpenGL 3.3 context was not created".into());
                            }
                        }
                    } else if tier >= 3 {
                        let (vmaj, vmin) = super::current_version();
                        if vmaj < 3 || (vmaj == 3 && vmin < 3) {
                            destroy_context(display, legacy);
                            destroy_pbuffer(display, pbuffer);
                            close_display(display);
                            return Err("glXCreateContextAttribsARB is missing".into());
                        }
                    }
                }
                let _ = make_current_fn(display, pbuffer, pbuffer, context);
                Ok(GlNative {
                    display,
                    fb,
                    pbuffer,
                    context,
                    close_display,
                    destroy_context,
                    destroy_pbuffer,
                    create_pbuffer,
                    make_current_fn,
                    width: 64,
                    height: 64,
                })
            }
        }

        pub(crate) fn make_current(&self) {
            unsafe {
                let _ = (self.make_current_fn)(self.display, self.pbuffer, self.pbuffer, self.context);
            }
        }

        pub(crate) fn resize(&mut self, w: i32, h: i32) {
            if w == self.width && h == self.height && self.pbuffer != 0 {
                self.make_current();
                return;
            }
            unsafe {
                let _ = (self.make_current_fn)(self.display, 0, 0, std::ptr::null_mut());
                if self.pbuffer != 0 {
                    (self.destroy_pbuffer)(self.display, self.pbuffer);
                }
                self.pbuffer = make_pbuffer(self.create_pbuffer, self.display, self.fb, w, h);
                self.width = w;
                self.height = h;
                let _ = (self.make_current_fn)(self.display, self.pbuffer, self.pbuffer, self.context);
            }
        }
    }

    impl Drop for GlNative {
        fn drop(&mut self) {
            unsafe {
                let _ = (self.make_current_fn)(self.display, 0, 0, std::ptr::null_mut());
                if !self.context.is_null() {
                    (self.destroy_context)(self.display, self.context);
                    self.context = std::ptr::null_mut();
                }
                if self.pbuffer != 0 {
                    (self.destroy_pbuffer)(self.display, self.pbuffer);
                    self.pbuffer = 0;
                }
                if !self.display.is_null() {
                    (self.close_display)(self.display);
                    self.display = std::ptr::null_mut();
                }
            }
        }
    }

    fn make_pbuffer(create: CreatePbuf, display: *mut c_void, fb: *mut c_void, w: i32, h: i32) -> usize {
        let spec = [GLX_PBUFFER_WIDTH, w.max(1), GLX_PBUFFER_HEIGHT, h.max(1), 0];
        unsafe { create(display, fb, spec.as_ptr()) }
    }
}

#[cfg(target_os = "macos")]
mod plat {
    use std::ffi::c_void;

    type ChoosePf = unsafe extern "system" fn(*const i32, *mut *mut c_void, *mut i32) -> i32;
    type CreateCtx = unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> i32;
    type SetCurrent = unsafe extern "system" fn(*mut c_void) -> i32;
    type DestroyCtx = unsafe extern "system" fn(*mut c_void) -> i32;
    type DestroyPf = unsafe extern "system" fn(*mut c_void) -> i32;

    const ACCEL: i32 = 73;
    const COLOR: i32 = 8;
    const ALPHA: i32 = 11;
    const DEPTH: i32 = 12;
    const STENCIL: i32 = 13;
    const PROFILE: i32 = 99;
    const LEGACY: i32 = 0x1000;
    const GL3: i32 = 0x3200;
    const GL4: i32 = 0x4100;

    pub(crate) struct GlNative {
        ctx: *mut c_void,
        pix: *mut c_void,
        set_current: SetCurrent,
        destroy_ctx: DestroyCtx,
        destroy_pix: DestroyPf,
    }

    fn framework() -> *mut c_void {
        static CELL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        *CELL.get_or_init(|| {
            super::dl::open(&[
                "/System/Library/Frameworks/OpenGL.framework/OpenGL",
                "/System/Library/Frameworks/OpenGL.framework/Versions/Current/OpenGL",
            ]) as usize
        }) as *mut c_void
    }

    pub(super) fn lookup(name: &str) -> Option<usize> {
        super::dl::addr(framework(), name)
    }

    struct Fns {
        choose: ChoosePf,
        create: CreateCtx,
        set_current: SetCurrent,
        destroy_ctx: DestroyCtx,
        destroy_pix: DestroyPf,
    }

    impl GlNative {
        pub(crate) fn create(tier: u32) -> Result<GlNative, String> {
            let gl = framework();
            if gl.is_null() {
                return Err("OpenGL.framework is missing".into());
            }
            let fns = Fns {
                choose: super::dl::sym(gl, "CGLChoosePixelFormat")?,
                create: super::dl::sym(gl, "CGLCreateContext")?,
                set_current: super::dl::sym(gl, "CGLSetCurrentContext")?,
                destroy_ctx: super::dl::sym(gl, "CGLDestroyContext")?,
                destroy_pix: super::dl::sym(gl, "CGLDestroyPixelFormat")?,
            };
            let legacy_attrs = [ACCEL, COLOR, 32, ALPHA, 8, DEPTH, 24, STENCIL, 8, PROFILE, LEGACY, 0];
            let (mut ctx, mut pix) = make_ctx(&fns, &legacy_attrs)?;
            if tier >= 3 {
                let mut upgraded = false;
                for profile in [GL4, GL3] {
                    let attrs = [ACCEL, PROFILE, profile, COLOR, 32, ALPHA, 8, DEPTH, 24, 0];
                    if let Ok((next, next_pix)) = make_ctx(&fns, &attrs) {
                        let (maj, min) = super::current_version();
                        if maj > 3 || (maj == 3 && min >= 3) {
                            unsafe {
                                (fns.destroy_ctx)(ctx);
                                (fns.destroy_pix)(pix);
                            }
                            ctx = next;
                            pix = next_pix;
                            upgraded = true;
                            break;
                        }
                        unsafe {
                            (fns.destroy_ctx)(next);
                            (fns.destroy_pix)(next_pix);
                            (fns.set_current)(ctx);
                        }
                    }
                }
                if !upgraded {
                    let (maj, min) = super::current_version();
                    if maj < 3 || (maj == 3 && min < 3) {
                        unsafe {
                            (fns.set_current)(std::ptr::null_mut());
                            (fns.destroy_ctx)(ctx);
                            (fns.destroy_pix)(pix);
                        }
                        return Err("OpenGL 3.3 context was not created".into());
                    }
                }
            }
            Ok(GlNative {
                ctx,
                pix,
                set_current: fns.set_current,
                destroy_ctx: fns.destroy_ctx,
                destroy_pix: fns.destroy_pix,
            })
        }

        pub(crate) fn make_current(&self) {
            unsafe {
                let _ = (self.set_current)(self.ctx);
            }
        }

        pub(crate) fn resize(&mut self, _w: i32, _h: i32) {}
    }

    impl Drop for GlNative {
        fn drop(&mut self) {
            unsafe {
                let _ = (self.set_current)(std::ptr::null_mut());
                if !self.ctx.is_null() {
                    (self.destroy_ctx)(self.ctx);
                    self.ctx = std::ptr::null_mut();
                }
                if !self.pix.is_null() {
                    (self.destroy_pix)(self.pix);
                    self.pix = std::ptr::null_mut();
                }
            }
        }
    }

    fn make_ctx(fns: &Fns, attrs: &[i32]) -> Result<(*mut c_void, *mut c_void), String> {
        unsafe {
            let mut pix = std::ptr::null_mut();
            let mut n = 0i32;
            let err = (fns.choose)(attrs.as_ptr(), &mut pix, &mut n);
            if err != 0 || pix.is_null() {
                return Err(format!("CGLChoosePixelFormat {err}"));
            }
            let mut ctx = std::ptr::null_mut();
            let err = (fns.create)(pix, std::ptr::null_mut(), &mut ctx);
            if err != 0 || ctx.is_null() {
                (fns.destroy_pix)(pix);
                return Err(format!("CGLCreateContext {err}"));
            }
            let err = (fns.set_current)(ctx);
            if err != 0 {
                (fns.destroy_ctx)(ctx);
                (fns.destroy_pix)(pix);
                return Err(format!("CGLSetCurrentContext {err}"));
            }
            Ok((ctx, pix))
        }
    }
}

pub(super) use plat::GlNative;

#[cfg(all(not(windows), not(target_os = "linux"), not(target_os = "macos")))]
mod plat {
    pub(crate) struct GlNative;

    pub(super) fn lookup(_name: &str) -> Option<usize> {
        None
    }

    impl GlNative {
        pub(crate) fn create(_tier: u32) -> Result<GlNative, String> {
            Err("OpenGL contexts are implemented for Windows, Linux, and macOS".into())
        }
        pub(crate) fn make_current(&self) {}
        pub(crate) fn resize(&mut self, _w: i32, _h: i32) {}
    }
}
