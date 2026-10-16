use std::path::{Path, PathBuf};

fn artifact_dir(out: &Path) -> PathBuf {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let mut dir = out.to_path_buf();
    loop {
        if dir.file_name().and_then(|name| name.to_str()) == Some(profile.as_str()) {
            return dir;
        }
        if !dir.pop() {
            return out.to_path_buf();
        }
    }
}

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    // A bare filename is searched next to the client exe. A separator makes the
    // loader resolve the path against the manifest directory.
    let library = match os.as_str() {
        "windows" => ".\\\\vkcompat.dll",
        "macos" | "ios" => "./libvkcompat.dylib",
        _ => "./libvkcompat.so",
    };
    let json = format!(
        "{{\n    \"file_format_version\": \"1.0.0\",\n    \"ICD\": {{\n        \"library_path\": \"{library}\",\n        \"api_version\": \"1.0.0\"\n    }}\n}}\n"
    );
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dest = artifact_dir(&out).join("vkcompat_icd.json");
    if let Err(err) = std::fs::write(&dest, json) {
        println!("cargo:warning=could not write ICD JSON to {}: {err}", dest.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
}
