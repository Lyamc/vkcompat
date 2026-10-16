//! Vulkan 1.0 ABI used by this ICD. Layouts match the Win64 Vulkan headers
//! (dispatchable and non-dispatchable handles are both pointers).
#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::{c_char, c_void};

pub const ICD_LOADER_MAGIC: usize = 0x01CDC0DE;
pub const API_VERSION_1_0: u32 = 1 << 22;
pub const WHOLE_SIZE: u64 = u64::MAX;
pub const MAX_PHYSICAL_DEVICE_NAME_SIZE: usize = 256;
pub const UUID_SIZE: usize = 16;
pub const MAX_EXTENSION_NAME_SIZE: usize = 256;
pub const MAX_DESCRIPTION_SIZE: usize = 256;
pub const MAX_MEMORY_TYPES: usize = 32;
pub const MAX_MEMORY_HEAPS: usize = 16;

pub const SUCCESS: i32 = 0;
pub const NOT_READY: i32 = 1;
pub const TIMEOUT: i32 = 2;
pub const INCOMPLETE: i32 = 5;
pub const ERROR_OUT_OF_HOST_MEMORY: i32 = -1;
pub const ERROR_INITIALIZATION_FAILED: i32 = -3;
pub const ERROR_LAYER_NOT_PRESENT: i32 = -6;
pub const ERROR_EXTENSION_NOT_PRESENT: i32 = -7;
pub const ERROR_INCOMPATIBLE_DRIVER: i32 = -9;
pub const ERROR_FORMAT_NOT_SUPPORTED: i32 = -11;

pub const STRUCTURE_TYPE_APPLICATION_INFO: i32 = 0;
pub const STRUCTURE_TYPE_INSTANCE_CREATE_INFO: i32 = 1;
pub const STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO: i32 = 2;
pub const STRUCTURE_TYPE_DEVICE_CREATE_INFO: i32 = 3;
pub const STRUCTURE_TYPE_SUBMIT_INFO: i32 = 4;
pub const STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO: i32 = 5;
pub const STRUCTURE_TYPE_FENCE_CREATE_INFO: i32 = 8;
pub const STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO: i32 = 9;
pub const STRUCTURE_TYPE_BUFFER_CREATE_INFO: i32 = 12;
pub const STRUCTURE_TYPE_IMAGE_CREATE_INFO: i32 = 14;
pub const STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO: i32 = 15;
pub const STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO: i32 = 16;
pub const STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO: i32 = 18;
pub const STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO: i32 = 19;
pub const STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO: i32 = 20;
pub const STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO: i32 = 22;
pub const STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO: i32 = 23;
pub const STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO: i32 = 24;
pub const STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO: i32 = 26;
pub const STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO: i32 = 27;
pub const STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO: i32 = 28;
pub const STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO: i32 = 29;
pub const STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO: i32 = 30;
pub const STRUCTURE_TYPE_SAMPLER_CREATE_INFO: i32 = 31;
pub const STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO: i32 = 32;
pub const STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO: i32 = 33;
pub const STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO: i32 = 34;
pub const STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET: i32 = 35;
pub const STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO: i32 = 37;
pub const STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO: i32 = 38;
pub const STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO: i32 = 39;
pub const STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO: i32 = 40;
pub const STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO: i32 = 42;
pub const STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO: i32 = 43;
pub const STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2: i32 = 1_000_059_000;
pub const STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2: i32 = 1_000_059_001;
pub const STRUCTURE_TYPE_PHYSICAL_DEVICE_MEMORY_PROPERTIES_2: i32 = 1_000_059_006;
pub const STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR: i32 = 1_000_001_000;
pub const STRUCTURE_TYPE_PRESENT_INFO_KHR: i32 = 1_000_001_001;
pub const STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR: i32 = 1_000_009_000;

pub const FORMAT_UNDEFINED: i32 = 0;
pub const FORMAT_R8G8B8A8_UNORM: i32 = 37;
pub const FORMAT_B8G8R8A8_UNORM: i32 = 44;
pub const FORMAT_R32_SFLOAT: i32 = 100;
pub const FORMAT_R32G32_SFLOAT: i32 = 103;
pub const FORMAT_R32G32B32_SFLOAT: i32 = 106;
pub const FORMAT_R32G32B32A32_SFLOAT: i32 = 109;
pub const FORMAT_D32_SFLOAT: i32 = 126;
pub const FORMAT_D24_UNORM_S8_UINT: i32 = 129;

pub const FORMAT_FEATURE_SAMPLED_IMAGE_BIT: u32 = 0x1;
pub const FORMAT_FEATURE_VERTEX_BUFFER_BIT: u32 = 0x40;
pub const FORMAT_FEATURE_COLOR_ATTACHMENT_BIT: u32 = 0x80;
pub const FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT: u32 = 0x100;
pub const FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT: u32 = 0x200;
pub const FORMAT_FEATURE_BLIT_SRC_BIT: u32 = 0x400;
pub const FORMAT_FEATURE_BLIT_DST_BIT: u32 = 0x800;

pub const IMAGE_TILING_OPTIMAL: i32 = 0;
pub const IMAGE_TILING_LINEAR: i32 = 1;
pub const IMAGE_TYPE_2D: i32 = 1;
pub const IMAGE_VIEW_TYPE_2D: i32 = 1;
pub const SHARING_MODE_EXCLUSIVE: i32 = 0;
pub const IMAGE_LAYOUT_UNDEFINED: i32 = 0;
pub const IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL: i32 = 2;
pub const IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL: i32 = 5;
pub const IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL: i32 = 6;
pub const IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL: i32 = 7;

pub const SAMPLE_COUNT_1_BIT: u32 = 1;
pub const IMAGE_USAGE_TRANSFER_SRC_BIT: u32 = 0x1;
pub const IMAGE_USAGE_TRANSFER_DST_BIT: u32 = 0x2;
pub const IMAGE_USAGE_SAMPLED_BIT: u32 = 0x4;
pub const IMAGE_USAGE_COLOR_ATTACHMENT_BIT: u32 = 0x10;
pub const IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT: u32 = 0x20;

pub const IMAGE_ASPECT_COLOR_BIT: u32 = 1;
pub const BUFFER_USAGE_TRANSFER_SRC_BIT: u32 = 0x1;
pub const BUFFER_USAGE_TRANSFER_DST_BIT: u32 = 0x2;
pub const BUFFER_USAGE_UNIFORM_BUFFER_BIT: u32 = 0x10;
pub const BUFFER_USAGE_STORAGE_BUFFER_BIT: u32 = 0x20;
pub const BUFFER_USAGE_VERTEX_BUFFER_BIT: u32 = 0x80;

pub const MEMORY_PROPERTY_DEVICE_LOCAL_BIT: u32 = 0x1;
pub const MEMORY_PROPERTY_HOST_VISIBLE_BIT: u32 = 0x2;
pub const MEMORY_PROPERTY_HOST_COHERENT_BIT: u32 = 0x4;
pub const MEMORY_PROPERTY_HOST_CACHED_BIT: u32 = 0x8;
pub const MEMORY_HEAP_DEVICE_LOCAL_BIT: u32 = 0x1;

pub const QUEUE_GRAPHICS_BIT: u32 = 0x1;
pub const QUEUE_COMPUTE_BIT: u32 = 0x2;
pub const QUEUE_TRANSFER_BIT: u32 = 0x4;

pub const SHADER_STAGE_VERTEX_BIT: u32 = 0x1;
pub const SHADER_STAGE_FRAGMENT_BIT: u32 = 0x10;
pub const SHADER_STAGE_COMPUTE_BIT: u32 = 0x20;

pub const PRIMITIVE_TOPOLOGY_POINT_LIST: i32 = 0;
pub const PRIMITIVE_TOPOLOGY_LINE_LIST: i32 = 1;
pub const PRIMITIVE_TOPOLOGY_TRIANGLE_LIST: i32 = 3;
pub const PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP: i32 = 4;
pub const PRIMITIVE_TOPOLOGY_TRIANGLE_FAN: i32 = 5;
pub const POLYGON_MODE_FILL: i32 = 0;
pub const CULL_MODE_NONE: u32 = 0;
pub const CULL_MODE_FRONT_BIT: u32 = 0x1;
pub const CULL_MODE_BACK_BIT: u32 = 0x2;
pub const FRONT_FACE_COUNTER_CLOCKWISE: i32 = 0;
pub const FRONT_FACE_CLOCKWISE: i32 = 1;

pub const FILTER_NEAREST: i32 = 0;
pub const FILTER_LINEAR: i32 = 1;
pub const SAMPLER_ADDRESS_MODE_REPEAT: i32 = 0;
pub const SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT: i32 = 1;
pub const SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE: i32 = 2;

pub const DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER: i32 = 1;
pub const DESCRIPTOR_TYPE_UNIFORM_BUFFER: i32 = 6;
pub const DESCRIPTOR_TYPE_STORAGE_BUFFER: i32 = 7;

pub const PIPELINE_BIND_POINT_GRAPHICS: i32 = 0;
pub const PIPELINE_BIND_POINT_COMPUTE: i32 = 1;
pub const COMMAND_BUFFER_LEVEL_PRIMARY: i32 = 0;
pub const SUBPASS_CONTENTS_INLINE: i32 = 0;
pub const DYNAMIC_STATE_VIEWPORT: i32 = 0;
pub const DYNAMIC_STATE_SCISSOR: i32 = 1;

pub const PHYSICAL_DEVICE_TYPE_OTHER: i32 = 0;
pub const PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU: i32 = 1;
pub const PHYSICAL_DEVICE_TYPE_DISCRETE_GPU: i32 = 2;
pub const PHYSICAL_DEVICE_TYPE_CPU: i32 = 4;

pub const PRESENT_MODE_IMMEDIATE_KHR: i32 = 0;
pub const PRESENT_MODE_FIFO_KHR: i32 = 2;
pub const COLOR_SPACE_SRGB_NONLINEAR_KHR: i32 = 0;
pub const COMPOSITE_ALPHA_OPAQUE_BIT_KHR: u32 = 0x1;
pub const SURFACE_TRANSFORM_IDENTITY_BIT_KHR: u32 = 0x1;

pub const EXT_SURFACE: &str = "VK_KHR_surface";
pub const EXT_WIN32_SURFACE: &str = "VK_KHR_win32_surface";
pub const EXT_PROPS2: &str = "VK_KHR_get_physical_device_properties2";
pub const EXT_SWAPCHAIN: &str = "VK_KHR_swapchain";
pub const EXT_DYNAMIC_RENDERING: &str = "VK_KHR_dynamic_rendering";

pub fn instance_extension_names() -> &'static [&'static str] {
    #[cfg(windows)]
    {
        &[EXT_SURFACE, EXT_WIN32_SURFACE, EXT_PROPS2]
    }
    #[cfg(not(windows))]
    {
        &[EXT_SURFACE, EXT_PROPS2]
    }
}

pub type VkResult = i32;
pub type VkBool32 = u32;
pub type VkFlags = u32;
pub type VkDeviceSize = u64;
pub type VkInstance = *mut c_void;
pub type VkPhysicalDevice = *mut c_void;
pub type VkDevice = *mut c_void;
pub type VkQueue = *mut c_void;
pub type VkCommandBuffer = *mut c_void;
pub type VkBuffer = *mut c_void;
pub type VkImage = *mut c_void;
pub type VkImageView = *mut c_void;
pub type VkShaderModule = *mut c_void;
pub type VkPipeline = *mut c_void;
pub type VkPipelineLayout = *mut c_void;
pub type VkPipelineCache = *mut c_void;
pub type VkDescriptorSetLayout = *mut c_void;
pub type VkDescriptorPool = *mut c_void;
pub type VkDescriptorSet = *mut c_void;
pub type VkFramebuffer = *mut c_void;
pub type VkRenderPass = *mut c_void;
pub type VkCommandPool = *mut c_void;
pub type VkSampler = *mut c_void;
pub type VkSemaphore = *mut c_void;
pub type VkFence = *mut c_void;
pub type VkDeviceMemory = *mut c_void;
pub type VkSurfaceKHR = *mut c_void;
pub type VkSwapchainKHR = *mut c_void;
pub type PfnVoidFunction = Option<unsafe extern "system" fn()>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkExtent2D {
    pub width: u32,
    pub height: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkExtent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkOffset2D {
    pub x: i32,
    pub y: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkOffset3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkRect2D {
    pub offset: VkOffset2D,
    pub extent: VkExtent2D,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub minDepth: f32,
    pub maxDepth: f32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkClearValue {
    pub float32: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkApplicationInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub pApplicationName: *const c_char,
    pub applicationVersion: u32,
    pub pEngineName: *const c_char,
    pub engineVersion: u32,
    pub apiVersion: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkInstanceCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub pApplicationInfo: *const VkApplicationInfo,
    pub enabledLayerCount: u32,
    pub ppEnabledLayerNames: *const *const c_char,
    pub enabledExtensionCount: u32,
    pub ppEnabledExtensionNames: *const *const c_char,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkMemoryHeap {
    pub size: VkDeviceSize,
    pub flags: VkFlags,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkMemoryType {
    pub propertyFlags: VkFlags,
    pub heapIndex: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPhysicalDeviceFeatures {
    pub robustBufferAccess: VkBool32,
    pub fullDrawIndexUint32: VkBool32,
    pub imageCubeArray: VkBool32,
    pub independentBlend: VkBool32,
    pub geometryShader: VkBool32,
    pub tessellationShader: VkBool32,
    pub sampleRateShading: VkBool32,
    pub dualSrcBlend: VkBool32,
    pub logicOp: VkBool32,
    pub multiDrawIndirect: VkBool32,
    pub drawIndirectFirstInstance: VkBool32,
    pub depthClamp: VkBool32,
    pub depthBiasClamp: VkBool32,
    pub fillModeNonSolid: VkBool32,
    pub depthBounds: VkBool32,
    pub wideLines: VkBool32,
    pub largePoints: VkBool32,
    pub alphaToOne: VkBool32,
    pub multiViewport: VkBool32,
    pub samplerAnisotropy: VkBool32,
    pub textureCompressionETC2: VkBool32,
    pub textureCompressionASTC_LDR: VkBool32,
    pub textureCompressionBC: VkBool32,
    pub occlusionQueryPrecise: VkBool32,
    pub pipelineStatisticsQuery: VkBool32,
    pub vertexPipelineStoresAndAtomics: VkBool32,
    pub fragmentStoresAndAtomics: VkBool32,
    pub shaderTessellationAndGeometryPointSize: VkBool32,
    pub shaderImageGatherExtended: VkBool32,
    pub shaderStorageImageExtendedFormats: VkBool32,
    pub shaderStorageImageMultisample: VkBool32,
    pub shaderStorageImageReadWithoutFormat: VkBool32,
    pub shaderStorageImageWriteWithoutFormat: VkBool32,
    pub shaderUniformBufferArrayDynamicIndexing: VkBool32,
    pub shaderSampledImageArrayDynamicIndexing: VkBool32,
    pub shaderStorageBufferArrayDynamicIndexing: VkBool32,
    pub shaderStorageImageArrayDynamicIndexing: VkBool32,
    pub shaderClipDistance: VkBool32,
    pub shaderCullDistance: VkBool32,
    pub shaderFloat64: VkBool32,
    pub shaderInt64: VkBool32,
    pub shaderInt16: VkBool32,
    pub shaderResourceResidency: VkBool32,
    pub shaderResourceMinLod: VkBool32,
    pub sparseBinding: VkBool32,
    pub sparseResidencyBuffer: VkBool32,
    pub sparseResidencyImage2D: VkBool32,
    pub sparseResidencyImage3D: VkBool32,
    pub sparseResidency2Samples: VkBool32,
    pub sparseResidency4Samples: VkBool32,
    pub sparseResidency8Samples: VkBool32,
    pub sparseResidency16Samples: VkBool32,
    pub sparseResidencyAliased: VkBool32,
    pub variableMultisampleRate: VkBool32,
    pub inheritedQueries: VkBool32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPhysicalDeviceLimits {
    pub maxImageDimension1D: u32,
    pub maxImageDimension2D: u32,
    pub maxImageDimension3D: u32,
    pub maxImageDimensionCube: u32,
    pub maxImageArrayLayers: u32,
    pub maxTexelBufferElements: u32,
    pub maxUniformBufferRange: u32,
    pub maxStorageBufferRange: u32,
    pub maxPushConstantsSize: u32,
    pub maxMemoryAllocationCount: u32,
    pub maxSamplerAllocationCount: u32,
    pub bufferImageGranularity: VkDeviceSize,
    pub sparseAddressSpaceSize: VkDeviceSize,
    pub maxBoundDescriptorSets: u32,
    pub maxPerStageDescriptorSamplers: u32,
    pub maxPerStageDescriptorUniformBuffers: u32,
    pub maxPerStageDescriptorStorageBuffers: u32,
    pub maxPerStageDescriptorSampledImages: u32,
    pub maxPerStageDescriptorStorageImages: u32,
    pub maxPerStageDescriptorInputAttachments: u32,
    pub maxPerStageResources: u32,
    pub maxDescriptorSetSamplers: u32,
    pub maxDescriptorSetUniformBuffers: u32,
    pub maxDescriptorSetUniformBuffersDynamic: u32,
    pub maxDescriptorSetStorageBuffers: u32,
    pub maxDescriptorSetStorageBuffersDynamic: u32,
    pub maxDescriptorSetSampledImages: u32,
    pub maxDescriptorSetStorageImages: u32,
    pub maxDescriptorSetInputAttachments: u32,
    pub maxVertexInputAttributes: u32,
    pub maxVertexInputBindings: u32,
    pub maxVertexInputAttributeOffset: u32,
    pub maxVertexInputBindingStride: u32,
    pub maxVertexOutputComponents: u32,
    pub maxTessellationGenerationLevel: u32,
    pub maxTessellationPatchSize: u32,
    pub maxTessellationControlPerVertexInputComponents: u32,
    pub maxTessellationControlPerVertexOutputComponents: u32,
    pub maxTessellationControlPerPatchOutputComponents: u32,
    pub maxTessellationControlTotalOutputComponents: u32,
    pub maxTessellationEvaluationInputComponents: u32,
    pub maxTessellationEvaluationOutputComponents: u32,
    pub maxGeometryShaderInvocations: u32,
    pub maxGeometryInputComponents: u32,
    pub maxGeometryOutputComponents: u32,
    pub maxGeometryOutputVertices: u32,
    pub maxGeometryTotalOutputComponents: u32,
    pub maxFragmentInputComponents: u32,
    pub maxFragmentOutputAttachments: u32,
    pub maxFragmentDualSrcAttachments: u32,
    pub maxFragmentCombinedOutputResources: u32,
    pub maxComputeSharedMemorySize: u32,
    pub maxComputeWorkGroupCount: [u32; 3],
    pub maxComputeWorkGroupInvocations: u32,
    pub maxComputeWorkGroupSize: [u32; 3],
    pub subPixelPrecisionBits: u32,
    pub subTexelPrecisionBits: u32,
    pub mipmapPrecisionBits: u32,
    pub maxDrawIndexedIndexValue: u32,
    pub maxDrawIndirectCount: u32,
    pub maxSamplerLodBias: f32,
    pub maxSamplerAnisotropy: f32,
    pub maxViewports: u32,
    pub maxViewportDimensions: [u32; 2],
    pub viewportBoundsRange: [f32; 2],
    pub viewportSubPixelBits: u32,
    pub minMemoryMapAlignment: usize,
    pub minTexelBufferOffsetAlignment: VkDeviceSize,
    pub minUniformBufferOffsetAlignment: VkDeviceSize,
    pub minStorageBufferOffsetAlignment: VkDeviceSize,
    pub minTexelOffset: i32,
    pub maxTexelOffset: u32,
    pub minTexelGatherOffset: i32,
    pub maxTexelGatherOffset: u32,
    pub minInterpolationOffset: f32,
    pub maxInterpolationOffset: f32,
    pub subPixelInterpolationOffsetBits: u32,
    pub maxFramebufferWidth: u32,
    pub maxFramebufferHeight: u32,
    pub maxFramebufferLayers: u32,
    pub framebufferColorSampleCounts: VkFlags,
    pub framebufferDepthSampleCounts: VkFlags,
    pub framebufferStencilSampleCounts: VkFlags,
    pub framebufferNoAttachmentsSampleCounts: VkFlags,
    pub maxColorAttachments: u32,
    pub sampledImageColorSampleCounts: VkFlags,
    pub sampledImageIntegerSampleCounts: VkFlags,
    pub sampledImageDepthSampleCounts: VkFlags,
    pub sampledImageStencilSampleCounts: VkFlags,
    pub storageImageSampleCounts: VkFlags,
    pub maxSampleMaskWords: u32,
    pub timestampComputeAndGraphics: VkBool32,
    pub timestampPeriod: f32,
    pub maxClipDistances: u32,
    pub maxCullDistances: u32,
    pub maxCombinedClipAndCullDistances: u32,
    pub discreteQueuePriorities: u32,
    pub pointSizeRange: [f32; 2],
    pub lineWidthRange: [f32; 2],
    pub pointSizeGranularity: f32,
    pub lineWidthGranularity: f32,
    pub strictLines: VkBool32,
    pub standardSampleLocations: VkBool32,
    pub optimalBufferCopyOffsetAlignment: VkDeviceSize,
    pub optimalBufferCopyRowPitchAlignment: VkDeviceSize,
    pub nonCoherentAtomSize: VkDeviceSize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPhysicalDeviceSparseProperties {
    pub residencyStandard2DBlockShape: VkBool32,
    pub residencyStandard2DMultisampleBlockShape: VkBool32,
    pub residencyStandard3DBlockShape: VkBool32,
    pub residencyAlignedMipSize: VkBool32,
    pub residencyNonResidentStrict: VkBool32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPhysicalDeviceProperties {
    pub apiVersion: u32,
    pub driverVersion: u32,
    pub vendorID: u32,
    pub deviceID: u32,
    pub deviceType: i32,
    pub deviceName: [c_char; MAX_PHYSICAL_DEVICE_NAME_SIZE],
    pub pipelineCacheUUID: [u8; UUID_SIZE],
    pub limits: VkPhysicalDeviceLimits,
    pub sparseProperties: VkPhysicalDeviceSparseProperties,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPhysicalDeviceMemoryProperties {
    pub memoryTypeCount: u32,
    pub memoryTypes: [VkMemoryType; MAX_MEMORY_TYPES],
    pub memoryHeapCount: u32,
    pub memoryHeaps: [VkMemoryHeap; MAX_MEMORY_HEAPS],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkQueueFamilyProperties {
    pub queueFlags: VkFlags,
    pub queueCount: u32,
    pub timestampValidBits: u32,
    pub minImageTransferGranularity: VkExtent3D,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkExtensionProperties {
    pub extensionName: [c_char; MAX_EXTENSION_NAME_SIZE],
    pub specVersion: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkFormatProperties {
    pub linearTilingFeatures: VkFlags,
    pub optimalTilingFeatures: VkFlags,
    pub bufferFeatures: VkFlags,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkImageFormatProperties {
    pub maxExtent: VkExtent3D,
    pub maxMipLevels: u32,
    pub maxArrayLayers: u32,
    pub sampleCounts: VkFlags,
    pub maxResourceSize: VkDeviceSize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkSurfaceCapabilitiesKHR {
    pub minImageCount: u32,
    pub maxImageCount: u32,
    pub currentExtent: VkExtent2D,
    pub minImageExtent: VkExtent2D,
    pub maxImageExtent: VkExtent2D,
    pub maxImageArrayLayers: u32,
    pub supportedTransforms: VkFlags,
    pub currentTransform: VkFlags,
    pub supportedCompositeAlpha: VkFlags,
    pub supportedUsageFlags: VkFlags,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkSurfaceFormatKHR {
    pub format: i32,
    pub colorSpace: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPhysicalDeviceProperties2 {
    pub sType: i32,
    pub pNext: *mut c_void,
    pub properties: VkPhysicalDeviceProperties,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPhysicalDeviceFeatures2 {
    pub sType: i32,
    pub pNext: *mut c_void,
    pub features: VkPhysicalDeviceFeatures,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPhysicalDeviceMemoryProperties2 {
    pub sType: i32,
    pub pNext: *mut c_void,
    pub memoryProperties: VkPhysicalDeviceMemoryProperties,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkDeviceQueueCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub queueFamilyIndex: u32,
    pub queueCount: u32,
    pub pQueuePriorities: *const f32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkDeviceCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub queueCreateInfoCount: u32,
    pub pQueueCreateInfos: *const VkDeviceQueueCreateInfo,
    pub enabledLayerCount: u32,
    pub ppEnabledLayerNames: *const *const c_char,
    pub enabledExtensionCount: u32,
    pub ppEnabledExtensionNames: *const *const c_char,
    pub pEnabledFeatures: *const VkPhysicalDeviceFeatures,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkSubmitInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub waitSemaphoreCount: u32,
    pub pWaitSemaphores: *const VkSemaphore,
    pub pWaitDstStageMask: *const VkFlags,
    pub commandBufferCount: u32,
    pub pCommandBuffers: *const VkCommandBuffer,
    pub signalSemaphoreCount: u32,
    pub pSignalSemaphores: *const VkSemaphore,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkMemoryAllocateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub allocationSize: VkDeviceSize,
    pub memoryTypeIndex: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkMemoryRequirements {
    pub size: VkDeviceSize,
    pub alignment: VkDeviceSize,
    pub memoryTypeBits: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkBufferCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub size: VkDeviceSize,
    pub usage: VkFlags,
    pub sharingMode: i32,
    pub queueFamilyIndexCount: u32,
    pub pQueueFamilyIndices: *const u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkImageCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub imageType: i32,
    pub format: i32,
    pub extent: VkExtent3D,
    pub mipLevels: u32,
    pub arrayLayers: u32,
    pub samples: VkFlags,
    pub tiling: i32,
    pub usage: VkFlags,
    pub sharingMode: i32,
    pub queueFamilyIndexCount: u32,
    pub pQueueFamilyIndices: *const u32,
    pub initialLayout: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkImageSubresourceRange {
    pub aspectMask: VkFlags,
    pub baseMipLevel: u32,
    pub levelCount: u32,
    pub baseArrayLayer: u32,
    pub layerCount: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkImageViewCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub image: VkImage,
    pub viewType: i32,
    pub format: i32,
    pub components: [i32; 4],
    pub subresourceRange: VkImageSubresourceRange,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkShaderModuleCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub codeSize: usize,
    pub pCode: *const u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPipelineShaderStageCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub stage: VkFlags,
    pub module: VkShaderModule,
    pub pName: *const c_char,
    pub pSpecializationInfo: *const c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkVertexInputBindingDescription {
    pub binding: u32,
    pub stride: u32,
    pub inputRate: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkVertexInputAttributeDescription {
    pub location: u32,
    pub binding: u32,
    pub format: i32,
    pub offset: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPipelineVertexInputStateCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub vertexBindingDescriptionCount: u32,
    pub pVertexBindingDescriptions: *const VkVertexInputBindingDescription,
    pub vertexAttributeDescriptionCount: u32,
    pub pVertexAttributeDescriptions: *const VkVertexInputAttributeDescription,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPipelineInputAssemblyStateCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub topology: i32,
    pub primitiveRestartEnable: VkBool32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPipelineRasterizationStateCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub depthClampEnable: VkBool32,
    pub rasterizerDiscardEnable: VkBool32,
    pub polygonMode: i32,
    pub cullMode: VkFlags,
    pub frontFace: i32,
    pub depthBiasEnable: VkBool32,
    pub depthBiasConstantFactor: f32,
    pub depthBiasClamp: f32,
    pub depthBiasSlopeFactor: f32,
    pub lineWidth: f32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkGraphicsPipelineCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub stageCount: u32,
    pub pStages: *const VkPipelineShaderStageCreateInfo,
    pub pVertexInputState: *const VkPipelineVertexInputStateCreateInfo,
    pub pInputAssemblyState: *const VkPipelineInputAssemblyStateCreateInfo,
    pub pTessellationState: *const c_void,
    pub pViewportState: *const c_void,
    pub pRasterizationState: *const VkPipelineRasterizationStateCreateInfo,
    pub pMultisampleState: *const c_void,
    pub pDepthStencilState: *const c_void,
    pub pColorBlendState: *const c_void,
    pub pDynamicState: *const c_void,
    pub layout: VkPipelineLayout,
    pub renderPass: VkRenderPass,
    pub subpass: u32,
    pub basePipelineHandle: VkPipeline,
    pub basePipelineIndex: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkComputePipelineCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub stage: VkPipelineShaderStageCreateInfo,
    pub layout: VkPipelineLayout,
    pub basePipelineHandle: VkPipeline,
    pub basePipelineIndex: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPipelineLayoutCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub setLayoutCount: u32,
    pub pSetLayouts: *const VkDescriptorSetLayout,
    pub pushConstantRangeCount: u32,
    pub pPushConstantRanges: *const c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkDescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptorType: i32,
    pub descriptorCount: u32,
    pub stageFlags: VkFlags,
    pub pImmutableSamplers: *const VkSampler,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkDescriptorSetLayoutCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub bindingCount: u32,
    pub pBindings: *const VkDescriptorSetLayoutBinding,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkDescriptorPoolCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub maxSets: u32,
    pub poolSizeCount: u32,
    pub pPoolSizes: *const c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkDescriptorSetAllocateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub descriptorPool: VkDescriptorPool,
    pub descriptorSetCount: u32,
    pub pSetLayouts: *const VkDescriptorSetLayout,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkDescriptorBufferInfo {
    pub buffer: VkBuffer,
    pub offset: VkDeviceSize,
    pub range: VkDeviceSize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkDescriptorImageInfo {
    pub sampler: VkSampler,
    pub imageView: VkImageView,
    pub imageLayout: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkWriteDescriptorSet {
    pub sType: i32,
    pub pNext: *const c_void,
    pub dstSet: VkDescriptorSet,
    pub dstBinding: u32,
    pub dstArrayElement: u32,
    pub descriptorCount: u32,
    pub descriptorType: i32,
    pub pImageInfo: *const VkDescriptorImageInfo,
    pub pBufferInfo: *const VkDescriptorBufferInfo,
    pub pTexelBufferView: *const c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkFramebufferCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub renderPass: VkRenderPass,
    pub attachmentCount: u32,
    pub pAttachments: *const VkImageView,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkRenderPassCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub attachmentCount: u32,
    pub pAttachments: *const c_void,
    pub subpassCount: u32,
    pub pSubpasses: *const c_void,
    pub dependencyCount: u32,
    pub pDependencies: *const c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkCommandPoolCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub queueFamilyIndex: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkCommandBufferAllocateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub commandPool: VkCommandPool,
    pub level: i32,
    pub commandBufferCount: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkCommandBufferBeginInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub pInheritanceInfo: *const c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkRenderPassBeginInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub renderPass: VkRenderPass,
    pub framebuffer: VkFramebuffer,
    pub renderArea: VkRect2D,
    pub clearValueCount: u32,
    pub pClearValues: *const VkClearValue,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkImageSubresourceLayers {
    pub aspectMask: VkFlags,
    pub mipLevel: u32,
    pub baseArrayLayer: u32,
    pub layerCount: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkBufferImageCopy {
    pub bufferOffset: VkDeviceSize,
    pub bufferRowLength: u32,
    pub bufferImageHeight: u32,
    pub imageSubresource: VkImageSubresourceLayers,
    pub imageOffset: VkOffset3D,
    pub imageExtent: VkExtent3D,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkSamplerCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub magFilter: i32,
    pub minFilter: i32,
    pub mipmapMode: i32,
    pub addressModeU: i32,
    pub addressModeV: i32,
    pub addressModeW: i32,
    pub mipLodBias: f32,
    pub anisotropyEnable: VkBool32,
    pub maxAnisotropy: f32,
    pub compareEnable: VkBool32,
    pub compareOp: i32,
    pub minLod: f32,
    pub maxLod: f32,
    pub borderColor: i32,
    pub unnormalizedCoordinates: VkBool32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkSemaphoreCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkFenceCreateInfo {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkWin32SurfaceCreateInfoKHR {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub hinstance: *mut c_void,
    pub hwnd: *mut c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkSwapchainCreateInfoKHR {
    pub sType: i32,
    pub pNext: *const c_void,
    pub flags: VkFlags,
    pub surface: VkSurfaceKHR,
    pub minImageCount: u32,
    pub imageFormat: i32,
    pub imageColorSpace: i32,
    pub imageExtent: VkExtent2D,
    pub imageArrayLayers: u32,
    pub imageUsage: VkFlags,
    pub imageSharingMode: i32,
    pub queueFamilyIndexCount: u32,
    pub pQueueFamilyIndices: *const u32,
    pub preTransform: VkFlags,
    pub compositeAlpha: VkFlags,
    pub presentMode: i32,
    pub clipped: VkBool32,
    pub oldSwapchain: VkSwapchainKHR,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VkPresentInfoKHR {
    pub sType: i32,
    pub pNext: *const c_void,
    pub waitSemaphoreCount: u32,
    pub pWaitSemaphores: *const VkSemaphore,
    pub swapchainCount: u32,
    pub pSwapchains: *const VkSwapchainKHR,
    pub pImageIndices: *const u32,
    pub pResults: *mut VkResult,
}

pub fn version_major(v: u32) -> u32 {
    (v >> 22) & 0x7f
}
pub fn version_minor(v: u32) -> u32 {
    (v >> 12) & 0x3ff
}

pub fn zeroed<T>() -> T {
    unsafe { std::mem::zeroed() }
}

pub unsafe fn cstr(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
}

pub fn name_bytes(name: &str) -> [c_char; MAX_PHYSICAL_DEVICE_NAME_SIZE] {
    let mut out = [0 as c_char; MAX_PHYSICAL_DEVICE_NAME_SIZE];
    let bytes = name.as_bytes();
    let n = bytes.len().min(MAX_PHYSICAL_DEVICE_NAME_SIZE - 1);
    for (i, b) in bytes.iter().take(n).enumerate() {
        out[i] = *b as c_char;
    }
    out
}

pub fn ext_bytes(name: &str) -> [c_char; MAX_EXTENSION_NAME_SIZE] {
    let mut out = [0 as c_char; MAX_EXTENSION_NAME_SIZE];
    let bytes = name.as_bytes();
    let n = bytes.len().min(MAX_EXTENSION_NAME_SIZE - 1);
    for (i, b) in bytes.iter().take(n).enumerate() {
        out[i] = *b as c_char;
    }
    out
}

pub fn advertised_limits() -> VkPhysicalDeviceLimits {
    let mut l: VkPhysicalDeviceLimits = zeroed();
    l.maxImageDimension1D = 8192;
    l.maxImageDimension2D = 8192;
    l.maxImageDimension3D = 1;
    l.maxImageDimensionCube = 1;
    l.maxImageArrayLayers = 1;
    l.maxTexelBufferElements = 65536;
    l.maxUniformBufferRange = 16384;
    l.maxStorageBufferRange = 1 << 27;
    l.maxPushConstantsSize = 128;
    l.maxMemoryAllocationCount = 4096;
    l.maxSamplerAllocationCount = 4000;
    l.bufferImageGranularity = 1;
    l.maxBoundDescriptorSets = 4;
    l.maxPerStageDescriptorSamplers = 16;
    l.maxPerStageDescriptorUniformBuffers = 12;
    l.maxPerStageDescriptorStorageBuffers = 8;
    l.maxPerStageDescriptorSampledImages = 16;
    l.maxPerStageResources = 64;
    l.maxDescriptorSetSamplers = 16;
    l.maxDescriptorSetUniformBuffers = 12;
    l.maxDescriptorSetStorageBuffers = 8;
    l.maxDescriptorSetSampledImages = 16;
    l.maxVertexInputAttributes = 16;
    l.maxVertexInputBindings = 16;
    l.maxVertexInputAttributeOffset = 2047;
    l.maxVertexInputBindingStride = 2048;
    l.maxVertexOutputComponents = 64;
    l.maxFragmentInputComponents = 64;
    l.maxFragmentOutputAttachments = 4;
    l.maxFragmentCombinedOutputResources = 8;
    l.maxComputeSharedMemorySize = 16384;
    l.maxComputeWorkGroupCount = [65535, 65535, 65535];
    l.maxComputeWorkGroupInvocations = 128;
    l.maxComputeWorkGroupSize = [128, 128, 64];
    l.subPixelPrecisionBits = 4;
    l.subTexelPrecisionBits = 4;
    l.mipmapPrecisionBits = 4;
    l.maxDrawIndexedIndexValue = u32::MAX;
    l.maxDrawIndirectCount = 1;
    l.maxViewports = 1;
    l.maxViewportDimensions = [8192, 8192];
    l.viewportBoundsRange = [-16384.0, 16384.0];
    l.minMemoryMapAlignment = 64;
    l.minTexelBufferOffsetAlignment = 256;
    l.minUniformBufferOffsetAlignment = 256;
    l.minStorageBufferOffsetAlignment = 256;
    l.maxFramebufferWidth = 8192;
    l.maxFramebufferHeight = 8192;
    l.maxFramebufferLayers = 1;
    l.framebufferColorSampleCounts = SAMPLE_COUNT_1_BIT;
    l.framebufferDepthSampleCounts = SAMPLE_COUNT_1_BIT;
    l.framebufferStencilSampleCounts = SAMPLE_COUNT_1_BIT;
    l.framebufferNoAttachmentsSampleCounts = SAMPLE_COUNT_1_BIT;
    l.maxColorAttachments = 4;
    l.sampledImageColorSampleCounts = SAMPLE_COUNT_1_BIT;
    l.sampledImageIntegerSampleCounts = SAMPLE_COUNT_1_BIT;
    l.sampledImageDepthSampleCounts = SAMPLE_COUNT_1_BIT;
    l.sampledImageStencilSampleCounts = SAMPLE_COUNT_1_BIT;
    l.storageImageSampleCounts = SAMPLE_COUNT_1_BIT;
    l.maxSampleMaskWords = 1;
    l.timestampPeriod = 1.0;
    l.maxClipDistances = 0;
    l.discreteQueuePriorities = 2;
    l.pointSizeRange = [1.0, 1.0];
    l.lineWidthRange = [1.0, 1.0];
    l.pointSizeGranularity = 1.0;
    l.lineWidthGranularity = 1.0;
    l.standardSampleLocations = 1;
    l.optimalBufferCopyOffsetAlignment = 1;
    l.optimalBufferCopyRowPitchAlignment = 1;
    l.nonCoherentAtomSize = 256;
    l
}

pub fn advertised_properties(name: &str, vendor: u32, device: u32, ty: i32) -> VkPhysicalDeviceProperties {
    let mut p: VkPhysicalDeviceProperties = zeroed();
    p.apiVersion = API_VERSION_1_0;
    p.driverVersion = 1;
    p.vendorID = vendor;
    p.deviceID = device;
    p.deviceType = ty;
    p.deviceName = name_bytes(name);
    p.limits = advertised_limits();
    p
}

pub fn advertised_memory() -> VkPhysicalDeviceMemoryProperties {
    let mut m: VkPhysicalDeviceMemoryProperties = zeroed();
    m.memoryTypeCount = 1;
    m.memoryTypes[0].propertyFlags = MEMORY_PROPERTY_DEVICE_LOCAL_BIT
        | MEMORY_PROPERTY_HOST_VISIBLE_BIT
        | MEMORY_PROPERTY_HOST_COHERENT_BIT
        | MEMORY_PROPERTY_HOST_CACHED_BIT;
    m.memoryTypes[0].heapIndex = 0;
    m.memoryHeapCount = 1;
    m.memoryHeaps[0].size = 512 * 1024 * 1024;
    m.memoryHeaps[0].flags = MEMORY_HEAP_DEVICE_LOCAL_BIT;
    m
}

pub fn advertised_queue() -> VkQueueFamilyProperties {
    VkQueueFamilyProperties {
        queueFlags: QUEUE_GRAPHICS_BIT | QUEUE_COMPUTE_BIT | QUEUE_TRANSFER_BIT,
        queueCount: 1,
        timestampValidBits: 0,
        minImageTransferGranularity: VkExtent3D {
            width: 1,
            height: 1,
            depth: 1,
        },
    }
}

pub fn format_bpp(fmt: i32) -> u32 {
    match fmt {
        FORMAT_R8G8B8A8_UNORM | FORMAT_B8G8R8A8_UNORM | FORMAT_D32_SFLOAT | FORMAT_D24_UNORM_S8_UINT
        | FORMAT_R32_SFLOAT => 4,
        FORMAT_R32G32_SFLOAT => 8,
        FORMAT_R32G32B32_SFLOAT => 12,
        FORMAT_R32G32B32A32_SFLOAT => 16,
        _ => 0,
    }
}

pub fn align_up(size: u64, align: u64) -> u64 {
    let align = align.max(1);
    size.div_ceil(align) * align
}
