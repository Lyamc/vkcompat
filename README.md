# vkcompat

Partial Vulkan 1.0 installable client driver. Shaders that translate straight
through run on the GPU. Loops, branches, and anything the host API cannot
express run on a CPU rasterizer in the same device.

On Windows the probe order, per physical GPU, is:

1. Direct3D 11 (feature level 11, otherwise 10). Software adapters are skipped.
2. If no Direct3D 11 device came up, Direct3D 9 shader model 3.
3. If no DirectX device came up, OpenGL. Tier 3 is GLSL 330, tier 2 is GLSL 120, tier 1 is fixed function.
4. A CPU device is always appended.

Linux and macOS builds do not compile Direct3D. They expose OpenGL (same tiers)
and the CPU device. Tier 3 and tier 2 run color, uniform-buffer, and texture
draws on the GPU. Tier 1 fixed function runs color and texture on the GPU.
Loops and compute stay on the CPU on every OpenGL tier. Compute is hardware
only for Windows Direct3D 11 feature level 11.

`VKCOMPAT_ONLY=d3d11|d3d9|gl|cpu` keeps a single kind and omits the CPU device.
`VKCOMPAT_ONLY=gl` forces OpenGL even when DirectX is present.
`VKCOMPAT_ALL_TIERS=1` lists OpenGL tiers 3, 2, and 1. `VKCOMPAT_TIER=1|2|3` forces one tier.

```
cargo build --release
target\release\vkcompat_test.exe
```

`cargo build` writes `vkcompat_icd.json` next to the shared library. `library_path`
is `.\vkcompat.dll` on Windows, `./libvkcompat.so` on Linux, and `./libvkcompat.dylib`
on macOS, so the loader resolves it from the manifest directory. It is not
`vulkan-1.dll`. Point the Khronos loader at that file:

```
set VK_DRIVER_FILES=C:\Build\graphics\vkcompat\target\release\vkcompat_icd.json
```

`vkcompat_loader.exe` loads `vulkan-1.dll` and enumerates devices through that
manifest. It does not link this crate. The shared library exports only
`vk_icdGetInstanceProcAddr`, `vk_icdGetPhysicalDeviceProcAddr`, and
`vk_icdNegotiateLoaderICDInterfaceVersion`.
