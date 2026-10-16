//! Partial Vulkan 1.0 ICD.
//! Windows probes DirectX 11, then DirectX 9, then OpenGL, then a CPU rasterizer.
//! Linux and macOS expose OpenGL and the CPU rasterizer. Direct3D is Windows-only.
//! The shared library exports only the three `vk_icd*` entry points.

#![allow(non_snake_case)]

pub mod driver;
pub mod emit;
pub mod entry;
pub mod gpu;
pub mod icd;
pub mod raster;
pub mod spirv;
pub mod vk;

pub use driver::{gfx_pipeline_note, gfx_pipeline_uses_hw};
