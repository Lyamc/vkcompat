//! Windows pixel contract. The binary calls the rlib and LoadLibrary's the ICD.
//! Non-Windows builds do not run the Win32 surface test; the library still exposes OpenGL and CPU.

#![allow(non_snake_case)]

#[cfg(windows)]
include!("../win_pixel_test.rs");

#[cfg(not(windows))]
fn main() {
    println!("vkcompat_test: pixel contract runs on Windows; this OS exposes OpenGL and the CPU device");
}
