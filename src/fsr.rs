#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(improper_ctypes)] // https://github.com/rust-lang/rust-bindgen/issues/1549

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

type VkPhysicalDevice = ash::vk::PhysicalDevice;
type VkDevice = ash::vk::Device;
type PFN_vkGetDeviceProcAddr = ash::vk::PFN_vkGetDeviceProcAddr;

bitflags::bitflags! {
    pub struct Fsr2InitializationFlags: u32 {
        const AUTO_EXPOSURE = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_AUTO_EXPOSURE;
        const INFINITE_DEPTH = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_DEPTH_INFINITE;
        const INVERTED_DEPTH = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_DEPTH_INVERTED;
        const DISPLAY_RESOLUTION_MOTION_VECTORS = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS;
        const JITTER_CANCELLED_MOTION_VECTORS = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION;
        const DYNAMIC_RESOLUTION = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_DYNAMIC_RESOLUTION;
        const HIGH_DYNAMIC_RANGE = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_HIGH_DYNAMIC_RANGE;
        const TEXTURE_1D = FfxFsr2InitializationFlagBits_FFX_FSR2_ENABLE_TEXTURE1D_USAGE;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fsr2QualityMode {
    Quality,
    Balanced,
    Performance,
    UltraPerformance,
}

pub struct Fsr2Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
}

pub enum Fsr2Exposure {
    AutoExposure,
    ManualExposure {
        pre_exposure: f32,
        exposure: Fsr2Texture,
    },
}

pub enum Fsr2ReactiveMask {
    NoMask,
    ManualMask(Fsr2Texture),
    AutoMask {
        color_opaque_only: Fsr2Texture,
        color_opauqe_and_transparent: Fsr2Texture,
        scale: f32,
        threshold: f32,
        binary_value: f32,
        flags: Fsr2AutoGenerateReactiveMaskFlags,
    },
}

bitflags::bitflags! {
    pub struct Fsr2AutoGenerateReactiveMaskFlags: u32 {
      // TODO
    }
}

pub enum Fsr2Sharpen {
    Disabled,
    Enabled { sharpness: f32 },
}

pub type Fsr2Resolution = FfxDimensions2D;
pub type Fsr2FloatCoordinates = FfxFloatCoords2D;
