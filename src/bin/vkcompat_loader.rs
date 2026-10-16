//! Loads the system Vulkan loader and enumerates vkcompat devices.
//! This binary does not reference the vkcompat crate.

#[cfg(not(windows))]
fn main() {
    println!("vkcompat_loader: the system Vulkan loader probe runs on Windows");
}

#[cfg(windows)]
fn main() {
    let code = match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("FAIL {e}");
            1
        }
    };
    std::process::exit(code);
}

#[cfg(windows)]
fn run() -> Result<(), String> {
    use std::ffi::{c_char, c_void};
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::FreeLibrary;
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    use windows::core::PCWSTR;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let json_path = if let Some(set) = std::env::var_os("VK_DRIVER_FILES") {
        let text = set.to_string_lossy();
        let first = text.split(';').next().unwrap_or("").trim();
        if first.is_empty() {
            return Err("VK_DRIVER_FILES is empty".into());
        }
        std::path::PathBuf::from(first)
    } else {
        let dir = exe.parent().ok_or("exe has no directory")?;
        let path = dir.join("vkcompat_icd.json");
        std::env::set_var("VK_DRIVER_FILES", &path);
        path
    };
    println!("exe={}", exe.display());
    let json = std::fs::read_to_string(&json_path).map_err(|e| format!("read {}: {e}", json_path.display()))?;
    let library = json_library_path(&json).ok_or("ICD JSON has no library_path")?;
    if library_file_name(&library).eq_ignore_ascii_case("vulkan-1.dll") {
        return Err(format!("library_path must not be vulkan-1.dll (got {library})"));
    }
    if !library.contains('\\') && !library.contains('/') {
        return Err(format!("library_path must include a separator so the loader uses the manifest directory (got {library})"));
    }
    println!("library_path={library}");
    println!("VK_DRIVER_FILES={}", std::env::var("VK_DRIVER_FILES").unwrap_or_default());
    if std::env::var_os("VK_LOADER_LAYERS_DISABLE").is_none() {
        std::env::set_var("VK_LOADER_LAYERS_DISABLE", "~implicit~");
    }
    println!("VK_LOADER_LAYERS_DISABLE={}", std::env::var("VK_LOADER_LAYERS_DISABLE").unwrap_or_default());

    let loader = r"C:\Windows\System32\vulkan-1.dll";
    let wide: Vec<u16> = std::ffi::OsStr::new(loader).encode_wide().chain(std::iter::once(0)).collect();
    let lib = unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) }.map_err(|e| format!("LoadLibrary {loader}: {e}"))?;
    let gipa_raw = unsafe { GetProcAddress(lib, windows::core::s!("vkGetInstanceProcAddr")) }
        .ok_or("vkGetInstanceProcAddr missing from vulkan-1.dll")?;
    let gipa: GetIpa = unsafe { std::mem::transmute(gipa_raw) };

    let create_ptr = unsafe { gipa(std::ptr::null_mut(), b"vkCreateInstance\0".as_ptr()) };
    if create_ptr.is_null() {
        unsafe { let _ = FreeLibrary(lib); }
        return Err("loader vkCreateInstance is null".into());
    }
    type CreateInstance = unsafe extern "system" fn(*const InstInfo, *const c_void, *mut *mut c_void) -> i32;
    let create: CreateInstance = unsafe { std::mem::transmute(create_ptr) };

    let app_name = b"vkcompat_loader\0";
    let app = AppInfo {
        s_type: 0,
        p_next: std::ptr::null(),
        app_name: app_name.as_ptr() as *const c_char,
        app_ver: 0,
        engine: std::ptr::null(),
        engine_ver: 0,
        api: 1 << 22,
    };
    let info = InstInfo {
        s_type: 1,
        p_next: std::ptr::null(),
        flags: 0,
        app: &app,
        layers: 0,
        layer_names: std::ptr::null(),
        extensions: 0,
        extension_names: std::ptr::null(),
    };
    let mut instance = std::ptr::null_mut();
    let rc = unsafe { create(&info, std::ptr::null(), &mut instance) };
    if rc != 0 || instance.is_null() {
        unsafe { let _ = FreeLibrary(lib); }
        return Err(format!("vkCreateInstance -> {rc}"));
    }
    println!("instance ok");

    let enum_ptr = unsafe { gipa(instance, b"vkEnumeratePhysicalDevices\0".as_ptr()) };
    let props_ptr = unsafe { gipa(instance, b"vkGetPhysicalDeviceProperties\0".as_ptr()) };
    if enum_ptr.is_null() || props_ptr.is_null() {
        unsafe { let _ = FreeLibrary(lib); }
        return Err("loader device enumeration pointers are null".into());
    }
    type EnumDev = unsafe extern "system" fn(*mut c_void, *mut u32, *mut *mut c_void) -> i32;
    type GetProps = unsafe extern "system" fn(*mut c_void, *mut u8);
    let enum_dev: EnumDev = unsafe { std::mem::transmute(enum_ptr) };
    let get_props: GetProps = unsafe { std::mem::transmute(props_ptr) };

    let mut count = 0u32;
    let rc = unsafe { enum_dev(instance, &mut count, std::ptr::null_mut()) };
    if rc != 0 {
        unsafe { let _ = FreeLibrary(lib); }
        return Err(format!("vkEnumeratePhysicalDevices count -> {rc}"));
    }
    let mut devs = vec![std::ptr::null_mut(); count as usize];
    let rc = unsafe { enum_dev(instance, &mut count, devs.as_mut_ptr()) };
    if rc != 0 {
        unsafe { let _ = FreeLibrary(lib); }
        return Err(format!("vkEnumeratePhysicalDevices -> {rc}"));
    }
    println!("{count} physical device(s)");
    let mut chosen: *mut c_void = std::ptr::null_mut();
    for dev in devs.iter().take(count as usize) {
        let mut buf = vec![0u8; 4096];
        unsafe { get_props(*dev, buf.as_mut_ptr()) };
        let name = cstr_at(&buf, 20);
        if chosen.is_null() && name.contains("vkcompat") {
            chosen = *dev;
        }
        println!("{name}");
    }
    if chosen.is_null() {
        unsafe { let _ = FreeLibrary(lib); }
        return Err("no vkcompat physical device".into());
    }
    submit_empty(gipa, instance, chosen)?;
    unsafe { let _ = FreeLibrary(lib); }
    Ok(())
}

#[cfg(windows)]
fn submit_empty(gipa: GetIpa, instance: *mut std::ffi::c_void, physical: *mut std::ffi::c_void) -> Result<(), String> {
    use std::ffi::c_void;
    let create_dev = must_proc(gipa, instance, b"vkCreateDevice\0")?;
    let get_queue = must_proc(gipa, instance, b"vkGetDeviceQueue\0")?;
    let create_pool = must_proc(gipa, instance, b"vkCreateCommandPool\0")?;
    let alloc_cmd = must_proc(gipa, instance, b"vkAllocateCommandBuffers\0")?;
    let begin_cmd = must_proc(gipa, instance, b"vkBeginCommandBuffer\0")?;
    let end_cmd = must_proc(gipa, instance, b"vkEndCommandBuffer\0")?;
    let submit = must_proc(gipa, instance, b"vkQueueSubmit\0")?;

    type CreateDevice = unsafe extern "system" fn(*mut c_void, *const DevInfo, *const c_void, *mut *mut c_void) -> i32;
    type GetQueue = unsafe extern "system" fn(*mut c_void, u32, u32, *mut *mut c_void);
    type CreatePool = unsafe extern "system" fn(*mut c_void, *const PoolInfo, *const c_void, *mut *mut c_void) -> i32;
    type AllocCmd = unsafe extern "system" fn(*mut c_void, *const AllocInfo, *mut *mut c_void) -> i32;
    type BeginCmd = unsafe extern "system" fn(*mut c_void, *const BeginInfo) -> i32;
    type EndCmd = unsafe extern "system" fn(*mut c_void) -> i32;
    type QueueSubmit = unsafe extern "system" fn(*mut c_void, u32, *const SubmitInfo, *mut c_void) -> i32;
    let create_dev: CreateDevice = unsafe { std::mem::transmute(create_dev) };
    let get_queue: GetQueue = unsafe { std::mem::transmute(get_queue) };
    let create_pool: CreatePool = unsafe { std::mem::transmute(create_pool) };
    let alloc_cmd: AllocCmd = unsafe { std::mem::transmute(alloc_cmd) };
    let begin_cmd: BeginCmd = unsafe { std::mem::transmute(begin_cmd) };
    let end_cmd: EndCmd = unsafe { std::mem::transmute(end_cmd) };
    let submit: QueueSubmit = unsafe { std::mem::transmute(submit) };

    let priority = 1.0f32;
    let queue_info = QueueInfo {
        s_type: 2,
        p_next: std::ptr::null(),
        flags: 0,
        family: 0,
        count: 1,
        priorities: &priority,
    };
    let dev_info = DevInfo {
        s_type: 3,
        p_next: std::ptr::null(),
        flags: 0,
        queue_count: 1,
        queues: &queue_info,
        layer_count: 0,
        layer_names: std::ptr::null(),
        ext_count: 0,
        ext_names: std::ptr::null(),
        features: std::ptr::null(),
    };
    log_flush("creating device");
    let mut device = std::ptr::null_mut();
    let rc = unsafe { create_dev(physical, &dev_info, std::ptr::null(), &mut device) };
    if rc != 0 || device.is_null() {
        return Err(format!("vkCreateDevice -> {rc}"));
    }
    log_flush("device ok");
    let mut queue = std::ptr::null_mut();
    unsafe { get_queue(device, 0, 0, &mut queue) };
    if queue.is_null() {
        return Err("vkGetDeviceQueue returned null".into());
    }
    log_flush("queue ok");
    let pool_info = PoolInfo { s_type: 39, p_next: std::ptr::null(), flags: 0, family: 0 };
    let mut pool = std::ptr::null_mut();
    let rc = unsafe { create_pool(device, &pool_info, std::ptr::null(), &mut pool) };
    if rc != 0 || pool.is_null() {
        return Err(format!("vkCreateCommandPool -> {rc}"));
    }
    log_flush("pool ok");
    let alloc = AllocInfo { s_type: 40, p_next: std::ptr::null(), pool, level: 0, count: 1 };
    let mut cmd = std::ptr::null_mut();
    let rc = unsafe { alloc_cmd(device, &alloc, &mut cmd) };
    if rc != 0 || cmd.is_null() {
        return Err(format!("vkAllocateCommandBuffers -> {rc}"));
    }
    log_flush("alloc ok");
    let begin = BeginInfo { s_type: 42, p_next: std::ptr::null(), flags: 0, inheritance: std::ptr::null() };
    let rc = unsafe { begin_cmd(cmd, &begin) };
    if rc != 0 {
        return Err(format!("vkBeginCommandBuffer -> {rc}"));
    }
    log_flush("begin ok");
    let rc = unsafe { end_cmd(cmd) };
    if rc != 0 {
        return Err(format!("vkEndCommandBuffer -> {rc}"));
    }
    log_flush("end ok");
    let submit_info = SubmitInfo {
        s_type: 4,
        p_next: std::ptr::null(),
        wait_count: 0,
        waits: std::ptr::null(),
        wait_stages: std::ptr::null(),
        cmd_count: 1,
        cmds: &cmd,
        signal_count: 0,
        signals: std::ptr::null(),
    };
    let rc = unsafe { submit(queue, 1, &submit_info, std::ptr::null_mut()) };
    if rc != 0 {
        return Err(format!("vkQueueSubmit -> {rc}"));
    }
    log_flush("submit ok");
    Ok(())
}

#[cfg(windows)]
fn log_flush(msg: &str) {
    println!("{msg}");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

#[cfg(windows)]
fn must_proc(gipa: GetIpa, instance: *mut std::ffi::c_void, name: &[u8]) -> Result<*const (), String> {
    let ptr = unsafe { gipa(instance, name.as_ptr()) };
    if ptr.is_null() {
        let label = String::from_utf8_lossy(name).trim_end_matches('\0').to_string();
        Err(format!("loader {label} is null"))
    } else {
        Ok(ptr)
    }
}

#[cfg(windows)]
type GetIpa = unsafe extern "system" fn(*mut std::ffi::c_void, *const u8) -> *const ();

#[cfg(windows)]
#[repr(C)]
struct QueueInfo {
    s_type: i32,
    p_next: *const std::ffi::c_void,
    flags: u32,
    family: u32,
    count: u32,
    priorities: *const f32,
}

#[cfg(windows)]
#[repr(C)]
struct DevInfo {
    s_type: i32,
    p_next: *const std::ffi::c_void,
    flags: u32,
    queue_count: u32,
    queues: *const QueueInfo,
    layer_count: u32,
    layer_names: *const *const std::ffi::c_char,
    ext_count: u32,
    ext_names: *const *const std::ffi::c_char,
    features: *const std::ffi::c_void,
}

#[cfg(windows)]
#[repr(C)]
struct PoolInfo {
    s_type: i32,
    p_next: *const std::ffi::c_void,
    flags: u32,
    family: u32,
}

#[cfg(windows)]
#[repr(C)]
struct AllocInfo {
    s_type: i32,
    p_next: *const std::ffi::c_void,
    pool: *mut std::ffi::c_void,
    level: i32,
    count: u32,
}

#[cfg(windows)]
#[repr(C)]
struct BeginInfo {
    s_type: i32,
    p_next: *const std::ffi::c_void,
    flags: u32,
    inheritance: *const std::ffi::c_void,
}

#[cfg(windows)]
#[repr(C)]
struct SubmitInfo {
    s_type: i32,
    p_next: *const std::ffi::c_void,
    wait_count: u32,
    waits: *const std::ffi::c_void,
    wait_stages: *const u32,
    cmd_count: u32,
    cmds: *const *mut std::ffi::c_void,
    signal_count: u32,
    signals: *const std::ffi::c_void,
}

#[cfg(windows)]
#[repr(C)]
struct AppInfo {
    s_type: i32,
    p_next: *const std::ffi::c_void,
    app_name: *const std::ffi::c_char,
    app_ver: u32,
    engine: *const std::ffi::c_char,
    engine_ver: u32,
    api: u32,
}

#[cfg(windows)]
#[repr(C)]
struct InstInfo {
    s_type: i32,
    p_next: *const std::ffi::c_void,
    flags: u32,
    app: *const AppInfo,
    layers: u32,
    layer_names: *const *const std::ffi::c_char,
    extensions: u32,
    extension_names: *const *const std::ffi::c_char,
}

#[cfg(windows)]
fn json_library_path(json: &str) -> Option<String> {
    let key = "\"library_path\"";
    let i = json.find(key)?;
    let rest = &json[i + key.len()..];
    let q = rest.find('"')?;
    let rest = &rest[q + 1..];
    let end = rest.find('"')?;
    Some(json_unescape(&rest[..end]))
}

fn json_unescape(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn library_file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

#[cfg(windows)]
fn cstr_at(buf: &[u8], offset: usize) -> String {
    let bytes = buf.get(offset..).unwrap_or(&[]);
    let n = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..n]).into_owned()
}
